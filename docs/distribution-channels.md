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
| winget | **PR open**: [microsoft/winget-pkgs#426321](https://github.com/microsoft/winget-pkgs/pull/426321) | Wait for community review/merge; separately, set the `WINGET_TOKEN` repo secret so future releases update automatically | `release.yml`'s `winget` job takes over on the next tag |
| Scoop | **Live**: [wiradigitalid/scoop-wiradesk](https://github.com/wiradigitalid/scoop-wiradesk) | Nothing — `excavator.yml` is already polling on its own schedule | Updates itself forever, no action needed |
| SourceForge | **Not started** — no account exists yet | Create a SourceForge account/project + SSH key, store `SF_USER`/`SF_SSH_KEY` as repo secrets (see below) — this one genuinely needs a human, it is a separate identity from GitHub | `release.yml`'s `sourceforge` job takes over on the next tag |

Check `3p.md`'s Progress entries for the running account of what changed and when.

## winget

See `packaging/winget/README.md` for the full history, including a real `wingetcreate` schema-
version gotcha hit while submitting. Short version: the 0.1.4 manifest was submitted
2026-08-30 as [microsoft/winget-pkgs#426321](https://github.com/microsoft/winget-pkgs/pull/426321).
`wingetcreate` forked `microsoft/winget-pkgs` under whichever GitHub account owned the
submitting token, not necessarily `wiradigitalid` — the PR itself names which. Once that PR
merges, set the `WINGET_TOKEN` repo secret and `release.yml`'s existing `winget` job carries
every release after.

## Scoop

Live at [wiradigitalid/scoop-wiradesk](https://github.com/wiradigitalid/scoop-wiradesk), pushed
2026-08-30. This is its own bucket rather than a submission to Scoop's `extras` bucket, because
the installer requires Administrator and installs to `%ProgramFiles%` — `extras` expects
portable, user-scoped packages, and this is neither. `.github/workflows/excavator.yml` in that
repo needs no further attention: it polls wira-desk's GitHub releases every four hours and bumps
the manifest itself. `packaging/scoop-bucket/` in this repo remains the source of truth if the
bucket ever needs to be regenerated or re-pushed.

## SourceForge

Nothing to generate here — `release.yml`'s `sourceforge` job (see its own comments) already
knows how to upload once the account side exists. That side is entirely manual and entirely
outside this repository:

1. Create a SourceForge account, then a project — Account → Create → New Project. The project
   name suggested throughout `release.yml` and this file is `wiradesk`; if that name is taken,
   pick another and update the `rsync` destination path in the `sourceforge` job to match.
2. The File Release System is available on a new project immediately — no approval wait.
3. Account Settings → SSH Settings → add a key. Generate one dedicated to this
   (`ssh-keygen -t ed25519 -f sourceforge_wiradesk -C "wiradesk-release-bot"`), upload the
   PUBLIC half there, then:
   ```powershell
   gh secret set SF_USER --body "<your-sourceforge-username>"
   gh secret set SF_SSH_KEY < sourceforge_wiradesk
   ```
4. Next tag, the `sourceforge` job stops skipping and mirrors the release automatically.

No PR, no fork, no third-party review — SourceForge's own account/project creation is the only
step, and it is the user's identity being registered, so it is left here as documentation
rather than something run on their behalf.
