<#
.SYNOPSIS
    Verifies that a public export tree is fit to publish. Fails closed.

.DESCRIPTION
    One executable gate, replacing the copy-paste command blocks that used to live in two
    prose documents. Those drifted: every time the patterns were widened by hand they
    immediately found something that had been shipping for months, which is the signature of
    coverage nobody owned.

    This script asserts. It never rewrites. The export used to run a set of regex
    substitutions over the copied files, and that model was wrong in a way worth recording:
    the substitutions removed nothing in practice (the private sources of every exported file
    were already clean), while one of them corrupted a comment line in published source by
    matching a prefix and leaving the tail attached to its replacement. A mechanism that has
    never detected anything and has damaged output once is worse than no mechanism. Exported
    files are authored publication-clean; this verifies that claim instead of pretending to
    repair it.

    Exit code 0 means the tree may be published. Non-zero means it must not be.

.PARAMETER Path
    Root of the export tree, or of a public repository checkout.

.PARAMETER SkipHistory
    Skip the single-commit check. For running against a public repository that has
    legitimately accumulated history since its initial release.

.EXAMPLE
    .\verify-public-export.ps1 -Path D:\export\wira-desk
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$Path,

    [switch]$SkipHistory
)

$ErrorActionPreference = 'Stop'

# Resolved through the PowerShell provider, not `[System.IO.Path]::GetFullPath`. The latter
# resolves a relative path against the *process* working directory, which `Set-Location` and
# `Push-Location` do not change -- so `-Path .` after a `Push-Location` silently scanned a
# different tree than the caller meant. Caught by pointing it at the private repository by
# accident, where it correctly reported hundreds of findings.
if (-not (Test-Path -LiteralPath $Path)) { throw "No such tree: $Path" }
$root = (Resolve-Path -LiteralPath $Path).ProviderPath

# This file carries the forbidden patterns as literals, so it necessarily matches itself.
# Excluded by name rather than by obfuscating the patterns, which would make the gate
# unreadable to hide a problem it does not have.
$selfName = 'verify-public-export.ps1'

# The web extensions were missing, and their absence was not cosmetic. The design system is
# almost entirely `.html`, `.jsx`, `.css`, and `.d.ts`, and it is the one carried corpus that
# is deliberately NOT exempt from the claims rule -- so the rule was scoped to enforce on
# exactly the file types the scanner never opened. It reported 146 of 201 files scanned and
# passed. Three fabricated version badges were sitting in those files the whole time, found by
# reading rather than by the gate. A rule that cannot see its own subject is not a rule.
$textExtensions = @('.rs', '.ps1', '.psm1', '.psd1', '.toml', '.lock', '.md', '.yml', '.yaml',
    '.rc', '.manifest', '.json', '.txt', '.html', '.htm', '.jsx', '.js', '.mjs', '.css', '.ts',
    '.svg', '.log', '')

$failures = New-Object System.Collections.Generic.List[string]
$checked = 0

