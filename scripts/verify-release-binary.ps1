# scripts/verify-release-binary.ps1
#
# Release binary dependency scanner and regression guard.
# Inspects Windows PE import tables of release executables to verify that no
# dynamic CRT libraries (VCRUNTIME*.dll, MSVCP*.dll) are imported, or confirms
# that the documented VC++ Redistributable fallback bundling is in place.

[CmdletBinding()]
param(
    [string]$Path = 'target\release',
    [switch]$AllowDynamicCrtIfBundled
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$targetDir = if ([System.IO.Path]::IsPathRooted($Path)) { $Path } else { Join-Path $repoRoot $Path }

if (-not (Test-Path $targetDir)) {
    Write-Error "Target path not found: $targetDir"
    exit 1
}

$binaries = if ((Get-Item $targetDir) -is [System.IO.DirectoryInfo]) {
    Get-ChildItem -Path $targetDir -Filter "*.exe" -File
} else {
    @(Get-Item $targetDir)
}

if ($binaries.Count -eq 0) {
    Write-Error "No .exe binaries found in $targetDir"
    exit 1
}

function Get-PeImports {
    param([string]$FilePath)

    $bytes = [System.IO.File]::ReadAllBytes($FilePath)
    if ($bytes.Length -lt 0x40 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
        return @() # Not a valid PE
    }

    $e_lfanew = [BitConverter]::ToUInt32($bytes, 0x3C)
    if ($e_lfanew + 24 -gt $bytes.Length) { return @() }
    if ($bytes[$e_lfanew] -ne 0x50 -or $bytes[$e_lfanew + 1] -ne 0x45) { return @() }

    $fileHeader = $e_lfanew + 4
    $numSections = [BitConverter]::ToUInt16($bytes, $fileHeader + 2)
    $optHeaderSize = [BitConverter]::ToUInt16($bytes, $fileHeader + 16)
    $optHeader = $fileHeader + 20

    $magic = [BitConverter]::ToUInt16($bytes, $optHeader)
    $importDirOffset = if ($magic -eq 0x010B) { $optHeader + 104 } elseif ($magic -eq 0x020B) { $optHeader + 120 } else { return @() }

    $importRva = [BitConverter]::ToUInt32($bytes, $importDirOffset)
    $importSize = [BitConverter]::ToUInt32($bytes, $importDirOffset + 4)
    if ($importRva -eq 0 -or $importSize -eq 0) { return @() }

    $secTable = $optHeader + $optHeaderSize
    $sections = @()
    for ($i = 0; $i -lt $numSections; $i++) {
        $sec = $secTable + $i * 40
        $sections += [PSCustomObject]@{
            VirtualAddress   = [BitConverter]::ToUInt32($bytes, $sec + 12)
            VirtualSize      = [BitConverter]::ToUInt32($bytes, $sec + 8)
            SizeOfRawData    = [BitConverter]::ToUInt32($bytes, $sec + 16)
            PointerToRawData = [BitConverter]::ToUInt32($bytes, $sec + 20)
        }
    }

    function RvaToOffset($rva) {
        foreach ($s in $sections) {
            $sz = [Math]::Max($s.VirtualSize, $s.SizeOfRawData)
            if ($rva -ge $s.VirtualAddress -and $rva -lt ($s.VirtualAddress + $sz)) {
                $offset = $rva - $s.VirtualAddress
                if ($offset -lt $s.SizeOfRawData) {
                    return $s.PointerToRawData + $offset
                }
            }
        }
        return $null
    }

    $importOffset = RvaToOffset $importRva
    if ($null -eq $importOffset) { return @() }

    $dllNames = @()
    $currDesc = $importOffset
    while ($currDesc + 20 -le $bytes.Length) {
        $allZero = $true
        for ($k = 0; $k -lt 20; $k++) {
            if ($bytes[$currDesc + $k] -ne 0) { $allZero = $false; break }
        }
        if ($allZero) { break }

        $nameRva = [BitConverter]::ToUInt32($bytes, $currDesc + 12)
        $nameOffset = RvaToOffset $nameRva
        if ($null -ne $nameOffset -and $nameOffset -lt $bytes.Length) {
            $end = $nameOffset
            while ($end -lt $bytes.Length -and $bytes[$end] -ne 0) { $end++ }
            $dllName = [System.Text.Encoding]::ASCII.GetString($bytes, $nameOffset, $end - $nameOffset)
            if ($dllName) { $dllNames += $dllName }
        }
        $currDesc += 20
    }

    return $dllNames
}

$hasFailures = $false

foreach ($bin in $binaries) {
    Write-Host "[*] Scanning $($bin.Name) ..."
    $imports = Get-PeImports $bin.FullName
    $msvcImports = @($imports | Where-Object { $_ -like "VCRUNTIME*.dll" -or $_ -like "MSVCP*.dll" })

    if ($msvcImports.Count -gt 0) {
        if ($AllowDynamicCrtIfBundled) {
            Write-Host "    [!] Dynamic MSVC imports detected: $($msvcImports -join ', ')" -ForegroundColor Yellow
            Write-Host "    [*] Verifying Inno Setup fallback bundling in packaging\wiradesk.iss..."

            $issContent = Get-Content "packaging\wiradesk.iss" -Raw
            if ($issContent -match "vc_redist\.x64\.exe" -or $issContent -match "VCRedist") {
                Write-Host "    [+] Bundled VC++ fallback confirmed in packaging\wiradesk.iss." -ForegroundColor Green
            } else {
                Write-Error "    [x] Dynamic CRT detected in $($bin.Name) without installer bundling!"
                $hasFailures = $true
            }
        } else {
            Write-Error "Dynamic MSVC imports detected in $($bin.Name): $($msvcImports -join ', ')"
            $hasFailures = $true
        }
    } else {
        Write-Host "    [+] Clean: 0 dynamic MSVC CRT imports." -ForegroundColor Green
    }
}

if ($hasFailures) {
    exit 1
} else {
    Write-Host "[+] All release binary import checks passed successfully." -ForegroundColor Green
    exit 0
}
