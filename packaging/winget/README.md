# winget

This folder holds the winget manifest as it should read for each released version — the
source of truth kept in-repo, regenerated from the actual GitHub release rather than hand-edited.
It is not itself what `winget install` reads; `microsoft/winget-pkgs` is, and getting a copy of
this content there is a separate, external step (see below).

## Two different jobs, and this folder is only the first

1. **Getting a version's manifest into `microsoft/winget-pkgs` at all.** Manual, every time, and
   `scripts/generate-winget-manifest.ps1` plus `wingetcreate submit` is how.
2. **Keeping it updated after that.** Automatic, forever, via the `winget` job already in
   `.github/workflows/release.yml` (`vedantmgoyal9/winget-releaser`) — but that job only works on
   a package that has ALREADY been accepted at least once. It cannot do job 1.

Confusing the two is the trap: winget-releaser existing in `release.yml` does not mean winget
already works. Nothing has been submitted yet; `winget install WiraDigitalIndonesia.WiraDesk`
will fail with "no package found" until the steps below happen once.

## One-time setup (do this once, ever)

1. Fork `microsoft/winget-pkgs` under the `wiradigitalid` GitHub account (or your own — the fork
   just needs to be reachable from the same account that owns the token below).
2. Create a **classic** GitHub personal access token with the `public_repo` scope. Fine-grained
   tokens are not accepted by either `wingetcreate` or the `winget-releaser` action.
3. Store it as the `WINGET_TOKEN` secret on this repository (`gh secret set WINGET_TOKEN`) — this
   is what flips `release.yml`'s `winget` job from skipped to active for every future tag.
4. Install `wingetcreate` (https://github.com/microsoft/winget-create) — `winget install
   Microsoft.WingetCreate`, or download the exe from that repo's releases.

## Submitting the first version (manual, one PR)

```powershell
pwsh scripts/generate-winget-manifest.ps1 -Version 0.1.4
wingetcreate submit --prtitle "New package: WiraDigitalIndonesia.WiraDesk version 0.1.4" `
  --token $env:WINGET_TOKEN packaging\winget\manifests\w\WiraDigitalIndonesia\WiraDesk\0.1.4
```

`wingetcreate submit` forks `microsoft/winget-pkgs` if needed, commits the three files at the
path winget-pkgs itself expects, and opens the pull request — it is community-reviewed and
typically takes a few days. **This is the external, visible action**: it opens a PR under your
account against a Microsoft repository, so treat it the same as any other public submission —
review the rendered manifest one more time before running it.

Nothing else needs doing after this PR merges except making sure the `WINGET_TOKEN` secret
(step 3 above) is set — from the next tag onward, `release.yml`'s `winget` job takes over
automatically, and this folder + script are only needed again if that automation is ever broken
and a manifest has to be regenerated and resubmitted by hand.

## Regenerating for a later version

`scripts/generate-winget-manifest.ps1 -Version X.Y.Z` reads the already-published GitHub release
for `vX.Y.Z` (URL and SHA-256, never transcribed by hand) and writes a fresh
`packaging/winget/manifests/w/WiraDigitalIndonesia/WiraDesk/X.Y.Z/` folder. Normally you will
never run this: `release.yml`'s automated job does the equivalent on every tag once winget knows
about the package. It exists for the case where that automation needs a manual assist.
