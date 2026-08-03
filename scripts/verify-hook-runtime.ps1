<#
.SYNOPSIS
    Elevated runtime verification for the Wira Desk keyboard hook (debug trace seam).

.DESCRIPTION
    Requires Administrator (same as wiradesk.exe). Builds the debug daemon, starts a
    single instance, drives debug-only WM_APP messages, and asserts on
    wiradesk-debug-trace.log (lifecycle, QPC latency, simulated shortcut/worker drain).

.EXAMPLE
    .\verify-hook-runtime.ps1

.EXAMPLE
    .\verify-hook-runtime.ps1 -SkipBuild -KeepDaemon
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$SkipUnitTests,
    [switch]$KeepDaemon,
    [int]$MaxLatencyUs = 10000
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $repoRoot

$WM_APP = 0x8000
$WM_APP_HOOK_CHECK = $WM_APP + 5
$WM_APP_DEBUG_TOGGLE_HOOK_FAIL = $WM_APP + 20
$WM_APP_DEBUG_TRIGGER_WARN = $WM_APP + 21
$WM_APP_DEBUG_HOOK_CHECK = $WM_APP + 22
$WM_APP_DEBUG_DUMP_HOOK_LATENCY = $WM_APP + 23
$WM_APP_DEBUG_SIMULATE_SHORTCUT = $WM_APP + 24
$WM_CLOSE = 0x0010

$THREAD_PRIORITY_TIME_CRITICAL = 15
$THREAD_QUERY_INFORMATION = 0x0040
$WindowClass = 'WiraDeskDaemonHiddenWindow'
$WindowTitle = 'WiraDeskDaemon'

function Write-Step($msg) { Write-Host "[verify-hook] $msg" -ForegroundColor Cyan }
function Write-Pass($msg) { Write-Host "[verify-hook] PASS: $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "[verify-hook] FAIL: $msg" -ForegroundColor Red }

function Test-IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $p = [Security.Principal.WindowsPrincipal]$id
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-IsAdmin)) {
    Write-Fail 'Administrator shell required (Wira Desk manifest + daemon tests).'
    exit 1
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WiraDeskWin32 {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessageW(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern IntPtr OpenThread(uint dwDesiredAccess, bool bInheritHandle, uint dwThreadId);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool CloseHandle(IntPtr hObject);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern int GetThreadPriority(IntPtr hThread);
}
"@

function Get-TracePath {
    Join-Path (Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'WiraDesk') 'wiradesk-debug-trace.log'
}

function Get-LogPath {
    Join-Path (Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'WiraDesk') 'wiradesk.log'
}

function Wait-DaemonWindow {
    param([int]$TimeoutSec = 20)
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        $h = [WiraDeskWin32]::FindWindowW($WindowClass, $WindowTitle)
        if ($h -ne [IntPtr]::Zero) { return $h }
        Start-Sleep -Milliseconds 200
    }
    return [IntPtr]::Zero
}

function Send-DaemonMsg {
    param(
        [IntPtr]$Hwnd,
        [uint32]$Msg,
        [uint64]$WParam = 0
    )
    if (-not [WiraDeskWin32]::PostMessageW($Hwnd, $Msg, [UIntPtr]$WParam, [IntPtr]::Zero)) {
        throw "PostMessageW 0x{0:X} failed (err=$([Runtime.InteropServices.Marshal]::GetLastWin32Error()))" -f $Msg
    }
}

function Get-ThreadPriorityById {
    param([uint32]$ThreadId)
    $h = [WiraDeskWin32]::OpenThread($THREAD_QUERY_INFORMATION, $false, $ThreadId)
    if ($h -eq [IntPtr]::Zero) { return $null }
    try { return [WiraDeskWin32]::GetThreadPriority($h) }
    finally { [WiraDeskWin32]::CloseHandle($h) | Out-Null }
}

function Wait-TraceMatch {
    param([string]$Pattern, [int]$TimeoutSec = 15)
    $path = Get-TracePath
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path $path) {
            $text = Get-Content -Path $path -Raw -ErrorAction SilentlyContinue
            if ($text -match $Pattern) { return $true }
        }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Get-TraceText { Get-Content -Path (Get-TracePath) -Raw -ErrorAction SilentlyContinue }

function Start-DaemonProcess {
    param([string]$ExePath)
    try {
        return Start-Process -FilePath $ExePath -PassThru -WindowStyle Hidden
    } catch {
        Write-Step "Start-Process blocked ($($_.Exception.Message)); trying Process.Start fallback ..."
        $p = [System.Diagnostics.Process]::Start($ExePath)
        if (-not $p) { throw 'Process.Start returned null' }
        return $p
    }
}

$results = [System.Collections.Generic.List[string]]::new()
function Record($name, [bool]$ok, [string]$detail = '') {
    if ($ok) {
        Write-Pass "$name $detail"
        $results.Add("PASS $name")
    } else {
        Write-Fail "$name $detail"
        $results.Add("FAIL $name")
    }
}

Write-Step 'Checking toolchain...'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Fail 'cargo not found on PATH.'
    exit 1
}

