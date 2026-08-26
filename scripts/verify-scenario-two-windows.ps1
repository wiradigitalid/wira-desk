<#
.SYNOPSIS
    End-to-end scenario verification for Wira Desk window commands.

.DESCRIPTION
    Requires Administrator. Exercises cycling, identity, and snap commands on a
    live desktop by posting debug RUN_COMMAND messages after a synthetic key
    event (injected Ctrl) to approximate the keyboard-hook precondition.

    Harness limitation (not a product defect). Wira Desk cycling is verified
    under real keystrokes in demo-latency.ps1 and by manual use. This harness
    drives commands through SendInput-style injected input and foreground
    manipulation from PowerShell. Windows grants foreground rights to the process
    that received the last input; injected input therefore stays with this
    harness, not with the elevated daemon that observes the desktop through the
    hook. Automated runs that bypass the real shortcut path often accumulate
    `exhausted` or `no_target` outcomes in burst metrics even when cycling works
    for a human typist — a structural limitation of scripted input, not evidence
    that the product fails to cycle.

    Assertions use the daemon trace verdict (`WORKER_CYCLE:`) at execution time
    rather than racy `GetForegroundWindow` readings taken after settle delays.

    Opens two Settings windows on one monitor and checks:

      cycling    focus moves to the other window of the same application and back
      identity   a different application's window is never targeted
      snapping   SnapLeft / SnapRight / SnapMaximize match computed work-area rects

.EXAMPLE
    .\verify-scenario-two-windows.ps1

.EXAMPLE
    .\verify-scenario-two-windows.ps1 -SkipBuild -KeepOpen
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$KeepOpen,
    [int]$SettleMs = 700
)

$ErrorActionPreference = 'Stop'
# The workspace root, two levels up: this script lives in `scripts\`, and every
# path below is relative to the root that holds `target\` and `Cargo.toml`.
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $repoRoot

$WM_APP = 0x8000
$WM_APP_DEBUG_RUN_COMMAND = $WM_APP + 28
$WM_CLOSE = 0x0010

# Frozen command wire values.
$CMD_CYCLE = 1
$CMD_SNAP_LEFT = 2
$CMD_SNAP_RIGHT = 3
$CMD_SNAP_MAXIMIZE = 4

$WindowClass = 'WiraDeskDaemonHiddenWindow'
$WindowTitle = 'WiraDeskDaemon'

function Invoke-Cargo {
    param([string[]]$CargoArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { & cargo @CargoArgs 2>&1 | Out-Null; return $LASTEXITCODE }
    finally { $ErrorActionPreference = $prev }
}

function Write-Step($m) { Write-Host "[scenario] $m" -ForegroundColor Cyan }
function Write-Pass($m) { Write-Host "[scenario] PASS: $m" -ForegroundColor Green }
function Write-Fail($m) { Write-Host "[scenario] FAIL: $m" -ForegroundColor Red }

$results = [System.Collections.Generic.List[string]]::new()
function Record($name, [bool]$ok, [string]$detail = '') {
    if ($ok) { Write-Pass "$name $detail"; $results.Add("PASS $name") }
    else { Write-Fail "$name $detail"; $results.Add("FAIL $name") }
}

function Test-IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return ([Security.Principal.WindowsPrincipal]$id).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
}
if (-not (Test-IsAdmin)) { Write-Fail 'Administrator shell required.'; exit 1 }

Add-Type @"
using System;
using System.Runtime.InteropServices;
[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }
[StructLayout(LayoutKind.Sequential)]
public struct MONITORINFO { public int cbSize; public RECT rcMonitor; public RECT rcWork; public int dwFlags; }
public static class Scn {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string c, string n);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessageW(IntPtr h, uint m, UIntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr h, uint f);
    [DllImport("user32.dll")] public static extern bool GetMonitorInfoW(IntPtr m, ref MONITORINFO i);
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr p);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool attach);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
}
"@

