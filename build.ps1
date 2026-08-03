<#
.SYNOPSIS
    Wira Desk build automation.

.DESCRIPTION
    Compiles the Wira Desk Cargo workspace (wiradesk.exe + wiradesk-settings.exe).

    -Mode dev   : debug build with local logging enabled (cargo build).
    -Mode prod  : release build with the aggressive size profile from Cargo.toml
                  (lto=true, opt-level="z", strip=true, panic="abort"); target
                  binary size < 500KB, headless (no console window).

    The script:
      1. Verifies the Rust toolchain (cargo) is available.
      2. Loads the MSVC build environment (vcvars64) so rustc's msvc target can
         find link.exe and the Windows SDK — required on machines where cargo is
         not launched from a Developer Command Prompt.
      3. Builds the workspace and reports the resulting binary sizes.

.EXAMPLE
    .\build.ps1 -Mode dev
    .\build.ps1 -Mode prod
#>
[CmdletBinding()]
param(
    [ValidateSet('dev', 'prod')]
    [string]$Mode = 'dev'
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

function Write-Step($msg) { Write-Host "[build] $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "[build] $msg" -ForegroundColor Green }
function Write-Err($msg)  { Write-Host "[build] $msg" -ForegroundColor Red }

# ── 1. Dependency check: Rust toolchain ────────────────────────────────────
Write-Step 'Checking Rust toolchain (cargo)...'
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-Err 'cargo not found on PATH. Install Rust from https://rustup.rs and retry.'
    exit 1
}
Write-Ok "cargo found: $((& cargo --version))"

# ── 2. Load MSVC environment (vcvars64) if link.exe is not already on PATH ──
if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    Write-Step 'MSVC linker not on PATH; locating vcvars64.bat...'
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $vcvars = $null
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) { $vcvars = Join-Path $vsPath 'VC\Auxiliary\Build\vcvars64.bat' }
    }
    if (-not ($vcvars -and (Test-Path $vcvars))) {
        # Fallback to well-known BuildTools / Community locations.
        $candidates = @(
            'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat',
            'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat'
        )
        $vcvars = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    }
    if (-not ($vcvars -and (Test-Path $vcvars))) {
        Write-Err 'Could not locate vcvars64.bat. Install VS Build Tools with the C++ workload (Microsoft.VisualStudio.Workload.VCTools).'
        exit 1
    }
    Write-Step "Importing MSVC environment from: $vcvars"
    # Run vcvars64 in a child cmd and import the resulting environment.
    $envDump = & cmd /c "`"$vcvars`" >nul 2>&1 && set"
    foreach ($line in $envDump) {
        if ($line -match '^([^=]+)=(.*)$') {
            Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
        }
    }
    if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
        Write-Err 'vcvars64 loaded but link.exe still not found. The C++ toolset may be incomplete.'
        exit 1
    }
    Write-Ok 'MSVC environment loaded.'
} else {
    Write-Ok 'MSVC linker already on PATH.'
}

# ── 3. Build ────────────────────────────────────────────────────────────────
Push-Location $repoRoot
try {
    if ($Mode -eq 'prod') {
        Write-Step 'Building PRODUCTION (release, size-optimized)...'
        & cargo build --release
        $targetDir = Join-Path $repoRoot 'target\release'
    } else {
        Write-Step 'Building DEV (debug, logging enabled)...'
        & cargo build
        $targetDir = Join-Path $repoRoot 'target\debug'
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Err "cargo build failed (exit $LASTEXITCODE)."
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

# ── 4. Report binary sizes ──────────────────────────────────────────────────
Write-Ok 'Build succeeded. Artifacts:'
foreach ($bin in @('wiradesk.exe', 'wiradesk-settings.exe')) {
    $path = Join-Path $targetDir $bin
    if (Test-Path $path) {
        $kb = [math]::Round((Get-Item $path).Length / 1KB, 1)
        Write-Host ("  {0,-24} {1,8} KB" -f $bin, $kb)
    }
}
if ($Mode -eq 'prod') {
    Write-Host '[build] Size target: wiradesk.exe under 500 KB.' -ForegroundColor Yellow
}