if (-not $SkipUnitTests) {
    Write-Step 'cargo test -p daemon ...'
    & cargo test -p daemon
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not $SkipBuild) {
    Write-Step 'cargo build -p daemon (debug) ...'
    & cargo build -p daemon
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$exe = Join-Path $repoRoot 'target\debug\wiradesk.exe'
if (-not (Test-Path $exe)) {
    Write-Fail "Missing $exe"
    exit 1
}

Write-Step 'Stopping existing wiradesk instances ...'
Get-Process -Name 'wiradesk' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

$tracePath = Get-TracePath
if (Test-Path $tracePath) { Remove-Item $tracePath -Force }

Write-Step "Starting $exe ..."
$proc = Start-DaemonProcess -ExePath $exe
Start-Sleep -Seconds 1

$hwnd = Wait-DaemonWindow
Record 'daemon_hidden_window' ($hwnd -ne [IntPtr]::Zero) '(FindWindowW class)'
if ($hwnd -eq [IntPtr]::Zero) {
    if (-not $KeepDaemon) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
    exit 1
}

Record 'hook_ready_trace' (Wait-TraceMatch 'HOOK_READY:\s*tid=(\d+)') '(trace file)'

$traceText = Get-TraceText
if ($traceText -match 'HOOK_READY:\s*tid=(\d+)') {
    $hookTid = [uint32]$Matches[1]
    $prio = Get-ThreadPriorityById -ThreadId $hookTid
    Record 'hook_thread_priority' ($prio -eq $THREAD_PRIORITY_TIME_CRITICAL) "(tid=$hookTid priority=$prio)"
} else {
    Record 'hook_thread_priority' $false '(no tid in trace)'
}

Write-Step 'Simulate primary shortcut (Win+Backtick) ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_SIMULATE_SHORTCUT -WParam 0
Start-Sleep -Milliseconds 500
Record 'sim_primary_shortcut' (Wait-TraceMatch 'SIM_SHORTCUT: scenario=primary enqueued=1 swallow_main_down=1 win_up_passed=1')
Record 'worker_drain_cycle' (Wait-TraceMatch 'WORKER_DRAIN: cycle=1')

Write-Step 'Simulate extra-modifier pass-through ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_SIMULATE_SHORTCUT -WParam 1
Start-Sleep -Milliseconds 300
Record 'sim_extra_modifier' (Wait-TraceMatch 'SIM_SHORTCUT: scenario=extra_mod pass=1')

Write-Step 'Dump QPC hook-path latency ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_DUMP_HOOK_LATENCY -WParam 0
Start-Sleep -Milliseconds 300
$latencyOk = $false
$latencyDetail = ''
if ((Get-TraceText) -match 'HOOK_LATENCY: max_us=(\d+) samples=(\d+)') {
    $maxUs = [int64]$Matches[1]
    $samples = [int64]$Matches[2]
    $latencyOk = ($samples -gt 0) -and ($maxUs -lt $MaxLatencyUs)
    $latencyDetail = "(max_us=$maxUs samples=$samples limit=$MaxLatencyUs)"
} else {
    $latencyDetail = '(HOOK_LATENCY line missing)'
}
Record 'hook_latency_qpc' $latencyOk $latencyDetail

Write-Step 'Tier-2 debug warn ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_TRIGGER_WARN
Start-Sleep -Milliseconds 400
$logPath = Get-LogPath
$logOk = (Test-Path $logPath) -and ((Get-Content $logPath -Tail 8 -ErrorAction SilentlyContinue) -match 'debug: simulated Tier-2')
Record 'tier2_log_warn' $logOk

Write-Step 'Tier-3 escalation ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_TOGGLE_HOOK_FAIL
Start-Sleep -Milliseconds 200
foreach ($i in 1..3) {
    Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_HOOK_CHECK
    Start-Sleep -Milliseconds 400
}
Record 'tier3_trace' (Wait-TraceMatch 'TIER3:\s*state')

Write-Step 'Recovery ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_TOGGLE_HOOK_FAIL
Start-Sleep -Milliseconds 200
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_APP_DEBUG_HOOK_CHECK
Record 'recovery_trace' (Wait-TraceMatch 'RECOVERY:')

Write-Step 'Clean shutdown ...'
Send-DaemonMsg -Hwnd $hwnd -Msg $WM_CLOSE
$exited = $proc.WaitForExit(8000)
if (-not $exited) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    $exited = $true
}
Record 'clean_shutdown' $exited

Write-Host ''
Write-Host '=== Summary ===' -ForegroundColor Yellow
$results | ForEach-Object { Write-Host $_ }
$failCount = ($results | Where-Object { $_ -like 'FAIL *' }).Count
if ($failCount -gt 0) {
    Write-Fail "$failCount check(s) failed."
    exit 1
}
Write-Pass 'All automated hook runtime checks passed.'
exit 0