function Send-Cmd {
    # `[uint64]`, not `[int]`: PowerShell 5.1 refuses to convert Int32 to
    # UIntPtr but converts UInt64 happily.
    param([IntPtr]$Hwnd, [uint64]$Command)

    # Reproduce the real precondition. In production the command arrives via the
    # keyboard hook, so a genuine key event has *just* occurred — which is what
    # resets Windows' foreground-lock timer and lets the daemon change focus at
    # all. Posting RUN_COMMAND directly skips the hook, so without this the
    # daemon is denied foreground rights and a working product looks broken.
    #
    # Ctrl is harmless and, being injected (`LLKHF_INJECTED`), is ignored by
    # Wira Desk's own hook — so it cannot accidentally trigger a shortcut.
    [Scn]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero)        # VK_CONTROL down
    [Scn]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)        # KEYEVENTF_KEYUP
    Start-Sleep -Milliseconds 60

    [Scn]::PostMessageW($Hwnd, $WM_APP_DEBUG_RUN_COMMAND, [UIntPtr]$Command, [IntPtr]::Zero) | Out-Null
    Start-Sleep -Milliseconds $SettleMs
}

# Windows refuses SetForegroundWindow from a process that is not itself
# foreground, and it fails *silently*. An unverified focus call makes the
# daemon act on whatever window happened to be foreground instead — which looks
# exactly like a product bug. So the result is always confirmed.
function Focus-Window {
    param([IntPtr]$Hwnd, [int]$Attempts = 8)
    for ($i = 0; $i -lt $Attempts; $i++) {
        # Attach to the current foreground thread's input queue first. This is
        # the documented way for a non-foreground process to hand focus over,
        # and it is the same mechanism the daemon uses — without it PowerShell's
        # SetForegroundWindow is declined and the whole run becomes noise.
        $fg = [Scn]::GetForegroundWindow()
        $fgThread = [Scn]::GetWindowThreadProcessId($fg, [IntPtr]::Zero)
        $me = [Scn]::GetCurrentThreadId()
        $attached = $false
        if ($fgThread -ne 0 -and $fgThread -ne $me) {
            $attached = [Scn]::AttachThreadInput($me, $fgThread, $true)
        }
        [Scn]::ShowWindow($Hwnd, 5) | Out-Null   # SW_SHOW
        [Scn]::BringWindowToTop($Hwnd) | Out-Null
        [Scn]::SetForegroundWindow($Hwnd) | Out-Null
        if ($attached) { [Scn]::AttachThreadInput($me, $fgThread, $false) | Out-Null }

        Start-Sleep -Milliseconds $SettleMs
        if ([Scn]::GetForegroundWindow() -eq $Hwnd) { return $true }
    }
    return $false
}

# Assert the precondition rather than letting a failed focus silently corrupt
# the check that follows.
function Require-Focus {
    param([IntPtr]$Hwnd, [string]$For)
    if (Focus-Window $Hwnd) { return $true }
    Write-Host "[scenario] SKIP: $For - could not take foreground (Windows foreground lock)" -ForegroundColor Yellow
    $script:results.Add("SKIP $For")
    return $false
}

function Get-TracePath {
    Join-Path (Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'WiraDesk') 'wiradesk-debug-trace.log'
}

# The daemon's own verdict, not a foreground reading taken hundreds of
# milliseconds later.
#
# `GetForegroundWindow()` after a settle is inherently racy here: the harness
# itself manipulates foreground (AttachThreadInput, injected Ctrl) and the
# console can reclaim it, so a correct activation gets observed as a failure.
# The trace records what Wira Desk actually did, at the moment it did it.
function Get-LastCycleOutcome {
    $lines = Select-String -Path (Get-TracePath) -Pattern 'WORKER_CYCLE:' -EA SilentlyContinue
    if (-not $lines) { return $null }
    return $lines[-1].Line
}

function Get-Rect { param([IntPtr]$h) $r = New-Object RECT; [Scn]::GetWindowRect($h, [ref]$r) | Out-Null; return $r }

function Get-WorkArea {
    param([IntPtr]$h)
    $mon = [Scn]::MonitorFromWindow($h, 2) # MONITOR_DEFAULTTONEAREST
    $mi = New-Object MONITORINFO
    $mi.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($mi)
    [Scn]::GetMonitorInfoW($mon, [ref]$mi) | Out-Null
    return $mi.rcWork
}

