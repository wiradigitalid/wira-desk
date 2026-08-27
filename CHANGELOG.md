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

## [0.1.0] - 2026-08-27

Initial public release of Wira Desk.

### Added

- Same-application window cycling, and an overlapping stack arrangement with a configurable
  width ratio. The stack is **on by default**: the setting gates only the arrangement, so
  off meant the shortcut silently did nothing rather than the feature being disabled.
- An optional check for new versions, off until you turn it on. It is the only network
  request the product makes, and `PRIVACY.md` describes it line by line.
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
- `wiradesk.log` is capped by size and rotated to a single `.old` generation; there is no
  longer-term retention.

**Factory reset:** delete `%APPDATA%\WiraDesk\config.toml` only — not the folder. Migration
re-runs if the legacy `%APPDATA%\WinTick\` directory still exists; that directory is
intentionally preserved for rollback, so removing the entire `WiraDesk` folder is not a reset.