function Write-Head($msg) { Write-Host "[gate] $msg" -ForegroundColor Cyan }
function Write-Pass($msg) { Write-Host "[gate] PASS  $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "[gate] FAIL  $msg" -ForegroundColor Red }
function Write-Skip($msg) { Write-Host "[gate] SKIP  $msg" -ForegroundColor Yellow }

function Get-TextFiles {
    # `target/` and `out/` are build output: gitignored, never published, and full of
    # absolute paths that would drown every real finding. Excluded so the gate stays about
    # the artefacts it governs.
    Get-ChildItem -LiteralPath $root -Recurse -File |
        Where-Object {
            $_.FullName -notmatch '\\\.git\\' -and
            $_.FullName -notmatch '\\(target|out)\\' -and
            $_.Name -ne $selfName -and
            $_.Extension.ToLowerInvariant() -in $textExtensions
        }
}

$files = @(Get-TextFiles)

# ---------------------------------------------------------------------------------------
# Checks. Each is a name, a regex, and an optional predicate that decides whether a hit is
# permitted. Patterns are bounded by PURPOSE, not by an expected hit count: an earlier
# revision of the identity rule asserted "exactly one hit" and broke the moment a correct
# repository URL added three legitimate ones.
# ---------------------------------------------------------------------------------------

# Local paths and workstation identity. Zero tolerance, no exceptions.
$checkLocalPaths = @{
    Name    = 'local paths'
    Pattern = 'file:///|[A-Za-z]:[/\\]Developer[/\\]|[A-Za-z]:[/\\]Users[/\\]'
    Allowed = { param($file, $line) $false }
}

# Maintainer handle. Permitted only where it serves a stated purpose: attribution in the
# README, and the repository URL in package metadata.
$checkIdentity = @{
    Name    = 'maintainer handle outside its two permitted locations'
    Pattern = 'kodesh87'
    Allowed = {
        param($file, $line)
        if ($file -eq 'README.md') { return $true }
        if ($file -like 'crates/*/Cargo.toml' -and $line -match '^\s*repository\s*=') { return $true }
        # Extended to the carried archives by an explicit owner decision: the handle is
        # already public through commit authorship, so listing it as a meeting participant
        # or a document author adds no exposure, and redacting it would have been a dozen
        # edits with no privacy gain.
        #
        # What this does NOT permit, and what it caught: three references to the *private*
        # repository URL in the design-system readme. Publishing those would have been a
        # dead link for every reader plus a disclosure of the private repo's name. They are
        # not attribution, so the narrowing had to be scoped to the handle appearing as
        # authorship -- not to these paths wholesale.
        if ($file -like '_bmad-output/*' -or $file -like 'design-system/*') { return $true }
        if ($file -like '_bmad/*') { return $true }
        if ($file -like '.claude/skills/*' -or $file -like '.agents/skills/*') { return $true }
        if ($file -like '.control/memlog/*') { return $true }
        return $false
    }
}

# Internal vocabulary.
$checkVocabulary = @{
    Name    = 'internal requirement or process vocabulary in product source'
    Pattern = '\bAC-[0-9]|\bac-[0-9]+\.[0-9]|\bAC-(WD|PUB)-|\bPUB-[0-9]|\bStory\s+[0-9]|\bEpic\s+[0-9]|\bAD-[0-9]|\bNFR[0-9]|spek-to-coding|coding-to-review|review-to-spek|\bhandover|inter-agent'
    Allowed = {
        param($file, $line)
        # Narrowed on purpose, with the reason recorded rather than the pattern quietly
        # widened. This rule exists to keep OPAQUE identifiers out of published source. In
        # the planning archive those identifiers are *defined* -- an epic file is where
        # `AC-2.6-005` gets its meaning -- so there they are a glossary, not noise. The
        # rule's purpose was right; its scope was wrong.
        #
        # `BMAD` and `_bmad` also left the pattern: the workflow is now a documented,
        # installable part of contributing (see CONTRIBUTING.md), so naming it is disclosure
        # rather than leakage.
        if ($file -like '_bmad-output/*' -or $file -like 'design-system/*') { return $true }
        if ($file -like '_bmad/*') { return $true }
        if ($file -like '.claude/skills/*' -or $file -like '.agents/skills/*') { return $true }
        if ($file -like '.control/memlog/*') { return $true }
        if ($file -like '.constitution/method/*') { return $true }
        # The WDI corpus, added when it landed rather than when it started failing CI --
        # it landed one commit after this gate was last aligned, and went red immediately.
        #
        # Same reasoning as the planning archive above, and it has to be, because it is the
        # same category of file. `.how/_platform/ARCHITECTURE-SPINE.md` is where `AD-9` is
        # *defined*; `.control/registry/requirements.yaml` is where `FR-13` is defined. An
        # identifier is opaque when it is referenced far from its meaning -- these are the
        # meaning. Excluding them keeps the rule pointed at what it was written for:
        # `AD-9` turning up in `README.md` or in a Rust doc comment, with nothing in the
        # published tree to say what it is.
        #
        # The pattern itself is untouched, and product source is still enforced. If this
        # rule ever fires on `crates/`, `docs/`, or a root document, that is a real finding.
        if ($file -like '.what/*' -or $file -like '.how/*' -or $file -like '.control/*') { return $true }
        return $false
    }
}

# Public commitments
$checkClaims = @{
    Name    = 'unapproved public claim'
    Pattern = '(?i)Microsoft Store|fully free|free forever|<\s*1\s*ms|<\s*2\s*MB|~\s*0%\s*CPU|\bv1\.[0-9]'
    Allowed = {
        param($file, $line)
        # Exempt for the planning archive ONLY, and the asymmetry is the point.
        #
        # In `_bmad-output` these figures are internal requirement targets -- things the
        # design aimed at, several never met, one formally replaced as unachievable. Its
        # README says exactly that before anything else. Stripping them would gut the
        # requirement documents, and a target stated as a target is not a claim.
        #
        # `design-system` is deliberately NOT exempt. There the identical figures were brand
        # headlines on slides, landing copy, and a UI mockup -- and there they are claims.
        # They were redacted rather than exempted. Same numbers, different context, opposite
        # treatment; if this rule ever starts firing on `design-system`, marketing copy has
        # come back and the fix is at the source.
        if ($file -like '_bmad-output/*') { return $true }
        if ($file -like '.claude/skills/*' -or $file -like '.agents/skills/*') { return $true }
        # The WDI corpus, on the same asymmetry that admits `_bmad-output` and refuses
        # `design-system`. `< 2 MB` in `NFR-1`, in the brief's sizing table, and in the
        # spine's RAM budget is a requirement target -- the number the design aims at, in the
        # document that exists to state what the design aims at. `v1.0.0` in a PRD revision
        # history is a document version, not a shipped release.
        #
        # `README.md`, `docs/`, and `crates/` stay enforced, which is where the same figures
        # would become promises to a reader. That split is the whole rule; it is not widened
        # here, only told where the requirement documents now live.
        if ($file -like '.what/*' -or $file -like '.how/*' -or $file -like '.control/*') { return $true }
        return $false
    }
}

# Language policy
#
#   1. The markers MUST be words that exist ONLY in Indonesian. An earlier revision included
#      "Fatal" -- shared by both languages -- and promptly flagged a correct English
#      translation as a finding.
#   2. This check MUST be case-sensitive. PowerShell's `-match` is case-INSENSITIVE by
#      default, which makes the marker `pada` match the identifier `$padA` (notepad A) in a
#      harness script. That exact false positive was diagnosed once before and reported as
#      four findings; running case-sensitively returns zero.
$checkLanguage = @{
    Name          = 'Indonesian text in a user-facing file'
    Pattern       = '\b(yang|dengan|untuk|tidak|adalah|pada|dari|akan|harus|bisa|karena|sehingga|jendela|lewat|kemudian|sebelumnya|wajib|supaya)\b'
    CaseSensitive = $true
    Allowed       = {
        param($file, $line)
        # Narrowed to what a user or contributor actually reads: the root documents, the
        # curated docs, and product source. The planning archive and the design system are
        # written in Indonesian, and translating roughly eighty files of internal working
        # material is high cost for low value -- so the language policy is amended for them
        # rather than the gate being quietly excepted. Recorded in the export spec (S-07).
        if ($file -like '_bmad-output/*' -or $file -like 'design-system/*') { return $true }
        if ($file -like '_bmad/*') { return $true }
        if ($file -like '.claude/skills/*' -or $file -like '.agents/skills/*') { return $true }
        if ($file -like '.control/memlog/*') { return $true }
        # `.work/` is scratch by definition -- `CLAUDE.md` says it empties when the task
        # closes -- so nothing in it is user-facing, which is what this rule is named after.
        # The owner set this scope deliberately on 2026-08-26: a worklog or a dispatch brief
        # is thinking written down, and forcing it into English costs the thinking in order to
        # save a document nobody outside the task will read.
        #
        # This exempts `.work/` from the LANGUAGE rule only. The other four checks still run
        # over it, and that split is the point: a local machine path or an internal identifier
        # in scratch is exactly as publishable as one in `crates/`.
        if ($file -like '.work/*') { return $true }
        return $false
    }
}

$checks = @($checkLocalPaths, $checkIdentity, $checkVocabulary, $checkClaims, $checkLanguage)

Write-Head "Scanning $($files.Count) text files under $root"

foreach ($check in $checks) {
    $hits = New-Object System.Collections.Generic.List[string]
    foreach ($f in $files) {
        $rel = $f.FullName.Substring($root.Length).TrimStart('\', '/').Replace('\', '/')
        $n = 0
        $caseSensitive = $false
        if ($check.ContainsKey('CaseSensitive')) { $caseSensitive = [bool]$check.CaseSensitive }
        foreach ($line in [System.IO.File]::ReadAllLines($f.FullName)) {
            $n++
            if ($caseSensitive) { $hit = $line -cmatch $check.Pattern }
            else { $hit = $line -match $check.Pattern }
            if ($hit) {
                $permitted = & $check.Allowed $rel $line
                if (-not $permitted) {
                    $hits.Add("${rel}:${n}: $($line.Trim())")
                }
            }
        }
    }
    $checked++
    if ($hits.Count -eq 0) {
        Write-Pass $check.Name
    }
    else {
        Write-Fail "$($check.Name) - $($hits.Count) hit(s)"
        foreach ($h in $hits) {
            $shown = $h
            if ($shown.Length -gt 160) { $shown = $shown.Substring(0, 160) + ' ...' }
            Write-Host "         $shown" -ForegroundColor DarkRed
            $failures.Add("$($check.Name): $shown")
        }
    }
}

# ---------------------------------------------------------------------------------------
# Structural checks: things that must be ABSENT, and history shape.
# ---------------------------------------------------------------------------------------

# `_bmad-output` and `design-system` were on this list and have been removed: the planning
# archive and the design system are now deliberately carried, so their presence is correct
# rather than a leak. What is excluded *within* them is asserted separately below, per file
# pattern, because that is where the real risk sits.
#
# `_bmad/`, `.claude/`, `.agents/`, and `.work/` were removed 2026-08-21: the owner decided
# the committed tooling bundle is part of the public repository so contributors clone a
# working agent setup. Only paths that are genuinely local or build output stay forbidden.
$forbiddenPaths = @(
    '.uv-cache', 'out', 'README-bmad.md'
)
$present = @()
foreach ($p in $forbiddenPaths) {
    if (Test-Path -LiteralPath (Join-Path $root $p)) { $present += $p }
}
$checked++
if ($present.Count -eq 0) {
    Write-Pass 'private corpus absent'
}
else {
    Write-Fail "private corpus present: $($present -join ', ')"
    $failures.Add("private corpus present: $($present -join ', ')")
}

# Carry-over exclusions, asserted independently of the copy logic that implements them.
# The export filters these during enumeration; this check proves the filter worked. A bug
# there should fail the export, not ship quietly -- and the reason each one is excluded is
# recorded in the carry-over contract, not here, because this file is published.
$excludedPatterns = @(
    @{ Name = 'marketing landing and store-listing kit'; Pattern = 'ui_kits[/\\]wintick-web[/\\]' },
    @{ Name = 'studio landing kit'; Pattern = 'ui_kits[/\\]wira-company[/\\]' },
    @{ Name = 'compiled design-system bundle'; Pattern = '_ds_bundle\.js$' },
    @{ Name = 'brand slide deck'; Pattern = 'design-system[/\\].*slides[/\\]' },
    @{ Name = 'standalone design-system viewers'; Pattern = '\(standalone\)\.html$' }
)
$leaked = @()
foreach ($rule in $excludedPatterns) {
    foreach ($f in Get-ChildItem -LiteralPath $root -Recurse -File) {
        if ($f.FullName -match '\\\.git\\') { continue }
        $rel = $f.FullName.Substring($root.Length).TrimStart('\', '/').Replace('\', '/')
        if ($rel -match $rule.Pattern) { $leaked += "$($rule.Name): $rel" }
    }
}
$checked++
if ($leaked.Count -eq 0) {
    Write-Pass "carry-over exclusions honoured ($($excludedPatterns.Count) rules)"
}
else {
    Write-Fail "excluded content present - $($leaked.Count) file(s)"
    foreach ($l in $leaked) {
        Write-Host "         $l" -ForegroundColor DarkRed
        $failures.Add($l)
    }
}

# Trackers are now published, but only the two this repository declares. The private
# repository carried a third under `.constitution/`, dense with governance history, and the
# two it did carry were long-running records rather than the fresh files authored here. So the
# rule is positional: a tracker at an undeclared path is a copied private one until proven
# otherwise, and fails.
$allowedTrackers = @('3p.md', 'docs/3p.md')
$trackers = @(Get-ChildItem -LiteralPath $root -Recurse -File -Filter '3p.md' -ErrorAction SilentlyContinue)
$strayTrackers = @()
foreach ($t in $trackers) {
    $rel = $t.FullName.Substring($root.Length).TrimStart('\', '/').Replace('\', '/')
    if ($allowedTrackers -notcontains $rel) { $strayTrackers += $rel }
}
$checked++
if ($strayTrackers.Count -eq 0) {
    Write-Pass "progress trackers only at declared paths ($($trackers.Count) found)"
}
else {
    Write-Fail "tracker at undeclared path: $($strayTrackers -join ', ')"
    $failures.Add("tracker at undeclared path: $($strayTrackers -join ', ')")
}

# Product version strings MUST agree with the crate version. Pattern-matching a *wrong*
# version was the obvious approach and it is the wrong one -- it requires knowing in advance
# which numbers are wrong, which is exactly what went unnoticed while three design mockups
# advertised 1.2.0 for a product at 0.1.0. Deriving the truth from `Cargo.toml` instead makes
# the rule self-maintaining: it keeps working after the next release, and it fails on drift in
# either direction rather than on a hardcoded list.
$daemonManifest = Join-Path $root 'crates\daemon\Cargo.toml'
if (Test-Path -LiteralPath $daemonManifest) {
    $crateVersion = $null
    foreach ($line in Get-Content -LiteralPath $daemonManifest) {
        if ($line -match '^\s*version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"') { $crateVersion = $Matches[1]; break }
    }
    $checked++
    if (-not $crateVersion) {
        Write-Fail 'could not read the crate version from crates/daemon/Cargo.toml'
        $failures.Add('crate version unreadable')
    }
    else {
        $drift = @()
        foreach ($f in Get-TextFiles) {
            $rel = $f.FullName.Substring($root.Length).TrimStart('\', '/').Replace('\', '/')
            # The planning archive is exempt: it records versions as they were at the time,
            # and rewriting history to match the present would falsify it.
            if ($rel -like '_bmad-output/*') { continue }
            $n = 0
            foreach ($line in (Get-Content -LiteralPath $f.FullName -ErrorAction SilentlyContinue)) {
                $n++
                foreach ($m in [regex]::Matches($line, '(?i)\b(?:WinTick|Wira\s+Desk)\s+v?([0-9]+\.[0-9]+\.[0-9]+)')) {
                    if ($m.Groups[1].Value -ne $crateVersion) { $drift += "${rel}:${n}: $($m.Value)" }
                }
            }
        }
        if ($drift.Count -eq 0) {
            Write-Pass "product version strings agree with the crate ($crateVersion)"
        }
        else {
            Write-Fail "product version drift - $($drift.Count) hit(s), crate is $crateVersion"
            foreach ($d in $drift) {
                Write-Host "         $d" -ForegroundColor DarkRed
                $failures.Add($d)
            }
        }
    }
}

# WDI Method installs `.constitution/method/` (portable guides) and `.constitution/project/`
# (product rules). Both are committed deliberately so contributors share one setup.
$constDir = Join-Path $root '.constitution'
if (Test-Path -LiteralPath $constDir) {
    $constFiles = @(Get-ChildItem -LiteralPath $constDir -Recurse -File |
        ForEach-Object { $_.FullName.Substring($constDir.Length).TrimStart('\', '/').Replace('\', '/') })
    $allowedConst = @(
        'project/',
        'method/'
    )
    $strayConst = @()
    foreach ($rel in $constFiles) {
        $ok = $false
        foreach ($prefix in $allowedConst) {
            if ($rel -eq $prefix -or $rel -like "$prefix*") { $ok = $true; break }
        }
        if (-not $ok) { $strayConst += $rel }
    }
    $checked++
    if ($strayConst.Count -eq 0) {
        Write-Pass 'constitution directory holds WDI method and product rules only'
    }
    else {
        Write-Fail "unexpected constitution files: $($strayConst -join ', ')"
        $failures.Add("unexpected constitution files: $($strayConst -join ', ')")
    }
}

if (-not $SkipHistory) {
    if (Test-Path -LiteralPath (Join-Path $root '.git')) {
        Push-Location $root
        try {
            $count = @(git log --oneline).Count
            $checked++
            if ($count -eq 1) {
                Write-Pass 'fresh history: exactly 1 commit'
            }
            else {
                Write-Fail "history has $count commits - the export must not inherit private history"
                $failures.Add("history has $count commits")
            }
        }
        finally { Pop-Location }
    }
    else {
        # Not counted as a check, and said out loud rather than passed over. A missing `.git`
        # is a legitimate state -- verifying a plain directory -- but silence here would be
        # indistinguishable from a check that ran and approved.
        Write-Skip 'fresh history: no .git directory, nothing to inspect'
    }
}
else {
    # Announced, because the alternative caused real doubt. Under `-SkipHistory` the total drops
    # by one, and a smaller number with no explanation reads exactly like a gate that quietly
    # lost a rule -- which took a dig through CI logs to rule out. The count must explain itself.
    Write-Skip 'fresh history: skipped (-SkipHistory), so the total below is one lower'
}

# ---------------------------------------------------------------------------------------

Write-Host ''
if ($failures.Count -eq 0) {
    Write-Host "[gate] $checked checks passed. Tree may be published." -ForegroundColor Green
    exit 0
}

Write-Host "[gate] $($failures.Count) finding(s). Tree MUST NOT be published." -ForegroundColor Red
Write-Host '[gate] Fix the source, not this gate. Widening a pattern to make a finding' -ForegroundColor Yellow
Write-Host '[gate] disappear is the failure mode this file exists to prevent.' -ForegroundColor Yellow
exit 1
