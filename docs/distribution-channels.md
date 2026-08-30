# Distribution channels

Three channels beyond the plain GitHub release, and what each one needs before it does
anything: none of them need a code-signing certificate (see `3p.md`'s signing-research entry
for why that question keeps coming up), but each has exactly one piece only a human can do —
create an account, fork a repo, open a PR under that account — and this file exists so that
piece is not lost between releases.

Distribution and promotion are not the same thing. Listing this project on winget, Scoop, or
SourceForge is quiet, one-time plumbing with no audience-facing moment to protect. A promotion
push (Product Hunt, Hacker News, curated download aggregators) drives a burst of first-time
downloads at once, and an unsigned SmartScreen warning during that burst wastes a moment that
does not come back — that is the piece that should wait for a signed certificate, not this.

## Status

| Channel | State (2026-08-30) | What's left | Then |
|---|---|---|---|
| winget | **PR open**: [microsoft/winget-pkgs#426321](https://github.com/microsoft/winget-pkgs/pull/426321) | Wait for community review/merge; `WINGET_TOKEN` repo secret is already set | `release.yml`'s `winget` job takes over on the next tag |
| Scoop | **Live**: [wiradigitalid/scoop-wiradesk](https://github.com/wiradigitalid/scoop-wiradesk) | Nothing — `excavator.yml` is already polling on its own schedule | Updates itself forever, no action needed |
| SourceForge | **Live**: [sourceforge.net/projects/wira-desk](https://sourceforge.net/projects/wira-desk/) | Nothing — see below, this needed no workflow code at all | Every future GitHub release is mirrored automatically by SourceForge itself |

Check `3p.md`'s Progress entries for the running account of what changed and when.

## winget

See `packaging/winget/README.md` for the full history, including a real `wingetcreate` schema-
version gotcha hit while submitting. Short version: the 0.1.4 manifest was submitted
2026-08-30 as [microsoft/winget-pkgs#426321](https://github.com/microsoft/winget-pkgs/pull/426321).
`wingetcreate` forked `microsoft/winget-pkgs` under whichever GitHub account owned the
submitting token, not necessarily `wiradigitalid` — the PR itself names which. The `WINGET_TOKEN`
repo secret is already set; once the PR merges, `release.yml`'s existing `winget` job carries
every release after with no further action.

## Scoop

Live at [wiradigitalid/scoop-wiradesk](https://github.com/wiradigitalid/scoop-wiradesk), pushed
2026-08-30. This is its own bucket rather than a submission to Scoop's `extras` bucket, because
the installer requires Administrator and installs to `%ProgramFiles%` — `extras` expects
portable, user-scoped packages, and this is neither. `.github/workflows/excavator.yml` in that
repo needs no further attention: it polls wira-desk's GitHub releases every four hours and bumps
the manifest itself. `packaging/scoop-bucket/` in this repo remains the source of truth if the
bucket ever needs to be regenerated or re-pushed.

## SourceForge

No workflow code needed at all, in the end. SourceForge has a built-in **GitHub Releases
Integration**: a webhook, configured from the SourceForge project's Files page, that fires on
this GitHub repository's `release` event and copies the new files into the project's File
Release System on its own. The project is `wira-desk` (note the hyphen — not `wiradesk`, which
several earlier drafts of this file and `release.yml` assumed before the project actually
existed) at https://sourceforge.net/projects/wira-desk/, and the webhook is visible under this
repo's Settings → Webhooks, pointed at `sourceforge.net/p/wira-desk/files-sf/github_webhook`,
firing on `release` only.

An earlier version of this repository carried a hand-rolled `sourceforge` job in `release.yml`
that did the equivalent with `rsync` over SSH, written before this native integration was found.
It was removed once the redundancy was clear — see `3p.md`'s 2026-08-30 entry — rather than kept
as a second, competing path to the same file tree. Nothing further is needed here: the next
GitHub release this repository publishes is the first real test of the webhook.
