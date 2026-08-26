<#
.SYNOPSIS
    Runtime verification that Settings is bound to the daemon's lifetime.

.DESCRIPTION
    Requires Administrator: the daemon refuses to run unelevated, so a harness
    that has to start and kill one must be elevated itself.

    Checks the three halves of the rule that unit tests cannot reach, because
    each needs a real daemon process and a real window:

      refuses-no-daemon   with no daemon running, Settings shows a message box
                          and never builds its window
      waiver-opens        WIRADESK_SETTINGS_ALLOW_NO_DAEMON opens it anyway,
                          which is what keeps verify-settings-runtime.ps1 able
                          to run without Administrator
      closes-on-exit      Settings opened against a live daemon exits on its own
                          once that daemon is killed, and does so within the
                          poll interval plus slack

    `closes-on-exit` kills the daemon with Stop-Process rather than asking it to
    shut down: an unexpected death is the case the watch exists for, and a
    graceful exit reaches the same code path anyway (Windows destroys a dead
    process's windows either way).

    The daemon this repository normally runs comes from the `WiraDesk` logon
    task. If that task exists and its daemon is running when this starts, the
    harness stops it for the duration and starts it again at the end, so the
    desktop is left as it was found.

.EXAMPLE
    .\verify-settings-daemon-lifetime.ps1

.EXAMPLE
    .\verify-settings-daemon-lifetime.ps1 -SkipBuild -Configuration release
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [ValidateSet('debug', 'release')]
    [string]$Configuration = 'debug',
    # Poll interval in daemon_watch.rs is 500ms. The allowance is deliberately
    # generous: a debug-build UI thread on a loaded desktop can miss a tick.
    [int]$CloseTimeoutMs = 6000
)

$ErrorActionPreference = 'Stop'
# The workspace root, two levels up: this script lives in `scripts\`, and every
# path below is relative to the root that holds `target\` and `Cargo.toml`.
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $repoRoot

$WindowClass = 'WiraDeskDaemonHiddenWindow'
$WindowTitle = 'WiraDeskDaemon'
$WaiverEnv = 'WIRADESK_SETTINGS_ALLOW_NO_DAEMON'
$TaskName = 'WiraDesk'
$DialogClass = '#32770'

function Write-Step($m) { Write-Host "[lifetime] $m" -ForegroundColor Cyan }
function Write-Pass($m) { Write-Host "[lifetime] PASS: $m" -ForegroundColor Green }
function Write-Fail($m) { Write-Host "[lifetime] FAIL: $m" -ForegroundColor Red }

$results = [System.Collections.Generic.List[string]]::new()
function Record($name, [bool]$ok, [string]$detail = '') {
    if ($ok) { Write-Pass "$name $detail"; $results.Add("PASS $name") }
    else { Write-Fail "$name $detail"; $results.Add("FAIL $name") }
}

# cargo writes progress to stderr, which `$ErrorActionPreference = 'Stop'`
# turns into a terminating NativeCommandError even on success. Run it with the
# preference relaxed and judge the result by the exit code instead.
function Invoke-Cargo {
    param([string[]]$CargoArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & cargo @CargoArgs 2>&1 | Out-Null
        return $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $prev
    }
}

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Fail 'Administrator required - the daemon refuses to run unelevated, so this harness cannot start one.'
    Write-Host '           Open PowerShell as Administrator and run this script again.' -ForegroundColor Yellow
    exit 1
}

try {
    Add-Type -AssemblyName UIAutomationClient -ErrorAction Stop
    Add-Type -AssemblyName UIAutomationTypes -ErrorAction Stop
} catch {
    Write-Fail "UI Automation assemblies unavailable: $($_.Exception.Message)"
    exit 1
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WiraDeskLifetime {
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);
}
"@

function Test-DaemonWindow {
    return [WiraDeskLifetime]::FindWindowW($WindowClass, $WindowTitle) -ne [IntPtr]::Zero
}

function Wait-For {
    param([scriptblock]$Condition, [int]$TimeoutMs = 20000, [int]$PollMs = 200)
    $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
    while ((Get-Date) -lt $deadline) {
        if (& $Condition) { return $true }
        Start-Sleep -Milliseconds $PollMs
    }
    return $false
}

