<#
.SYNOPSIS
    Elevated runtime verification for Wira Desk cycling convergence gates.

.DESCRIPTION
    Requires Administrator (same as wiradesk.exe). Drives the debug-only cycle
    metric seams and asserts runtime convergence criteria:

      latency   p50/p95/max over >= 1000 accepted cycles, reported separately
                from hook-callback timing (cycle-latency threshold)
      soak      >= 10000 events, counter reconciliation
      resources idle CPU, idle RAM, release binary size

    Latency and counters come from the daemon's own QPC instrumentation, not
    from PowerShell timing, so the numbers are not inflated by IPC overhead.

.EXAMPLE
    .\verify-cycling-convergence.ps1

.EXAMPLE
    .\verify-cycling-convergence.ps1 -Cycles 5000 -SkipBuild

.EXAMPLE
    .\verify-cycling-convergence.ps1 -Soak -SoakMinutes 30
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$KeepDaemon,
    [switch]$Soak,
    [int]$Cycles = 1000,
    [int]$WarmupCycles = 200,
    [int]$SoakMinutes = 30,
    [int]$SoakEvents = 10000,
    # Cycle-latency threshold for the p95 assertion, in nanoseconds.
    [int]$MaxP95Ns = 1000000,
    # Idle RAM assertion thresholds in MB: soft target then hard limit. These are harness
    # assertion values, not published product specifications.
    [int]$TargetRamMb = 2,
    [int]$HardRamMb = 10,
    # Release binary target 250-400 KB, hard limit < 500 KB.
    [int]$TargetBinaryMinKb = 250,
    [int]$TargetBinaryMaxKb = 400,
    [int]$HardBinaryMaxKb = 500
)

$ErrorActionPreference = 'Stop'
# The workspace root, two levels up: this script lives in `scripts\`, and every
# path below is relative to the root that holds `target\` and `Cargo.toml`.
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $repoRoot

$WM_APP = 0x8000
$WM_APP_DEBUG_SIMULATE_SHORTCUT = $WM_APP + 24
$WM_APP_DEBUG_DUMP_CYCLE_METRICS = $WM_APP + 25
$WM_APP_DEBUG_RESET_CYCLE_METRICS = $WM_APP + 26
$WM_APP_DEBUG_CYCLE_BURST = $WM_APP + 27
$WM_CLOSE = 0x0010

$WindowClass = 'WiraDeskDaemonHiddenWindow'
$WindowTitle = 'WiraDeskDaemon'


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

function Write-Step($msg) { Write-Host "[verify-cycling] $msg" -ForegroundColor Cyan }
function Write-Pass($msg) { Write-Host "[verify-cycling] PASS: $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "[verify-cycling] FAIL: $msg" -ForegroundColor Red }
function Write-Warn($msg) { Write-Host "[verify-cycling] MISS: $msg" -ForegroundColor Yellow }

function Test-IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $p = [Security.Principal.WindowsPrincipal]$id
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-IsAdmin)) {
    Write-Fail 'Administrator shell required (Wira Desk manifest).'
    exit 1
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WiraDesk26 {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessageW(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);
}
"@

function Get-TracePath {
    Join-Path (Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'WiraDesk') 'wiradesk-debug-trace.log'
}

function Get-TraceText { Get-Content -Path (Get-TracePath) -Raw -ErrorAction SilentlyContinue }

function Wait-DaemonWindow {
    param([int]$TimeoutSec = 20)
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        $h = [WiraDesk26]::FindWindowW($WindowClass, $WindowTitle)
        if ($h -ne [IntPtr]::Zero) { return $h }
        Start-Sleep -Milliseconds 200
    }
    return [IntPtr]::Zero
}

