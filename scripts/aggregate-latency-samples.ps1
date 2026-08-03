<#
.SYNOPSIS
    Compute cycle-latency percentiles from durable `CYCLE_SAMPLE` trace lines.

.DESCRIPTION
    The daemon's in-process sample buffer dies with the process, so
    `WM_APP_DEBUG_DUMP_CYCLE_METRICS` only ever describes the current session.
    A machine that reboots regularly would therefore never accumulate a sample
    large enough to justify a latency threshold, and percentiles from separate
    sessions cannot legitimately be averaged together afterwards — only raw
    samples pool.

    Every completed cycle is therefore also written to the trace file as

        CYCLE_SAMPLE: ns=<nanoseconds> outcome=<activated|exhausted|no_target>

    which survives restarts. This script pools those lines across as many
    sessions as the log holds and reports the distribution.

    Defaults to `activated` only, deliberately. The threshold is about perceived latency
    when focus moves; a cycle that found no target costs roughly enumeration
    alone, and one whose candidates all failed activation pays two bounded polls
    per candidate. Pooling all three yields a percentile describing no real user
    experience. Every outcome is still counted and reported so a sample drawn
    from the wrong path is visible rather than silent.

    Requires a daemon built with metric seams compiled in — the
    `release-metrics` profile, or any debug build. A plain `--release` daemon
    writes no trace at all.

.EXAMPLE
    .\aggregate-latency-samples.ps1

.EXAMPLE
    .\aggregate-latency-samples.ps1 -Outcome all -Path C:\archive\old-trace.log
#>
[CmdletBinding()]
param(
    # Trace file(s). Accepts several so archived logs can be pooled.
    [string[]]$Path,
    [ValidateSet('activated', 'exhausted', 'no_target', 'all')]
    [string]$Outcome = 'activated',
    # Warn when the sample is too small for the reported p95 to mean much.
    [int]$MinSamples = 200
)

$ErrorActionPreference = 'Stop'

if (-not $Path) {
    $Path = @(Join-Path $env:APPDATA 'WiraDesk\wiradesk-debug-trace.log')
}

function Write-Head($m) { Write-Host $m -ForegroundColor Cyan }
function Write-Warn2($m) { Write-Host $m -ForegroundColor Yellow }

$missing = $Path | Where-Object { -not (Test-Path $_) }
if ($missing) {
    Write-Warn2 "Not found, skipped:"
    $missing | ForEach-Object { Write-Warn2 "  $_" }
}
$present = $Path | Where-Object { Test-Path $_ }
if (-not $present) { Write-Host "No trace file to read." -ForegroundColor Red; exit 1 }

$byOutcome = @{ activated = @(); exhausted = @(); no_target = @() }

foreach ($file in $present) {
    foreach ($m in (Select-String -Path $file -Pattern 'CYCLE_SAMPLE: ns=(\d+) outcome=(\w+)')) {
        $ns = [int64]$m.Matches[0].Groups[1].Value
        $oc = $m.Matches[0].Groups[2].Value
        if ($byOutcome.ContainsKey($oc)) { $byOutcome[$oc] += $ns }
    }
}

$total = ($byOutcome.Values | ForEach-Object { $_.Count } | Measure-Object -Sum).Sum
if ($total -eq 0) {
    Write-Host "No CYCLE_SAMPLE lines found." -ForegroundColor Red
    Write-Host "The daemon must be built with metric seams (release-metrics or debug)." -ForegroundColor Red
    exit 1
}

# Nearest-rank, matching `metrics.rs::percentile` exactly: a reported percentile
# is always an observed sample, never a synthesized value that never occurred.
function Get-Percentile {
    param([int64[]]$Sorted, [double]$P)
    if ($Sorted.Count -eq 0) { return 0 }
    $rank = [math]::Ceiling($P * $Sorted.Count)
    $idx = [math]::Min([math]::Max($rank - 1, 0), $Sorted.Count - 1)
    return $Sorted[$idx]
}

Write-Host ""
Write-Head "--------- SAMPLE POOL ---------"
foreach ($f in $present) { Write-Host "  source: $f" }
Write-Host ""
foreach ($k in @('activated', 'exhausted', 'no_target')) {
    Write-Host ("  {0,-10} {1,7} samples" -f $k, $byOutcome[$k].Count)
}
Write-Host ("  {0,-10} {1,7}" -f 'TOTAL', $total)

$selected = if ($Outcome -eq 'all') {
    @($byOutcome['activated']) + @($byOutcome['exhausted']) + @($byOutcome['no_target'])
} else {
    @($byOutcome[$Outcome])
}

Write-Host ""
Write-Head "--------- Cycle latency ($Outcome) ---------"

if ($selected.Count -eq 0) {
    Write-Warn2 "  No samples for outcome '$Outcome'."
    if ($Outcome -eq 'activated') {
        Write-Warn2 "  Focus never moved in any recorded cycle. Either the daemon was driven"
        Write-Warn2 "  by a harness that bypasses the keyboard hook, or cycling had no target."
    }
    exit 1
}

$sorted = [int64[]]($selected | Sort-Object)
$p50 = Get-Percentile $sorted 0.50
$p95 = Get-Percentile $sorted 0.95
$p99 = Get-Percentile $sorted 0.99
$max = $sorted[-1]
$min = $sorted[0]

Write-Host ("  samples  = {0}" -f $sorted.Count)
Write-Host ("  min      = {0:N3} ms" -f ($min / 1e6))
Write-Host ("  p50      = {0:N3} ms" -f ($p50 / 1e6))
Write-Host ("  p95      = {0:N3} ms" -f ($p95 / 1e6))
Write-Host ("  p99      = {0:N3} ms" -f ($p99 / 1e6))
Write-Host ("  max      = {0:N3} ms" -f ($max / 1e6))
Write-Host ""

if ($sorted.Count -lt $MinSamples) {
    Write-Warn2 ("  Only {0} samples. At this size the reported p95 is one of the largest few" -f $sorted.Count)
    Write-Warn2 ("  observations, not a stable estimate. Collect at least {0} before setting" -f $MinSamples)
    Write-Warn2  "  a threshold from it."
} else {
    Write-Host "  Sample size is adequate for a p95 estimate." -ForegroundColor Green
}

# Explicit, so the caller reads this run's result rather than whatever
# `$LASTEXITCODE` happened to hold from an earlier command.
exit 0