# ── Build ────────────────────────────────────────────────────────────────────

if (-not $SkipBuild) {
    Write-Step "Building daemon and settings ($Configuration) ..."
    $cargoArgs = @('build', '-p', 'daemon', '-p', 'settings')
    if ($Configuration -eq 'release') { $cargoArgs += '--release' }
    if ((Invoke-Cargo $cargoArgs) -ne 0) { Write-Fail 'build failed'; exit 1 }
}

$daemonExe = Join-Path $repoRoot "target\$Configuration\wiradesk.exe"
$settingsExe = Join-Path $repoRoot "target\$Configuration\wiradesk-settings.exe"
foreach ($p in @($daemonExe, $settingsExe)) {
    if (-not (Test-Path $p)) { Write-Fail "missing $p"; exit 1 }
}

# ── Take the desktop as we found it, to give it back ─────────────────────────

$task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
$installedWasRunning = $null -ne (Get-Process wiradesk -ErrorAction SilentlyContinue)
Write-Step "Daemon running at start: $installedWasRunning; logon task present: $($null -ne $task)"

$harnessDaemon = $null
$harnessSettings = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()

function Stop-AnyDaemon {
    Get-Process wiradesk -ErrorAction SilentlyContinue | ForEach-Object {
        try { $_.Kill(); $_.WaitForExit(5000) } catch {}
    }
    return (Wait-For { -not (Test-DaemonWindow) } 10000)
}

function Stop-HarnessSettings {
    foreach ($p in $harnessSettings) {
        try { if (-not $p.HasExited) { $p.Kill() } } catch {}
    }
    $harnessSettings.Clear()
}

function Restore-Desktop {
    Stop-HarnessSettings
    if ($harnessDaemon) {
        try { if (-not $harnessDaemon.HasExited) { $harnessDaemon.Kill() } } catch {}
    }
    Get-Process wiradesk -ErrorAction SilentlyContinue | ForEach-Object {
        try { $_.Kill(); $_.WaitForExit(5000) } catch {}
    }
    if ($installedWasRunning -and $task) {
        Write-Step "Restarting the $TaskName logon task ..."
        try {
            Start-ScheduledTask -TaskName $TaskName
            if (Wait-For { Test-DaemonWindow } 20000) { Write-Step 'Daemon is back.' }
            else { Write-Host "[lifetime] NOTE: daemon did not come back - start it yourself with: Start-ScheduledTask -TaskName $TaskName" -ForegroundColor Yellow }
        } catch {
            Write-Host "[lifetime] NOTE: could not restart the task: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    } elseif ($installedWasRunning) {
        Write-Host '[lifetime] NOTE: a daemon was running at start and there is no logon task - start it yourself.' -ForegroundColor Yellow
    }
}

trap { Write-Fail "unexpected error: $_"; Restore-Desktop; exit 1 }

# Every check below needs the desktop to start from "no daemon", including the
# two that then start one.
Write-Step 'Stopping any running daemon ...'
if (-not (Stop-AnyDaemon)) { Write-Fail 'the daemon hidden window is still there after killing every wiradesk process'; Restore-Desktop; exit 1 }

function Start-SettingsProcess {
    param([switch]$Waive)
    if ($Waive) { $env:WIRADESK_SETTINGS_ALLOW_NO_DAEMON = '1' }
    else { Remove-Item "Env:\$WaiverEnv" -ErrorAction SilentlyContinue }
    $p = Start-Process -FilePath $settingsExe -PassThru
    $harnessSettings.Add($p)
    return $p
}

# Reports what kind of top-level window a process put up, or $null for none.
function Get-ProcessWindow {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutMs = 15000)
    $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
    while ((Get-Date) -lt $deadline) {
        $Process.Refresh()
        if ($Process.HasExited) { return $null }
        if ($Process.MainWindowHandle -ne 0) {
            $el = [System.Windows.Automation.AutomationElement]::FromHandle($Process.MainWindowHandle)
            if ($el) {
                return @{ Class = $el.Current.ClassName; Name = $el.Current.Name; Element = $el }
            }
        }
        Start-Sleep -Milliseconds 250
    }
    return $null
}

