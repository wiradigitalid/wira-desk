# scripts/restart-daemon.ps1
#
# Process lifecycle helper for Wira Desk daemon.
# Terminates any stale wiradesk.exe processes, validates the newest compiled binary,
# launches the daemon elevated, and polls for WiraDeskDaemonHiddenWindow readiness.

[CmdletBinding()]
param(
    [ValidateSet('debug', 'release')]
    [string]$Profile = 'debug'
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot

Write-Host "[*] Stopping any running wiradesk instances..."
Get-Process -Name "wiradesk" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "    Terminating process $($_.Id)..."
    try { $_.Kill(); $_.WaitForExit(3000) } catch {}
}
Start-Sleep -Milliseconds 300

$exePath = Join-Path $repoRoot "target\$Profile\wiradesk.exe"
if (-not (Test-Path $exePath)) {
    # Fallback to other profile if preferred is missing
    $other = if ($Profile -eq 'debug') { 'release' } else { 'debug' }
    $fallbackPath = Join-Path $repoRoot "target\$other\wiradesk.exe"
    if (Test-Path $fallbackPath) {
        $exePath = $fallbackPath
    } else {
        throw "No compiled daemon executable found at $exePath. Run 'cargo build' first."
    }
}

$fileInfo = Get-Item $exePath
Write-Host "[*] Launching latest daemon binary ($($fileInfo.FullName), last written: $($fileInfo.LastWriteTime))..."

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $exePath
$startInfo.WorkingDirectory = $repoRoot
$startInfo.Verb = "RunAs" # Elevated
$startInfo.UseShellExecute = $true

try {
    $proc = [System.Diagnostics.Process]::Start($startInfo)
    Write-Host "[+] Daemon process started with PID: $($proc.Id)"
} catch {
    Write-Warning "Failed to start elevated daemon: $_"
    exit 1
}

# Try P/Invoke FindWindowW for message window detection
$canFindWindow = $false
try {
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class WiraDeskDaemonFinder {
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);
}
"@ -ErrorAction Stop
    $canFindWindow = $true
} catch {
    # Type might already be defined in session
    if ([System.Management.Automation.PSTypeName]'WiraDeskDaemonFinder'.Type) {
        $canFindWindow = $true
    }
}

$WindowClass = 'WiraDeskDaemonHiddenWindow'
$WindowTitle = 'WiraDeskDaemon'

$timeoutSeconds = 6
$elapsed = 0.0
$ready = $false

while ($elapsed -lt $timeoutSeconds) {
    Start-Sleep -Milliseconds 400
    $elapsed += 0.4

    if ($canFindWindow) {
        $hwnd = [WiraDeskDaemonFinder]::FindWindowW($WindowClass, $WindowTitle)
        if ($hwnd -ne [IntPtr]::Zero) {
            $ready = $true
            break
        }
    } else {
        $active = Get-Process -Name "wiradesk" -ErrorAction SilentlyContinue
        if ($active) {
            $ready = $true
            break
        }
    }
}

if ($ready) {
    Write-Host "[+] Daemon message window ($WindowClass) is active and ready for reload signals."
} else {
    Write-Warning "Daemon message window did not report ready within $timeoutSeconds seconds."
    exit 1
}
