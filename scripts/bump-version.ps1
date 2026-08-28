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
#
# `-Encoding UTF8` is not optional. Without it Windows PowerShell 5.1 reads the file as the
# system ANSI codepage, and every em dash in the comments above the version comes back as
# mojibake — which `Set-Content` then writes out as the corruption it read. The manifest's
# prose was silently mangled once this way before the encoding was pinned.
$lines = Get-Content -LiteralPath $manifest -Encoding UTF8
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

# Written through .NET rather than `Set-Content -Encoding utf8`, which in Windows PowerShell
# 5.1 means UTF-8 *with* a BOM. Cargo tolerates one, but it turns a one-line version change
# into a diff that also rewrites the first line of the file, and nothing here asked for that.
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($manifest, $lines, $utf8NoBom)
Write-Host "  Cargo.toml updated"

# Refreshes the workspace entries in `Cargo.lock` without touching dependency versions.
# `--offline` because this must not become an opportunity to pull new dependencies; a bump
# is not an update.
# No `2>&1` here, and that is the whole point. Cargo writes its progress to stderr; in Windows
# PowerShell 5.1 redirecting a native command's stderr wraps each line in an ErrorRecord, and
# with `$ErrorActionPreference = 'Stop'` the first such line terminates the script — after the
# manifest was written but before the lock was. The bump then looked like it had worked, and CI,
# which builds with `--locked`, is where you found out it had not.
Push-Location $root
try {
    cargo update --offline --workspace | Out-Null
}
finally {
    Pop-Location
}

# Verified against the file rather than against cargo's exit code. A lock left at the old
# version is the failure that matters, and it is cheap to look.
$members = 'daemon', 'settings', 'shared'
$lockPath = Join-Path $root 'Cargo.lock'
$lock = Get-Content -LiteralPath $lockPath -Encoding UTF8 -Raw
$stale = $members | Where-Object {
    $lock -notmatch ('(?m)^name = "{0}"\r?\nversion = "{1}"$' -f [regex]::Escape($_), [regex]::Escape($next))
}
if ($stale) {
    Write-Warning ("Cargo.lock still at the old version for: {0}" -f ($stale -join ', '))
    Write-Warning 'Run `cargo update --offline --workspace` by hand and commit the lock. CI builds'
    Write-Warning 'with --locked, so a stale lock fails the release rather than warning about it.'
}
else {
    Write-Host '  Cargo.lock refreshed'
}

Write-Host ''
Write-Host 'Next, and neither is optional:'
Write-Host "  1. Add a '## [$next]' section to CHANGELOG.md. The release workflow refuses a"
Write-Host '     tag without one, and the in-app updater shows that text to the user.'
Write-Host '  2. Commit Cargo.toml AND Cargo.lock together. CI builds with --locked.'
