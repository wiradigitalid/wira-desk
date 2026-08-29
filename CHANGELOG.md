# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versioning is
[semantic](https://semver.org/). Below `1.0` the **minor** digit carries the breaking change —
`0.1.x` to `0.2.0` is the incompatible step.

Two things depend on the shape of this file, so the headings are not free-form:

- `release.yml` extracts the `## [x.y.z]` section matching the git tag and publishes it as the
  release notes. A tag with no matching section **fails the release** rather than shipping
  notes nobody wrote.
- The in-app updater shows the section for the version it is offering, so this is what a user
  reads before deciding to update. Write it for them, not for the commit log.

Who may move which digit is a rule, not a convention — see `AGENTS.md`, "Versioning authority".
Work that needs a minor or major bump belongs under **Unreleased** and stays there until the
owner decides.

## [Unreleased]

Nothing yet.

## [0.1.4] - 2026-08-29

No functional change from 0.1.3. This version exists to be found: an installed 0.1.3 checking
`releases/latest/download/latest.json` for the first time against a real newer tag, with no
debug seam or stand-in descriptor involved anywhere -- the last half of gate 2 this repository
had not yet proven for real. See 3p.md for what it verified.

## [0.1.3] - 2026-08-29

No functional change from 0.1.2. This version exists to run `release.yml` for the first time
ever — the tag-vs-crate check, the Inno Setup build inside CI, and the real
`releases/latest/download/latest.json` this repository's own updater will check against, none
of which had executed even once before this tag. A deliberate, disposable step before any
version is promoted as the one people should install; see 3p.md for what it verified.

## [0.1.2] - 2026-08-28

No change from 0.1.1, which was itself never published. This version exists to carry a
rebuild — one that verifies upgrade-in-place from an installed 0.1.1, and one that tests
whether a freshly built unsigned binary is refused by Smart App Control purely for being new.

**Three unreleased sections now stand below this one, and that is two too many.** At release
time they collapse into a single section for whichever version is tagged first: a published
changelog should not advertise versions nobody could ever install. That collapse is the
owner's call, because which version becomes the first public release is a release decision,
not a patch.

## [0.1.1] - 2026-08-28

0.1.0 was never published, so nothing here is a regression from a shipped version. What it
fixes is a defect that only appears when Settings is opened from Program Files rather than
from the tray icon.

### Fixed

- **Settings opened outside the tray could not reach Wira Desk, and said nothing about it.**
  The background process runs elevated; Settings inherits that when the tray launches it and
  runs at ordinary privilege when you start it yourself. Windows discards messages sent from
  the second to the first, so shortcut recording missed keys and a saved change never
  reached the running program — under a status line that read like an ordinary success. Both
  messages are now admitted explicitly, a refused message is retried instead of being
  remembered as sent, and the status line distinguishes "Wira Desk is not running" from
  "Wira Desk refused it", because those need different actions from you.
- Uninstalling removed every installed file but left an empty `Wira Desk` folder behind in
  Program Files. The folder is now removed too, and only when it is genuinely empty.
- `scripts/bump-version.ps1` read the manifest as the system codepage and wrote it back with a
  byte-order mark, corrupting the prose above the version it was there to change. It also
  redirected `cargo`'s stderr, which under Windows PowerShell 5.1 ends the script on the first
  progress line — after the manifest was written and before `Cargo.lock` was, leaving a lock
  that fails CI's `--locked` build. The refreshed lock is now verified against the file rather
  than assumed from an exit code.

## [0.1.0] - 2026-08-27

Initial public release of Wira Desk.

### Added

- Same-application window cycling, and an overlapping stack arrangement with a configurable
  width ratio. The stack is **on by default**: the setting gates only the arrangement, so
  off meant the shortcut silently did nothing rather than the feature being disabled.
- A daily check for new versions, on by default and switchable off in Settings. It is the
  only network request the product makes, and `PRIVACY.md` describes it line by line.
- A settings window with shortcut configuration, a live key-check diagnostic, and a first-run
  tutorial.
- Optional start at sign-in, through a Windows scheduled task.
- A per-machine installer, built by CI on every push rather than only at release time, and
  published with SHA-256 checksums.
- Installation through winget, as a second channel alongside the installer.

### Security

- The scheduled task no longer trusts the path it stored: an install that moved, or a task
  written by an older version, is rewritten rather than followed. A location writable by a
  non-administrator is reported, because a task that runs elevated at every sign-in with no
  prompt turns such a directory into a privilege-escalation route.
- The installer toolchain is pinned by SHA-256 with a security floor asserted during the build,
  so a version carrying a known privilege-escalation flaw cannot be reintroduced quietly.

### Known limitations

- **The binaries are not code-signed.** Windows SmartScreen will warn, and the elevation prompt
  will show an unverified publisher. `SECURITY.md` says what that does and does not tell you.
- On a Windows 11 machine with **Smart App Control** enforcing, the daemon will not start at
  all. Unlike SmartScreen this offers no way through — it judges the file itself, and an
  unsigned binary no reputation service has seen yet is refused with only an entry in the
  CodeIntegrity event log to show for it. Code signing is the fix; there is no setting in
  Wira Desk that changes it.
- `wiradesk.log` is capped by size and rotated to a single `.old` generation; there is no
  longer-term retention.

**Factory reset:** delete `%APPDATA%\WiraDesk\config.toml` only — not the folder. Migration
re-runs if the legacy `%APPDATA%\WinTick\` directory still exists; that directory is
intentionally preserved for rollback, so removing the entire `WiraDesk` folder is not a reset.
