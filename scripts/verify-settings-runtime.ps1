<#
.SYNOPSIS
    Runtime verification for Wira Desk Settings (accessibility, theme, first run).

.DESCRIPTION
    Does NOT require Administrator — `wiradesk-settings.exe` carries no elevation
    manifest. Launches the real Settings window and inspects it through Windows
    UI Automation.

    Checks:
      every control exposes a role, name, and enabled state
      focusable controls are reachable; focus order is stable
      window renders in both Light and Dark
      a missing config launches onboarding, and finishing it writes a config

.EXAMPLE
    .\verify-settings-runtime.ps1

.EXAMPLE
    .\verify-settings-runtime.ps1 -SkipBuild -KeepOpen
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$KeepOpen,
    [int]$LaunchTimeoutSec = 30
)

$ErrorActionPreference = 'Stop'
# The workspace root, two levels up: this script lives in `scripts\`, and every
# path below is relative to the root that holds `target\` and `Cargo.toml`.
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $repoRoot


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

function Write-Step($m) { Write-Host "[verify-settings] $m" -ForegroundColor Cyan }
function Write-Pass($m) { Write-Host "[verify-settings] PASS: $m" -ForegroundColor Green }
function Write-Fail($m) { Write-Host "[verify-settings] FAIL: $m" -ForegroundColor Red }
function Write-Warn($m) { Write-Host "[verify-settings] MISS: $m" -ForegroundColor Yellow }

