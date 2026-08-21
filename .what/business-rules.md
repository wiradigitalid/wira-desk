---
type: rules
scope: global
status: reviewed
created: 2026-08-21
updated: 2026-08-21
---

# Business Rules — Wira Desk

Cross-component business rules binding more than one Product Component (`window-management`, `settings`).

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| BR-1 | Configuration persistence is written exclusively by the settings process; the background daemon reloads configuration only upon receiving an explicit IPC signal (`WM_APP_RELOAD_CONFIG`), never through background file-system polling or file watching. | `settings`, `window-management` | FR-7, AD-1, AD-5 | active |
| BR-2 | Both binaries read and write `%APPDATA%\WiraDesk\config.toml` strictly adhering to the shared schema owned by `_platform`, guaranteeing configuration forward and backward compatibility across versions. | `settings`, `window-management` | FR-7, CAP-3, AD-12 | active |
| BR-3 | First-run onboarding may launch the settings process elevated directly from the parent daemon without triggering a secondary User Account Control (UAC) elevation prompt in the same logon session. | `settings`, `window-management` | FR-8, FR-17, AD-11 | active |
| BR-4 | The auto-start scheduled task must launch the background daemon with highest privileges configured specifically for the active `%USERNAME%`, ensuring user AppData directory paths and privilege integrity levels align exactly. | `settings`, `window-management` | FR-13, CAP-10, AD-13 | active |
| BR-5 | System tray "View Logs" action opens the diagnostic log file itself in a plain-text viewer; the settings UI does not duplicate log inspection or file handling interfaces. | `window-management`, `settings` | FR-12, CAP-11, AD-1 | active |

## Rationale & Enforcement

### BR-1 — Explicit IPC Configuration Reload
- **Rationale:** Background file watchers (e.g. `ReadDirectoryChangesW`) incur OS thread overhead, risk race conditions with partial/in-flight file writes, and consume unnecessary idle memory. By restricting file writes strictly to `wiradesk-settings.exe` and notifying the daemon via an explicit Windows message (`WM_APP_RELOAD_CONFIG`), configuration updates are atomic, deterministic, and zero-overhead when idle.
- **Enforcement:** The daemon never instantiates a file watcher. Settings performs an atomic write (temporary file swap) and dispatches `WM_APP_RELOAD_CONFIG` to the daemon hidden message window handle.

### BR-2 — Canonical Shared Configuration Schema
- **Rationale:** Having multiple binaries interpret configuration independently risks schema drift, corrupted preferences, or parser panics. Placing the configuration data model and parsing logic in the shared crate ensures identical serialization, deserialization, default value fallbacks, and validation rules across both the headless daemon and settings UI.
- **Enforcement:** Both `crates/daemon` and `crates/settings` depend on `crates/shared` as the sole source of truth for the TOML configuration struct.

### BR-3 — Seamless Elevated Onboarding Handoff
- **Rationale:** Wira Desk runs elevated to manipulate administrator windows under Windows UIPI. Triggering repeated UAC elevation dialogs during initial onboarding frustrates new users and creates security fatigue. Spawning `wiradesk-settings.exe` via `ShellExecute` inheriting parent process token privileges delivers a smooth first-run onboarding experience.
- **Enforcement:** The elevated daemon process launches the settings binary passing the `--onboarding` flag using standard elevated process token inheritance.

### BR-4 — Per-User Elevated Auto-Start Alignment
- **Rationale:** If the scheduled task runs under the `SYSTEM` account, `%APPDATA%` resolves to `C:\Windows\System32\config\systemprofile\AppData\Roaming`, causing the daemon to miss user configuration and disconnect from the interactive desktop session. Configuring the task with `/RU %USERNAME%` and `/RL HIGHEST` ensures the daemon accesses the user's roaming directory while maintaining the administrative privilege level required for UIPI bypass.
- **Enforcement:** The daemon owns task registration (`crates/daemon/src/autostart.rs`), invoking `schtasks /Create /TN WiraDesk /TR "\"<exe_path>\"" /SC ONLOGON /RL HIGHEST /RU "%USERNAME%" /F`. The settings process never calls `schtasks`: it writes `general.auto_start` to `config.toml` and signals reload, and the daemon reconciles the task on reload. `schtasks /Query` is the authoritative source for the menu checkmark, not the config value.

### BR-5 — Single Responsibility for Diagnostic Log Access
- **Rationale:** Duplicate UI for log viewing inside the settings binary adds bloat and complicates UI navigation. Handing the log straight to a text viewer from the tray menu keeps the settings binary focused purely on preferences and onboarding, adhering to single-responsibility boundaries. Opening the file rather than its folder is the deliberate choice: the folder holds `config.toml` beside the log, and pointing a user at a directory to find one file is a step the tray menu can simply take for them.
- **Enforcement:** The tray context menu "View Logs" handler (`crates/daemon/src/menu.rs`, `view_logs`) creates the log file if absent and spawns `notepad.exe <log_path>` with `CREATE_NO_WINDOW`; settings UI omits log viewer components. Log writes are open-write-close per line precisely so the file can be read by a viewer while the daemon is running.

## Retired

*(No retired business rules)*
