# scoop-wiradesk

A [Scoop](https://scoop.sh) bucket for [Wira Desk](https://github.com/wiradigitalid/wira-desk).

This is a dedicated bucket rather than a submission to Scoop's `extras` bucket, because Wira
Desk's installer requires Administrator and installs to `%ProgramFiles%` - it is not portable
and not user-scoped, which is what `extras` expects of the packages it accepts. Running it
through Scoop is still safe: `scoop install` will show the same UAC prompt the plain installer
or `winget install` would.

## Install

```powershell
scoop bucket add wiradesk https://github.com/wiradigitalid/scoop-wiradesk
scoop install wiradesk
```

## Update

```powershell
scoop update wiradesk
```

## How this bucket stays current

`bucket/wiradesk.json` carries `checkver`/`autoupdate` fields pointing at wira-desk's own GitHub
releases. `.github/workflows/excavator.yml` runs on a schedule (every four hours) and bumps the
manifest automatically the moment a new tag's release is published there - nothing needs to be
pushed from the wira-desk repository itself for this to happen.

Manual regeneration, if ever needed: bump `version` and `url` by hand, or trigger the
`Excavator` workflow's `workflow_dispatch` from the Actions tab.

## Source of truth

The staged content of this bucket lives at `packaging/scoop-bucket/` in the
[wira-desk](https://github.com/wiradigitalid/wira-desk) repository, and this repository is a
plain copy of it. If the two ever disagree, wira-desk's copy is correct; push its content here
again to fix the drift.