function Get-WindowText {
    param($Element)
    if (-not $Element) { return '' }
    $all = $Element.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.Condition]::TrueCondition)
    return (($all | ForEach-Object { $_.Current.Name }) -join ' ')
}

# ── Check 1: no daemon, no waiver → a message box, never a window ────────────

Write-Step 'Check 1: launching Settings with no daemon running ...'
$s1 = Start-SettingsProcess
$w1 = Get-ProcessWindow -Process $s1
if (-not $w1) {
    Record 'refuses-no-daemon' $false 'no window appeared at all - expected the explanatory message box'
} else {
    $isDialog = $w1.Class -eq $DialogClass
    $text = Get-WindowText $w1.Element
    $saysWhy = $text -match 'not running'
    Record 'refuses-no-daemon' ($isDialog -and $saysWhy) "class='$($w1.Class)' name='$($w1.Name)' explains-why=$saysWhy"
    if (-not $isDialog) {
        Write-Host "[lifetime]   the window it opened was not a dialog - the Settings window itself must not be built without a daemon" -ForegroundColor Yellow
    }
}
Stop-HarnessSettings

# ── Check 2: the waiver opens it anyway ──────────────────────────────────────

Write-Step "Check 2: launching Settings with $WaiverEnv set ..."
$s2 = Start-SettingsProcess -Waive
$w2 = Get-ProcessWindow -Process $s2
$waiverOpened = ($null -ne $w2) -and ($w2.Class -ne $DialogClass)
Record 'waiver-opens' $waiverOpened "class='$(if ($w2) { $w2.Class } else { 'none' })' name='$(if ($w2) { $w2.Name } else { 'none' })'"
Stop-HarnessSettings
Remove-Item "Env:\$WaiverEnv" -ErrorAction SilentlyContinue

# ── Check 3: opened against a live daemon, closes when it dies ───────────────

Write-Step 'Check 3: starting a daemon, opening Settings against it, then killing the daemon ...'
$harnessDaemon = Start-Process -FilePath $daemonExe -PassThru -WindowStyle Hidden
if (-not (Wait-For { Test-DaemonWindow } 25000)) {
    Record 'closes-on-exit' $false 'the daemon never published its hidden window'
} else {
    Write-Step "Daemon ready (pid $($harnessDaemon.Id))."
    $s3 = Start-SettingsProcess
    $w3 = Get-ProcessWindow -Process $s3
    $opened = ($null -ne $w3) -and ($w3.Class -ne $DialogClass)
    Record 'opens-with-daemon' $opened "class='$(if ($w3) { $w3.Class } else { 'none' })'"

    if (-not $opened) {
        Record 'closes-on-exit' $false 'skipped - Settings never opened against the live daemon'
    } else {
        # Prove it is not simply closing on its own before the daemon dies.
        Start-Sleep -Milliseconds 2500
        $s3.Refresh()
        Record 'stays-open-while-daemon-lives' (-not $s3.HasExited) 'survived 5 poll intervals'

        Write-Step 'Killing the daemon (an unexpected death, as from Task Manager) ...'
        $clock = [System.Diagnostics.Stopwatch]::StartNew()
        try { $harnessDaemon.Kill(); $harnessDaemon.WaitForExit(5000) } catch {}
        $closed = Wait-For { $s3.Refresh(); $s3.HasExited } $CloseTimeoutMs 100
        $clock.Stop()
        Record 'closes-on-exit' $closed "$($clock.ElapsedMilliseconds)ms after the daemon died (allowance ${CloseTimeoutMs}ms)"
    }
}

# ── Teardown ─────────────────────────────────────────────────────────────────

Restore-Desktop

Write-Host ''
Write-Step '───────── SUMMARY ─────────'
$results | ForEach-Object {
    $color = if ($_ -like 'PASS*') { 'Green' } else { 'Red' }
    Write-Host "  $_" -ForegroundColor $color
}
$failed = ($results | Where-Object { $_ -like 'FAIL*' }).Count
$passed = ($results | Where-Object { $_ -like 'PASS*' }).Count
Write-Host ''
Write-Step "$passed passed, $failed failed"
if ($failed -gt 0) { exit 1 }
exit 0