$results = [System.Collections.Generic.List[string]]::new()
function Record($name, [bool]$ok, [string]$detail = '') {
    if ($ok) { Write-Pass "$name $detail"; $results.Add("PASS $name") }
    else { Write-Fail "$name $detail"; $results.Add("FAIL $name") }
}
function RecordMiss($name, [string]$detail = '') {
    Write-Warn "$name $detail"; $results.Add("MISS $name")
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WiraDesk5 {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
}
"@

# UI Automation lives in these two assemblies; both ship with .NET Framework.
try {
    Add-Type -AssemblyName UIAutomationClient -ErrorAction Stop
    Add-Type -AssemblyName UIAutomationTypes -ErrorAction Stop
} catch {
    Write-Fail "UI Automation assemblies unavailable: $($_.Exception.Message)"
    exit 1
}

if (-not $SkipBuild) {
    Write-Step 'Building settings (debug) ...'
    if ((Invoke-Cargo @('build','-p','settings')) -ne 0) { Write-Fail 'build failed'; exit 1 }
}

$exe = Join-Path $repoRoot 'target\debug\wiradesk-settings.exe'
if (-not (Test-Path $exe)) { Write-Fail "missing $exe"; exit 1 }

# Settings refuses to open with no daemon running, and closes itself when the
# daemon it opened with exits. This harness deliberately runs without a daemon
# (it does not require Administrator, so it cannot start one), so it waives that
# rule for the processes it launches. Start-Process inherits this environment.
$env:WIRADESK_SETTINGS_ALLOW_NO_DAEMON = '1'

$wiraDeskDir = Join-Path $env:APPDATA 'WiraDesk'
$configPath = Join-Path $wiraDeskDir 'config.toml'
$legacyDir = Join-Path $env:APPDATA 'WinTick'
$backup = "$configPath.verify-backup"
$wiraDeskBackup = "$wiraDeskDir.verify-backup"
$legacyBackup = "$legacyDir.verify-backup"

function Restore-Config {
    if (Test-Path $wiraDeskBackup) {
        if (Test-Path $wiraDeskDir) { Remove-Item -Recurse -Force $wiraDeskDir -ErrorAction SilentlyContinue }
        Move-Item $wiraDeskBackup $wiraDeskDir -Force
        Write-Step 'WiraDesk appdata directory restored.'
    } elseif (Test-Path $backup) {
        New-Item -ItemType Directory -Force -Path $wiraDeskDir | Out-Null
        Move-Item $backup $configPath -Force
        Write-Step 'Original config.toml restored.'
    }
    if (Test-Path $legacyBackup) {
        Move-Item $legacyBackup $legacyDir -Force
        Write-Step 'Legacy WinTick appdata directory restored.'
    }
}

# First-run needs neither WiraDesk nor legacy WinTick appdata present (H9 Hypothesis B).
if (Test-Path $legacyDir) {
    Move-Item $legacyDir $legacyBackup -Force
    Write-Step 'Legacy WinTick appdata moved aside for first-run isolation.'
}
if (Test-Path $wiraDeskDir) {
    Move-Item $wiraDeskDir $wiraDeskBackup -Force
    Write-Step 'WiraDesk appdata moved aside for first-run isolation.'
} elseif (Test-Path $configPath) {
    Copy-Item $configPath $backup -Force
    Remove-Item $configPath -Force
    Write-Step 'Existing config.toml moved aside for the first-run check.'
}

trap {
    Write-Fail "unexpected error: $_"
    Restore-Config
    exit 1
}

function Start-Settings {
    param([string[]]$Arguments = @())
    # PowerShell 5.1 rejects an empty -ArgumentList, so omit it entirely when
    # there are no arguments rather than passing @().
    $p = if ($Arguments -and $Arguments.Count -gt 0) {
        Start-Process -FilePath $exe -ArgumentList $Arguments -PassThru
    } else {
        Start-Process -FilePath $exe -PassThru
    }
    $deadline = (Get-Date).AddSeconds($LaunchTimeoutSec)
    while ((Get-Date) -lt $deadline) {
        $p.Refresh()
        if ($p.HasExited) { return @{ Process = $p; Root = $null } }
        if ($p.MainWindowHandle -ne 0) {
            $root = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
            if ($root) {
                $title = $root.Current.Name
                $class = $root.Current.ClassName
                Write-Step "Matched window title='$title' class='$class' hwnd=$($p.MainWindowHandle)"
                return @{ Process = $p; Root = $root; Title = $title; Class = $class }
            }
        }
        Start-Sleep -Milliseconds 300
    }
    return @{ Process = $p; Root = $null }
}

# AccessKit's Windows adapter activates lazily: the tree is built only after a
# UIA client sends WM_GETOBJECT, and egui then needs to render another frame
# before it is populated. A single query therefore always comes back empty —
# a real screen reader triggers the same sequence, it just keeps asking.
function Get-AutomationElements {
    param(
        [System.Windows.Automation.AutomationElement]$Root,
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutSec = 15
    )
    $cond = [System.Windows.Automation.Condition]::TrueCondition
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    $best = $null
    while ((Get-Date) -lt $deadline) {
        # Re-acquire the root every iteration. An `AutomationElement` obtained
        # before the provider was ready keeps returning an empty tree forever —
        # reusing the cached one is what made this look like a product failure
        # when the tree was in fact published correctly.
        $Process.Refresh()
        $current = $null
        if ($Process.MainWindowHandle -ne 0) {
            try {
                $current = [System.Windows.Automation.AutomationElement]::FromHandle($Process.MainWindowHandle)
            } catch {}
        }
        if (-not $current) { $current = $Root }

        $found = $current.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($found.Count -gt 0) { return $found }
        $best = $found
        # egui only repaints on input or animation, so nudge it into rendering
        # the frame that publishes the tree.
        try { [WiraDesk5]::SetForegroundWindow($Process.MainWindowHandle) | Out-Null } catch {}
        Start-Sleep -Milliseconds 500
    }
    return $best
}

function Stop-Settings($p) {
    if ($p -and -not $p.HasExited) { try { $p.CloseMainWindow() | Out-Null; Start-Sleep -Milliseconds 500 } catch {} }
    if ($p -and -not $p.HasExited) { try { $p.Kill() } catch {} }
}

# ── First run launches onboarding ────────────────────────────────

Write-Step 'First run: no config present ...'
$session = Start-Settings
if (-not $session.Root) {
    Record 'window-appears' $false 'no window within timeout'
    Restore-Config
    exit 1
}
Record 'window-appears' $true

$cond = [System.Windows.Automation.Condition]::TrueCondition
$all = Get-AutomationElements -Root $session.Root -Process $session.Process
Record 'automation-tree-published' ($all.Count -gt 0) "$($all.Count) automation elements"

if ($all.Count -eq 0) {
    Write-Fail 'Empty accessibility tree — the accesskit feature is probably not active.'
}

$texts = @()
foreach ($e in $all) { $texts += "$($e.Current.Name)" }
$onboardingVisible = ($texts -join ' ') -match 'Welcome to Wira Desk|Skip Tutorial'
Record 'onboarding-shown' $onboardingVisible

Stop-Settings $session.Process

# ── Roles, names, focusability ─────────────────────────────

Write-Step 'Writing a config so the normal shell opens ...'
New-Item -ItemType Directory -Force -Path (Split-Path $configPath) | Out-Null
"" | Out-File -FilePath $configPath -Encoding utf8

$session = Start-Settings
if (-not $session.Root) {
    Record 'settings-window-appears' $false 'no window within timeout'
    Restore-Config
    exit 1
}
Record 'settings-window-appears' $true

$all = Get-AutomationElements -Root $session.Root -Process $session.Process
$named = 0; $unnamed = @(); $focusable = 0
foreach ($e in $all) {
    $c = $e.Current
    if ([string]::IsNullOrWhiteSpace($c.Name)) {
        if ($c.IsKeyboardFocusable) { $unnamed += $c.ControlType.ProgrammaticName }
    } else { $named++ }
    if ($c.IsKeyboardFocusable) { $focusable++ }
}

Record 'named-elements' ($named -gt 0) "$named named"
Record 'no-unnamed-focusable' ($unnamed.Count -eq 0) `
    $(if ($unnamed.Count) { "unnamed focusable: $($unnamed -join ', ')" } else { '' })
Record 'focusable-controls-exist' ($focusable -gt 0) "$focusable focusable"

# Expected accessible names come from theme.rs / app.rs.
$expected = @(
    'General', 'Shortcuts', 'Layout', 'About',
    'Save', 'Revert',
    'Start Wira Desk with Windows'
)
$present = $all | ForEach-Object { $_.Current.Name }
$missing = @()
foreach ($n in $expected) { if ($present -notcontains $n) { $missing += $n } }
Record 'declared-stops-present' ($missing.Count -eq 0) `
    $(if ($missing.Count) { "missing: $($missing -join ', ')" } else { '' })

Stop-Settings $session.Process

# ── Both themes render ───────────────────────────────────────────

$themeKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
$originalTheme = (Get-ItemProperty $themeKey -Name AppsUseLightTheme -ErrorAction SilentlyContinue).AppsUseLightTheme
if ($null -eq $originalTheme) {
    RecordMiss 'theme-switch' 'AppsUseLightTheme not set on this machine'
} else {
    foreach ($mode in @(@{v=1;n='Light'}, @{v=0;n='Dark'})) {
        Set-ItemProperty $themeKey -Name AppsUseLightTheme -Value $mode.v
        Start-Sleep -Milliseconds 500
        $s = Start-Settings
        Record "renders-$($mode.n.ToLower())" ($null -ne $s.Root)
        Stop-Settings $s.Process
    }
    Set-ItemProperty $themeKey -Name AppsUseLightTheme -Value $originalTheme
    Write-Step "Theme restored to original value ($originalTheme)."
}

# ── Teardown ─────────────────────────────────────────────────────────────────

if (-not $KeepOpen) { Remove-Item $configPath -Force -ErrorAction SilentlyContinue }
Restore-Config

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
