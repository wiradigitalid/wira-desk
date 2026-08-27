<#
.SYNOPSIS
    Moves the product version, and refuses to move the digits an agent may not move.

.DESCRIPTION
    The version is written by hand in exactly one place: `[workspace.package] version` in the
    root `Cargo.toml`. The three crates inherit it, and each resource script receives it as
    preprocessor macros from its `build.rs`, so there is nothing else to edit and no duplicate
    to keep in step. This script exists to make that one edit correct rather than to hunt down
    eleven of them, which is what it used to take.

    `Cargo.lock` records the workspace crates' versions too. It is generated, so it is
    refreshed here rather than edited — and it MUST be committed alongside the manifest,
    because CI builds with `--locked` and a stale lock fails the release.

    WHY THIS SCRIPT REFUSES THINGS. `AGENTS.md`, under "Versioning authority", says an agent
    may move the patch digit and must not move minor or major. A rule in a document is the
    governing instrument; this is the speed bump that stops an absent-minded violation of it.
    It is not a security boundary — anyone can edit the manifest by hand — and it is not meant
    to be. It is meant to make the wrong bump require a deliberate flag, so it cannot happen by
    momentum.

    Below 1.0 the minor digit carries the breaking change, so `-Minor` is the incompatible step
    and `-Major` is essentially the single 0.x -> 1.0 event.

.PARAMETER Patch
    Increment the patch digit. Needs no authorisation.

.PARAMETER Minor
    Increment the minor digit and reset patch to zero. Requires -Owner.

.PARAMETER Major
    Increment the major digit and reset minor and patch to zero. Requires -Owner.

.PARAMETER Set
    Set an exact version, e.g. 0.4.0. Requires -Owner unless only the patch digit changes.

.PARAMETER Owner
    Confirms the owner asked for a minor or major change. See "Versioning authority".

.PARAMETER DryRun
    Report what would change and touch nothing.

.EXAMPLE
    ./scripts/bump-version.ps1 -Patch

.EXAMPLE
    ./scripts/bump-version.ps1 -Minor -Owner
#>
[CmdletBinding(DefaultParameterSetName = 'Patch')]
param(
    [Parameter(ParameterSetName = 'Patch')][switch]$Patch,
    [Parameter(ParameterSetName = 'Minor')][switch]$Minor,
    [Parameter(ParameterSetName = 'Major')][switch]$Major,
    [Parameter(ParameterSetName = 'Set', Mandatory = $true)][string]$Set,
    [switch]$Owner,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$manifest = Join-Path $root 'Cargo.toml'
if (-not (Test-Path -LiteralPath $manifest)) { throw "root Cargo.toml not found at $manifest" }

# Section-aware on purpose: the root manifest holds several tables, and a bare
# `version = "..."` under any other one is a different fact.
$lines = Get-Content -LiteralPath $manifest
$inWorkspacePackage = $false
$lineIndex = -1
$current = $null
for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^\s*\[([^\]]+)\]') {
        $inWorkspacePackage = ($Matches[1] -eq 'workspace.package')
        continue
    }
    if ($inWorkspacePackage -and $lines[$i] -match '^\s*version\s*=\s*"([0-9]+)\.([0-9]+)\.([0-9]+)"\s*$') {
        $current = [pscustomobject]@{
            Major = [int]$Matches[1]
            Minor = [int]$Matches[2]
            Patch = [int]$Matches[3]
        }
        $lineIndex = $i
        break
    }
}
if (-not $current) {
    throw 'could not read version from [workspace.package] in the root Cargo.toml'
}

$from = '{0}.{1}.{2}' -f $current.Major, $current.Minor, $current.Patch

switch ($PSCmdlet.ParameterSetName) {
    'Patch' { $next = '{0}.{1}.{2}' -f $current.Major, $current.Minor, ($current.Patch + 1); $digit = 'patch' }
    'Minor' { $next = '{0}.{1}.0'   -f $current.Major, ($current.Minor + 1);                 $digit = 'minor' }
    'Major' { $next = '{0}.0.0'     -f ($current.Major + 1);                                 $digit = 'major' }
    'Set' {
        if ($Set -notmatch '^([0-9]+)\.([0-9]+)\.([0-9]+)$') {
            throw "-Set expects three numeric fields, e.g. 0.4.0; got '$Set'"
        }
        $next = $Set
        $digit = if ([int]$Matches[1] -ne $current.Major) { 'major' }
                 elseif ([int]$Matches[2] -ne $current.Minor) { 'minor' }
                 elseif ([int]$Matches[3] -ne $current.Patch) { 'patch' }
                 else { 'nothing' }
    }
}

if ($digit -eq 'nothing') {
    Write-Host "already at $from; nothing to do"
    exit 0
}

if ($digit -ne 'patch' -and -not $Owner) {
    Write-Host ''
    Write-Host "REFUSED: $from -> $next moves the $digit digit." -ForegroundColor Red
    Write-Host ''
    Write-Host 'An agent may move the patch digit only. See AGENTS.md, "Versioning authority".'
    Write-Host 'A change needing a minor or major bump belongs under ## [Unreleased] in'
    Write-Host 'CHANGELOG.md, left there for the owner to decide.'
    Write-Host ''
    Write-Host 'If the owner asked for this, re-run with -Owner.'
    exit 2
}

Write-Host "$from -> $next ($digit)"

if ($DryRun) {
    Write-Host 'dry run: nothing written'
    exit 0
}

$lines[$lineIndex] = $lines[$lineIndex] -replace '"[0-9]+\.[0-9]+\.[0-9]+"', "`"$next`""
Set-Content -LiteralPath $manifest -Value $lines -Encoding utf8
Write-Host "  Cargo.toml updated"

# Refreshes the workspace entries in `Cargo.lock` without touching dependency versions.
# `--offline` because this must not become an opportunity to pull new dependencies; a bump
# is not an update.
Push-Location $root
try {
    cargo update --offline --workspace 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Warning 'cargo update --offline failed; run `cargo check` and commit Cargo.lock by hand'
    }
    else {
        Write-Host '  Cargo.lock refreshed'
    }
}
finally {
    Pop-Location
}

Write-Host ''
Write-Host 'Next, and neither is optional:'
Write-Host "  1. Add a '## [$next]' section to CHANGELOG.md. The release workflow refuses a"
Write-Host '     tag without one, and the in-app updater shows that text to the user.'
Write-Host '  2. Commit Cargo.toml AND Cargo.lock together. CI builds with --locked.'
