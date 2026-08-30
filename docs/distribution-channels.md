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

| Channel | Manifest/script ready | The one human step | Then |
|---|---|---|---|
| winget | Yes — `packaging/winget/` | Fork `microsoft/winget-pkgs`, get a classic PAT, run `wingetcreate submit` once | `release.yml`'s `winget` job updates it forever |
| Scoop | Yes — `packaging/scoop-bucket/` | Create the `wiradigitalid/scoop-wiradesk` GitHub repo and push this folder to it once | `excavator.yml` (already in that folder) updates it forever, on its own schedule |
| SourceForge | Yes — `release.yml`'s `sourceforge` job | Create a SourceForge project + SSH key, store `SF_USER`/`SF_SSH_KEY` as repo secrets once | Every tag mirrors there automatically, same as winget |

Nothing in this table has happened yet. All three are still "prepared", not "live" — check
`3p.md`'s Progress entries for the date any of these actually flips.

## winget

See `packaging/winget/README.md` for the full one-time bootstrap and the regeneration script.
Short version: `scripts/generate-winget-manifest.ps1 -Version X.Y.Z` writes the manifest from
the real GitHub release, `wingetcreate submit` opens the PR. Do this once; `release.yml`
already carries the automation for every release after the first is accepted.

## Scoop

See `packaging/scoop-bucket/README.md`. Short version: this project needs its own bucket
rather than a submission to Scoop's `extras` bucket, because the installer requires
Administrator and installs to `%ProgramFiles%` — `extras` expects portable, user-scoped
packages, and this is neither.

To go live:

```powershell
gh repo create wiradigitalid/scoop-wiradesk --public --description "Scoop bucket for Wira Desk"
git -C packaging/scoop-bucket init
git -C packaging/scoop-bucket add -A
git -C packaging/scoop-bucket commit -m "Initial bucket"
git -C packaging/scoop-bucket remote add origin https://github.com/wiradigitalid/scoop-wiradesk.git
git -C packaging/scoop-bucket push -u origin main
```

This creates a public repository under the `wiradigitalid` account — an external, visible
action, so it is written out here rather than run automatically. After it exists,
`.github/workflows/excavator.yml` in that repo needs no further attention: it polls wira-desk's
GitHub releases every four hours and bumps the manifest itself.

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
