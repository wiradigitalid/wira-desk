# Dispatch brief — NSIS installer for Wira Desk

Self-contained. Do not assume access to any prior conversation.

## Goal

Add a per-machine NSIS installer that packages `wiradesk.exe` and `wiradesk-settings.exe` into
`%ProgramFiles%\Wira Desk\`, and wire it into the tag-triggered release workflow so a `v*` tag
publishes `WiraDesk-Setup-<version>.exe` alongside the two raw binaries.

## Files in scope

Create:

- `packaging/wiradesk.nsi`
- `packaging/README.md` — how to build the installer locally, nothing else

Modify:

- `.github/workflows/release.yml`
- `README.md` — the `## Install` section, which currently tells the user to copy two files by hand
- `3p.md` — add a Progress entry describing what was added and why it is not obvious

Do not touch anything else. In particular: no changes under `crates/`, no changes to `build.ps1`,
no changes to `SECURITY.md` (the orchestrator owns that edit).

## Facts from this codebase that constrain the installer

Read these before writing the script; every one of them is load-bearing.

1. **Both executables must land in the same folder.** `crates/shared/src/constants.rs`
   (`SETTINGS_EXE_NAME`) — the daemon builds the path to the settings executable relative to its
   own install folder. Splitting them breaks the tray Settings action.

2. **Per-machine install into `%ProgramFiles%` is a security requirement, not a preference.**
   `SECURITY.md`, "Hardening guidance": the auto-start task runs the daemon elevated at every logon
   with no prompt, so any user-writable install path is an unprompted elevated foothold. The
   installer therefore requires admin and must never fall back to a per-user location.

3. **The installer must NOT create the scheduled task.** `crates/daemon/src/autostart.rs` owns it:
   `schtasks /Create /TN WiraDesk /TR "<abs path>" /SC ONLOGON /RL HIGHEST /RU <user> /F`, toggled
   from the app's own Auto-Start menu item. `schtasks /Query` is the authoritative source for that
   checkmark, so a task created behind the app's back is a second source of truth.

4. **The uninstaller MUST remove that task**: `schtasks /Delete /TN WiraDesk /F`. The task name is
   `TASK_NAME` in `crates/shared/src/constants.rs`. Skipping this leaves an orphaned task pointing
   at a deleted executable.

5. **Single-instance mutexes exist and are the reliable way to detect a running instance**
   (`crates/shared/src/constants.rs`):
   - `Global\WiraDeskSingleInstanceMutex` (daemon)
   - `Global\WiraDeskSettingsSingleInstanceMutex` (settings window)

6. **User data lives at `%APPDATA%\WiraDesk\`** (`config.toml`, `wiradesk.log`). `README.md`
   "Factory reset" and `CHANGELOG.md` both warn that deleting the *folder* is not a reset:
   migration re-runs from a legacy `%APPDATA%\WinTick\` directory that is deliberately preserved.
   The uninstaller must not delete that folder by default.

7. **The daemon is elevated and the settings window is not necessarily.** The uninstaller runs
   elevated, so terminating the daemon is permitted.

## Requirements

**R1.** `packaging/wiradesk.nsi` builds with `makensis` and produces `WiraDesk-Setup-<version>.exe`.
Version comes from the command line (`/DVERSION=...`), defaulting to the value in
`crates/daemon/Cargo.toml` when not supplied. Do not hardcode a second copy of the version.

**R2.** `Unicode true`, `RequestExecutionLevel admin`, x64 only, default install directory
`$PROGRAMFILES64\Wira Desk`.

**R3.** Installer and uninstaller icons use `crates/daemon/wiradesk.ico`. The license page shows
`LICENSE` (MIT).

**R4. Upgrade safety.** Before writing any file, the installer must detect a running instance via
the two mutexes in fact 5 and refuse to overwrite while one is held — with a dialog naming what to
close, and a retry. Only after the user consents may it terminate `wiradesk.exe` and
`wiradesk-settings.exe`, and it must verify termination before proceeding rather than assuming it
worked. Overwriting a locked executable half-installs silently, and that is the most likely
real-world failure of this installer.

**R5. Uninstaller.** Stops running instances (same handling as R4), runs
`schtasks /Delete /TN WiraDesk /F`, removes the installed files, the Start Menu shortcut, and the
uninstall registry entry. It offers to remove `%APPDATA%\WiraDesk\` as an **unchecked, opt-in**
step, with text stating that keeping it preserves settings — see fact 6.

**R6. Uninstall registry entry** under
`HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\WiraDesk`, with `DisplayName`,
`DisplayVersion`, `Publisher` (`Wira Digital Indonesia`), `DisplayIcon`, `UninstallString`,
`QuietUninstallString`, `EstimatedSize`, `NoModify`, `NoRepair`, `URLInfoAbout`.

**R7. Silent install must work** — `/S` for both install and uninstall, no dialogs, correct exit
codes. This is a prerequisite for a future winget submission, so treat it as a requirement, not a
nicety.

**R8.** Start Menu shortcut for `wiradesk.exe` only. No desktop shortcut, and no "run at finish"
checkbox that would start an elevated daemon the user did not ask for.

**R9. `release.yml`.** After the existing `cargo build --workspace --release --locked` step, build
the installer into `dist\` so the existing `Get-FileHash dist\*.exe` step covers it, and add the
installer to the `gh release create` argument list. Verify whether `makensis` is present on the
`windows-latest` runner; if it is not, install it explicitly in the workflow rather than hoping.
Leave a clearly marked comment where a future code-signing step belongs — but do not add signing.

**R10.** `README.md` `## Install` becomes: download the installer from Releases, run it, done —
keeping the existing manual-copy instructions as a secondary "from source" path, since the project
still publishes raw binaries.

## MUST NOT

- MUST NOT embed any absolute path from a local machine (for example a `D:\...` developer path)
  anywhere. `scripts/verify-public-export.ps1` is a CI gate and will fail the build. Use paths
  relative to the repository root.
- MUST NOT add product claims (performance numbers, "secure", version badges) to any file. Same gate.
- MUST NOT create the scheduled task, write to `%APPDATA%`, or start the daemon from the installer.
- MUST NOT modify `crates/`, `build.ps1`, or `SECURITY.md`.
- MUST NOT add a plugin download or a vendored binary without stating it explicitly in the final
  report. If an NSIS plugin looks necessary, prefer a plain `nsExec` call to a system tool.

## Definition of done

1. `makensis` builds `packaging\wiradesk.nsi` with zero warnings, against binaries produced by
   `.\build.ps1 -Mode prod`.
2. Installer verified by hand: fresh install into `%ProgramFiles%\Wira Desk\`, both executables
   present in the same folder, Start Menu shortcut works, the app appears in Apps & Features.
3. Upgrade-over-running-instance verified: start the daemon, run the installer, confirm it detects
   and refuses rather than corrupting the install.
4. Uninstall verified: files gone, `schtasks /Query /TN WiraDesk` reports the task is absent, and
   `%APPDATA%\WiraDesk\` still present because the opt-in box was left unchecked.
5. `/S` silent install and silent uninstall verified.
6. These pass from the repository root:

   ```powershell
   .\scripts\verify-public-export.ps1 -Path . -SkipHistory
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   $env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
   ```

7. `3p.md` carries a Progress entry recording what was added and the reasoning not readable from
   the script itself — specifically why the installer does not create the scheduled task, and why
   `%APPDATA%` removal is opt-in.

## Report back

State which files changed, whether each numbered verification above passed or was skipped and why,
any plugin or external tool introduced, and anything changed that this brief did not authorise.