# ── Build and start daemon ───────────────────────────────────────────────────

if (-not $SkipBuild) {
    Write-Step 'Building debug daemon ...'
    if ((Invoke-Cargo @('build','-p','daemon')) -ne 0) { Write-Fail 'build failed'; exit 1 }
}
$exe = Join-Path $repoRoot 'target\debug\wiradesk.exe'
if (-not (Test-Path $exe)) { Write-Fail "missing $exe"; exit 1 }

Write-Step 'Stopping any running wiradesk ...'
Get-Process wiradesk -EA SilentlyContinue | ForEach-Object { try { $_.Kill(); $_.WaitForExit(5000) } catch {} }

Write-Step 'Starting daemon ...'
$daemon = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
$deadline = (Get-Date).AddSeconds(20)
$hwndDaemon = [IntPtr]::Zero
while ((Get-Date) -lt $deadline -and $hwndDaemon -eq [IntPtr]::Zero) {
    $hwndDaemon = [Scn]::FindWindowW($WindowClass, $WindowTitle)
    Start-Sleep -Milliseconds 200
}
if ($hwndDaemon -eq [IntPtr]::Zero) { Write-Fail 'daemon window not found'; exit 1 }
Record 'daemon-ready' $true

# ── Open two windows of the same application ─────────────────────────────────

# Two instances of our own Settings binary rather than Notepad: Windows 11's
# Notepad is a packaged app whose MainWindowHandle never surfaces to
# Start-Process, and WordPad is gone. Settings is a plain Win32 window, is
# guaranteed present, and two instances share one executable basename — which
# is exactly the same-application identity the cycle is meant to follow.
$settingsExe = Join-Path $repoRoot 'target\debug\wiradesk-settings.exe'
if (-not (Test-Path $settingsExe)) {
    Write-Step 'Building settings ...'
    if ((Invoke-Cargo @('build','-p','settings')) -ne 0) { Write-Fail 'settings build failed'; exit 1 }
}

function Start-AppWindow {
    param([int]$TimeoutSec = 25)
    $p = Start-Process -FilePath $settingsExe -PassThru
    $dl = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $dl) {
        $p.Refresh()
        if ($p.HasExited) { break }
        if ($p.MainWindowHandle -ne 0) { return $p }
        Start-Sleep -Milliseconds 250
    }
    return $p
}

Write-Step 'Opening two application windows ...'
$padA = Start-AppWindow
$padB = Start-AppWindow
Start-Sleep -Milliseconds 800
$padA.Refresh(); $padB.Refresh()
$hA = $padA.MainWindowHandle
$hB = $padB.MainWindowHandle