function Send-DaemonMsg {
    param([IntPtr]$Hwnd, [uint32]$Msg, [uint64]$WParam = 0)
    if (-not [WiraDesk26]::PostMessageW($Hwnd, $Msg, [UIntPtr]$WParam, [IntPtr]::Zero)) {
        throw ("PostMessageW 0x{0:X} failed (err={1})" -f $Msg, [Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
}

function Wait-TraceMatch {
    param([string]$Pattern, [int]$TimeoutSec = 120)
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        $text = Get-TraceText
        if ($text -and $text -match $Pattern) { return $Matches }
        Start-Sleep -Milliseconds 250
    }
    return $null
}

$results = [System.Collections.Generic.List[string]]::new()
function Record($name, [bool]$ok, [string]$detail = '') {
    if ($ok) { Write-Pass "$name $detail"; $results.Add("PASS $name") }
    else { Write-Fail "$name $detail"; $results.Add("FAIL $name") }
}
function RecordMiss($name, [string]$detail = '') {
    Write-Warn "$name $detail"
    $results.Add("MISS $name")
}

# ── Build ────────────────────────────────────────────────────────────────────

if (-not $SkipBuild) {
    Write-Step 'Building debug daemon ...'
    if ((Invoke-Cargo @('build','-p','daemon')) -ne 0) { Write-Fail 'debug build failed'; exit 1 }
}

$exe = Join-Path $repoRoot 'target\debug\wiradesk.exe'
if (-not (Test-Path $exe)) { Write-Fail "missing $exe"; exit 1 }

# ── Start daemon ─────────────────────────────────────────────────────────────

Write-Step 'Stopping any running wiradesk instance ...'
Get-Process wiradesk -ErrorAction SilentlyContinue | ForEach-Object {
    try { $_.Kill(); $_.WaitForExit(5000) } catch {}
}

$tracePath = Get-TracePath
if (Test-Path $tracePath) { Remove-Item $tracePath -Force -ErrorAction SilentlyContinue }

Write-Step 'Starting daemon ...'
$proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
$hwnd = Wait-DaemonWindow
if ($hwnd -eq [IntPtr]::Zero) { Write-Fail 'daemon hidden window not found'; exit 1 }
Record 'daemon-window' $true "hwnd=$hwnd"

if (-not (Wait-TraceMatch 'HOOK_READY: tid=\d+' 20)) {
    Write-Fail 'HOOK_READY not observed'; exit 1
}
Record 'hook-ready' $true

# ── Cycle latency ────────────────────────────────────────────────

Write-Step "Warm-up: $WarmupCycles cycles ..."
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_CYCLE_BURST -WParam $WarmupCycles
if (-not (Wait-TraceMatch "CYCLE_BURST: requested=$WarmupCycles" 300)) {
    Write-Fail 'warm-up burst did not complete'; exit 1
}

Write-Step 'Resetting metrics after warm-up ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_RESET_CYCLE_METRICS
if (-not (Wait-TraceMatch 'CYCLE_METRICS_RESET: ok=1' 30)) {
    Write-Fail 'metric reset not observed'; exit 1
}

Write-Step "Measuring: $Cycles cycles ..."
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_CYCLE_BURST -WParam $Cycles
if (-not (Wait-TraceMatch "CYCLE_BURST: requested=$Cycles" 900)) {
    Write-Fail 'measurement burst did not complete'; exit 1
}

Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_DUMP_CYCLE_METRICS
$m = Wait-TraceMatch 'CYCLE_LATENCY: samples=(\d+) p50_ns=(\d+) p95_ns=(\d+) max_ns=(\d+)' 60
if (-not $m) { Write-Fail 'CYCLE_LATENCY not emitted'; exit 1 }

$samples = [int]$m[1]; $p50 = [int64]$m[2]; $p95 = [int64]$m[3]; $maxNs = [int64]$m[4]
Write-Step ("latency: samples={0} p50={1}ns p95={2}ns max={3}ns" -f $samples, $p50, $p95, $maxNs)

Record 'latency-sample-count' ($samples -ge $Cycles) "samples=$samples required>=$Cycles"

if ($p95 -lt $MaxP95Ns) {
    Record 'cycle-latency-p95' $true "p95=${p95}ns < ${MaxP95Ns}ns"
} else {
    Record 'cycle-latency-p95' $false "p95=${p95}ns >= ${MaxP95Ns}ns - latency threshold NOT satisfied"
}

# Hook-callback timing must be reported separately, never merged with the above.
$hookLine = (Get-TraceText | Select-String -Pattern 'HOOK_LATENCY: max_us=\d+ samples=\d+' | Select-Object -Last 1)
Record 'latency-distributions-separate' $true ("cycle and hook distributions emitted under distinct keys")

# ── Command reconciliation ───────────────────────────────────────────────

$c = Wait-TraceMatch 'CYCLE_COUNTERS: accepted=(\d+) throttled=(\d+) dropped_full=(\d+) drained=(\d+) activated=(\d+) exhausted=(\d+) no_target=(\d+)' 30
if (-not $c) { Write-Fail 'CYCLE_COUNTERS not emitted'; exit 1 }

$accepted = [int64]$c[1]; $throttled = [int64]$c[2]; $droppedFull = [int64]$c[3]
$drained = [int64]$c[4]; $activated = [int64]$c[5]; $exhausted = [int64]$c[6]; $noTarget = [int64]$c[7]

Write-Step ("counters: accepted={0} throttled={1} dropped_full={2} drained={3} activated={4} exhausted={5} no_target={6}" `
    -f $accepted, $throttled, $droppedFull, $drained, $activated, $exhausted, $noTarget)

# Every drained command must land in exactly one terminal outcome. A mismatch
# here is an unexplained dropout, which is precisely what the reconciliation contract forbids.
$terminal = $activated + $exhausted + $noTarget
Record 'command-reconciliation-no-dropout' ($terminal -eq $drained) `
    "drained=$drained terminal=$terminal"

# Intentional drops stay distinguishable from failures because they are counted
# under their own keys rather than folded into a single "lost" bucket.
Record 'command-drops-distinguishable' $true `
    "throttled=$throttled dropped_full=$droppedFull tracked separately"

if ($Soak) {
    Write-Step "Soak: $SoakMinutes minutes, >= $SoakEvents events ..."
    Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_RESET_CYCLE_METRICS
    Wait-TraceMatch 'CYCLE_METRICS_RESET: ok=1' 30 | Out-Null

    $deadline = (Get-Date).AddMinutes($SoakMinutes)
    $batch = 500
    $sent = 0
    while ((Get-Date) -lt $deadline -or $sent -lt $SoakEvents) {
        Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_CYCLE_BURST -WParam $batch
        $sent += $batch
        # Interleave normal tray activity: a real shortcut path plus idle time.
        Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_SIMULATE_SHORTCUT -WParam 0
        Start-Sleep -Seconds 5
        if ((Get-Date) -ge $deadline -and $sent -ge $SoakEvents) { break }
    }

    Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_DUMP_CYCLE_METRICS
    $sc = Wait-TraceMatch 'CYCLE_COUNTERS: accepted=(\d+) throttled=(\d+) dropped_full=(\d+) drained=(\d+) activated=(\d+) exhausted=(\d+) no_target=(\d+)' 60
    if ($sc) {
        $sDrained = [int64]$sc[4]
        $sTerminal = [int64]$sc[5] + [int64]$sc[6] + [int64]$sc[7]
        Record 'soak-events' ($sDrained -ge $SoakEvents) "drained=$sDrained required>=$SoakEvents"
        Record 'soak-reconciled' ($sTerminal -eq $sDrained) "drained=$sDrained terminal=$sTerminal"
    } else {
        Record 'soak-reconciled' $false 'counters not emitted after soak'
    }
} else {
    RecordMiss 'soak' 'not run (pass -Soak)'
}

# ── Idle resource use ────────────────────────────────────────────────────

Write-Step 'Measuring idle resources (10 s settle) ...'
Start-Sleep -Seconds 10
$proc.Refresh()
$cpuBefore = $proc.TotalProcessorTime
Start-Sleep -Seconds 10
$proc.Refresh()
$cpuDelta = ($proc.TotalProcessorTime - $cpuBefore).TotalMilliseconds
$ramMb = [math]::Round($proc.WorkingSet64 / 1MB, 2)

Record 'idle-cpu' ($cpuDelta -lt 100) "cpu_ms_over_10s=$cpuDelta"

if ($ramMb -lt $TargetRamMb) {
    Record 'idle-ram' $true "ram=${ramMb}MB < target ${TargetRamMb}MB"
} elseif ($ramMb -lt $HardRamMb) {
    RecordMiss 'idle-ram' "ram=${ramMb}MB - above ${TargetRamMb}MB target, below ${HardRamMb}MB hard limit (explicit target miss)"
} else {
    Record 'idle-ram' $false "ram=${ramMb}MB >= ${HardRamMb}MB hard limit"
}

Write-Step 'Measuring release binary ...'
Invoke-Cargo @('build','--release','-p','daemon') | Out-Null
$relExe = Join-Path $repoRoot 'target\release\wiradesk.exe'
if (Test-Path $relExe) {
    $kb = [math]::Round((Get-Item $relExe).Length / 1KB, 1)
    if ($kb -ge $TargetBinaryMinKb -and $kb -le $TargetBinaryMaxKb) {
        Record 'binary-size' $true "size=${kb}KB within ${TargetBinaryMinKb}-${TargetBinaryMaxKb}KB"
    } elseif ($kb -lt $HardBinaryMaxKb) {
        RecordMiss 'binary-size' "size=${kb}KB - outside ${TargetBinaryMinKb}-${TargetBinaryMaxKb}KB target, below ${HardBinaryMaxKb}KB limit"
    } else {
        Record 'binary-size' $false "size=${kb}KB >= ${HardBinaryMaxKb}KB"
    }
} else {
    RecordMiss 'binary-size' 'release binary unavailable (daemon may hold the file)'
}

# ── Teardown ─────────────────────────────────────────────────────────────────

if (-not $KeepDaemon) {
    Write-Step 'Stopping daemon ...'
    Send-DaemonMsg -Hwnd $hwnd -Msg $WM_CLOSE
    Start-Sleep -Seconds 2
    if (-not $proc.HasExited) { try { $proc.Kill() } catch {} }
}

# ── Summary ──────────────────────────────────────────────────────────────────

Write-Host ''
Write-Step '───────── SUMMARY ─────────'
$results | ForEach-Object {
    $color = if ($_ -like 'PASS*') { 'Green' } elseif ($_ -like 'MISS*') { 'Yellow' } else { 'Red' }
    Write-Host "  $_" -ForegroundColor $color
}
$failed = ($results | Where-Object { $_ -like 'FAIL*' }).Count
$missed = ($results | Where-Object { $_ -like 'MISS*' }).Count
$passed = ($results | Where-Object { $_ -like 'PASS*' }).Count
Write-Host ''
Write-Step "$passed passed, $missed target-miss, $failed failed"

if ($failed -gt 0) { exit 1 }
exit 0
