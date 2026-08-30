#!/usr/bin/env pwsh
<#
.SYNOPSIS
Regenerates the winget manifest for one already-published release, from the GitHub release
itself, so the manifest can never disagree with what was actually shipped.

.DESCRIPTION
This is the piece `release.yml`'s `winget` job does NOT cover: that job (winget-releaser) only
updates an ALREADY-ACCEPTED package, and needs one manifest to already exist in
microsoft/winget-pkgs before it has anything to update. Getting that first manifest in — and
regenerating any later version by hand, if the automated job is ever unavailable — is what this
script is for.

Nothing here talks to microsoft/winget-pkgs directly. It only writes the three manifest files
under packaging/winget/manifests/, mirroring that repo's own directory layout, so the result can
be copied straight into a fork or handed to `wingetcreate submit`.

.PARAMETER Version
The already-released version to generate a manifest for, e.g. 0.1.4. There must be a matching
GitHub release tagged v<Version> with a WiraDesk-<Version>-x64-setup.exe asset.

.EXAMPLE
pwsh scripts/generate-winget-manifest.ps1 -Version 0.1.5
Writes packaging/winget/manifests/w/WiraDigitalIndonesia/WiraDesk/0.1.5/*.yaml, then prints the
`wingetcreate submit` command that actually opens the pull request.
#>
param(
    [Parameter(Mandatory)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'

$identifier = 'WiraDigitalIndonesia.WiraDesk'
$repo = 'wiradigitalid/wira-desk'
$tag = "v$Version"
$setupName = "WiraDesk-$Version-x64-setup.exe"

# Read from the release itself, never transcribed by hand: a manifest that disagrees with the
# artefact it names is worse than no manifest, because winget would install the wrong bytes.
$releaseJson = gh release view $tag --repo $repo --json assets
if ($LASTEXITCODE -ne 0) {
    throw "no GitHub release found for tag $tag in $repo - tag it and let release.yml publish first"
}
$release = $releaseJson | ConvertFrom-Json
$asset = $release.assets | Where-Object { $_.name -eq $setupName }
if (-not $asset) {
    throw "release $tag has no asset named $setupName"
}
if (-not $asset.digest -or $asset.digest -notmatch '^sha256:([0-9a-fA-F]{64})$') {
    throw "asset $setupName has no usable sha256 digest"
}
$sha256 = $Matches[1].ToUpper()
$url = $asset.url

$repoRoot = Split-Path -Parent $PSScriptRoot
$outDir = Join-Path $repoRoot "packaging\winget\manifests\w\WiraDigitalIndonesia\WiraDesk\$Version"
New-Item -ItemType Directory -Force $outDir | Out-Null

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.28.0.schema.json
PackageIdentifier: $identifier
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.28.0
"@ | Set-Content -Encoding utf8 (Join-Path $outDir "$identifier.yaml")

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.28.0.schema.json
PackageIdentifier: $identifier
PackageVersion: $Version
InstallerType: inno
Scope: machine
InstallModes:
  - silent
  - silentWithProgress
UpgradeBehavior: install
Installers:
  - Architecture: x64
    InstallerUrl: $url
    InstallerSha256: $sha256
ManifestType: installer
ManifestVersion: 1.28.0
"@ | Set-Content -Encoding utf8 (Join-Path $outDir "$identifier.installer.yaml")

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.28.0.schema.json
PackageIdentifier: $identifier
PackageVersion: $Version
PackageLocale: en-US
Publisher: Wira Digital Indonesia
PublisherUrl: https://github.com/wiradigitalid
PublisherSupportUrl: https://github.com/$repo/issues
PackageName: Wira Desk
PackageUrl: https://github.com/$repo
License: MIT
LicenseUrl: https://github.com/$repo/blob/main/LICENSE
PrivacyUrl: https://github.com/$repo/blob/main/PRIVACY.md
ShortDescription: Lightweight desktop tools for Windows - same-app window switching and window arrangement via a tray daemon.
Description: |-
  Wira Desk runs as an elevated system-tray daemon with a global low-level keyboard hook for
  same-app window switching and window arrangement, plus a Settings companion app for
  configuring shortcuts.

  The installer requires Administrator and installs to %ProgramFiles%, which is deliberate:
  auto-start runs the daemon elevated at every logon with no prompt, so a directory only
  administrators can write is what protects it. Auto-start is not switched on by the
  installer - that stays the user's choice from the tray menu or Settings.
Moniker: wiradesk
Tags:
  - window-manager
  - productivity
  - keyboard-shortcuts
  - tray
  - hotkeys
ManifestType: defaultLocale
ManifestVersion: 1.28.0
"@ | Set-Content -Encoding utf8 (Join-Path $outDir "$identifier.locale.en-US.yaml")

Write-Host "Wrote $outDir"
Write-Host ""
Write-Host "This is a LOCAL, in-repo copy kept as the source of truth for this manifest -"
Write-Host "it does not by itself reach microsoft/winget-pkgs. To actually submit it:"
Write-Host ""
Write-Host "  wingetcreate submit --prtitle `"New version: $identifier version $Version`" ``"
Write-Host "    --token `$env:WINGET_TOKEN $outDir"
Write-Host ""
Write-Host "wingetcreate (https://github.com/microsoft/winget-create) forks microsoft/winget-pkgs"
Write-Host "under your GitHub account if needed, commits these files at the matching path, and"
Write-Host "opens the pull request. See packaging/winget/README.md for the one-time setup and"
Write-Host "why this manual step exists at all even though later releases are automatic."