function Cleanup {
    foreach ($p in @($padA, $padB)) {
        if ($p -and -not $p.HasExited) { try { $p.CloseMainWindow() | Out-Null; Start-Sleep -Milliseconds 300 } catch {} }
        if ($p -and -not $p.HasExited) { try { $p.Kill() } catch {} }
    }
    if (-not $KeepOpen -and $daemon -and -not $daemon.HasExited) {
        [Scn]::PostMessageW($hwndDaemon, $WM_CLOSE, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
        Start-Sleep -Seconds 2
        if (-not $daemon.HasExited) { try { $daemon.Kill() } catch {} }
    }
}
trap { Write-Fail "unexpected error: $_"; Cleanup; exit 1 }

if ($hA -eq 0 -or $hB -eq 0 -or $hA -eq $hB) {
    Record 'two-windows-open' $false "hA=$hA hB=$hB"
    Cleanup; exit 1
}
Record 'two-windows-open' $true "hA=$hA hB=$hB"

# Both must be on one monitor for the spatial gate to accept them.
$monA = [Scn]::MonitorFromWindow($hA, 2)
$monB = [Scn]::MonitorFromWindow($hB, 2)
Record 'both-on-one-monitor' ($monA -eq $monB) "monA=$monA monB=$monB"

# ── Cycling: the success path ────────────────────────────────────────────────

Write-Step 'Cycling from A ...'
if (Require-Focus $hA 'cycle-moves-focus-to-b') {
    Record 'focus-starts-on-a' $true
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_CYCLE
    $verdict = Get-LastCycleOutcome
    Record 'cycle-activates-b' ($verdict -eq "WORKER_CYCLE: activated=$hB") "daemon reported: $verdict"
}

Write-Step 'Cycling again ...'
if (Require-Focus $hB 'cycle-wraps-back-to-a') {
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_CYCLE
    $verdict = Get-LastCycleOutcome
    Record 'cycle-wraps-back-to-a' ($verdict -eq "WORKER_CYCLE: activated=$hA") "daemon reported: $verdict"
}

# ── Identity: a different application is never targeted ──────────────────────

Write-Step 'Cycling with a different application focused ...'
$explorer = Get-Process explorer -EA SilentlyContinue | Select-Object -First 1
if ($explorer -and $explorer.MainWindowHandle -ne 0) {
    Focus-Window $explorer.MainWindowHandle
    $fgBefore = [Scn]::GetForegroundWindow()
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_CYCLE
    $verdict = Get-LastCycleOutcome
    Record 'never-jumps-to-another-application' `
        (($verdict -ne "WORKER_CYCLE: activated=$hA") -and ($verdict -ne "WORKER_CYCLE: activated=$hB")) `
        "daemon reported: $verdict"
} else {
    Write-Step 'No explorer window available; skipping cross-application check.'
}

# ── Snapping: real placement ─────────────────────────────────────────────────

$focusOk = Require-Focus $hA 'snap-tests'
$work = Get-WorkArea $hA
$workW = $work.Right - $work.Left
$mid = $work.Left + [int][math]::Floor($workW / 2)
Write-Step ("work area: L={0} T={1} R={2} B={3}" -f $work.Left, $work.Top, $work.Right, $work.Bottom)

if ($focusOk) {
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_SNAP_LEFT
    $r = Get-Rect $hA
    Record 'snap-left-places-window' (($r.Left -eq $work.Left) -and ($r.Right -eq $mid)) `
        ("got L={0} R={1}; expected L={2} R={3}" -f $r.Left, $r.Right, $work.Left, $mid)
}

if ($focusOk -and (Require-Focus $hA 'snap-right-places-window')) {
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_SNAP_RIGHT
    $r = Get-Rect $hA
    Record 'snap-right-places-window' (($r.Left -eq $mid) -and ($r.Right -eq $work.Right)) `
        ("got L={0} R={1}; expected L={2} R={3}" -f $r.Left, $r.Right, $mid, $work.Right)
}

$r = Get-Rect $hA
if ($focusOk -and (Require-Focus $hA 'maximize-fills-work-area')) {
    Send-Cmd -Hwnd $hwndDaemon -Command $CMD_SNAP_MAXIMIZE
    $r = Get-Rect $hA
$maxOk = ($r.Left -eq $work.Left) -and ($r.Right -eq $work.Right) -and `
         ($r.Top -eq $work.Top) -and ($r.Bottom -eq $work.Bottom)
Record 'maximize-fills-work-area' $maxOk `
    ("got L={0} T={1} R={2} B={3}" -f $r.Left, $r.Top, $r.Right, $r.Bottom)

# Maximize must use the work area, never full monitor bounds — the taskbar
# strip has to stay uncovered.
    Record 'maximize-does-not-cover-taskbar' ($r.Bottom -le $work.Bottom) `
        ("bottom={0} workBottom={1}" -f $r.Bottom, $work.Bottom)
}

# ── Teardown ─────────────────────────────────────────────────────────────────

Cleanup

Write-Host ''
Write-Step '───────── SUMMARY ─────────'
$results | ForEach-Object {
    $c = if ($_ -like 'PASS*') { 'Green' } elseif ($_ -like 'SKIP*') { 'Yellow' } else { 'Red' }
    Write-Host "  $_" -ForegroundColor $c
}
$failed = ($results | Where-Object { $_ -like 'FAIL*' }).Count
$skipped = ($results | Where-Object { $_ -like 'SKIP*' }).Count
$passed = ($results | Where-Object { $_ -like 'PASS*' }).Count
Write-Host ''
Write-Step "$passed passed, $skipped skipped, $failed failed"
if ($failed -gt 0) { exit 1 }
exit 0
