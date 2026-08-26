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
| BR-6 | Two actions configured to the same chord is a defined condition, and the two components answer it differently on purpose. The settings process refuses to save a configuration containing one, naming both fields. The daemon, which has no such veto over a file it did not write, keeps the chord for whichever field comes first in the fixed precedence order, leaves the later field unbound, and emits exactly one Tier-2 warning naming both — except on an explicit reload, where a last-known-good configuration exists and the whole candidate is refused instead. | `settings`, `window-management` | FR-7, FR-18, DEC-001, DEC-009 | active |
| BR-7 | The auto-start task's stored executable path must track the running daemon rather than the daemon's location at the moment auto-start was switched on, and the safety of that location must be reported to the user without ever being enforced against them. | `window-management`, `settings` | FR-13, CAP-10, AD-13, AD-7 | active |

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
- **Enforcement:** The daemon owns task registration (`crates/daemon/src/autostart.rs`), invoking `schtasks /Create /TN WiraDesk /TR "\"<exe_path>\"" /SC ONLOGON /RL HIGHEST /RU "%USERNAME%" /F`. The settings process never calls `schtasks`: it writes `general.auto_start` to `config.toml` and signals reload, and the daemon reconciles the task on reload. `schtasks /Query` is the authoritative source for the menu checkmark, not the config value — and it stays a pure existence check: keeping the stored path current is `BR-7`'s job, deliberately separated so the checkmark cannot go blank over a drifted path.

### BR-5 — Single Responsibility for Diagnostic Log Access
- **Rationale:** Duplicate UI for log viewing inside the settings binary adds bloat and complicates UI navigation. Handing the log straight to a text viewer from the tray menu keeps the settings binary focused purely on preferences and onboarding, adhering to single-responsibility boundaries. Opening the file rather than its folder is the deliberate choice: the folder holds `config.toml` beside the log, and pointing a user at a directory to find one file is a step the tray menu can simply take for them.
- **Enforcement:** The tray context menu "View Logs" handler (`crates/daemon/src/menu.rs`, `view_logs`) creates the log file if absent and spawns `notepad.exe <log_path>` with `CREATE_NO_WINDOW`; settings UI omits log viewer components. Log writes are open-write-close per line precisely so the file can be read by a viewer while the daemon is running.

### BR-6 — One Chord, One Action, Answered Differently on Each Side
- **Rationale:** The two components stand in different positions and a single uniform answer is dishonest at one end or the other. Settings owns the file it writes and has a human in front of it, so it can refuse a collision at the moment it is created and point at both fields — `DEC-001`'s shape. The daemon reads a file it did not write and may be starting with nothing to fall back on; refusing there means either no daemon or a daemon on full defaults, which spends every unrelated setting the user owns to resolve one ambiguous pair. Unbinding the later field is proportionate: the user loses exactly what was ambiguous. On an explicit reload the calculus inverts — a last-known-good configuration exists, a human just pressed Save, and Settings cannot have produced the collision, so a duplicate arriving there means the file was hand-edited and refusing the whole candidate is the honest answer rather than a quiet repair.
- **Enforcement:** `crates/settings/src/persistence.rs` (`validate_config`, `find_conflict`) rejects at save and names both fields. `crates/daemon/src/hook.rs` (`load_shortcuts`) applies the precedence order at startup and warns once. `crates/daemon/src/config.rs` (`validate`) carries `RejectReason::DuplicateShortcut` for the reload path, inside the existing all-or-nothing reject contract. The precedence order is the declaration order of the fields, which is what `decode_command`'s `if / else if` chain already resolves to — the rule specifies behaviour that existed unspecified, it does not add new behaviour there.

### BR-7 — The Stored Path Is Reconciled, and the Location Is Reported Not Policed

- **Rationale:** `/TR` is an absolute path frozen at the moment auto-start was switched on, and `AD-13` fixes it that way on purpose — an absolute path with no working directory is what makes the action un-hijackable. The cost of freezing it is that it can outlive the file it named, and existence of the task says nothing about whether it still points anywhere useful. Left alone the failure is silent and it points the wrong way: a user who does the right thing and moves the binary into `%ProgramFiles%` keeps a logon task aimed at wherever they ran it from first, which is typically `Downloads` or a build directory. So the elevated, promptless launch keeps happening — from the user-writable path, not the protected one.

  That makes the install location a privilege boundary, and `SECURITY.md` had been asking the reader to hold it by hand. Reading it is cheap, so it is read. Acting on it is where the rule stops: **warning is the ceiling, refusal is out of bounds.** Building this project produces a binary in `target\release`, which is exactly the shape the check condemns; a guard that blocks the maintainer's own workflow is one that gets disabled, and a disabled guard protects nobody. The owner is told, and the decision stays theirs.

- **Enforcement:** `crates/daemon/src/autostart.rs` — `refresh_registered_path` re-registers from `current_exe()` at startup when a task exists, relying on the `/F` already in `create_args`; it runs before `legacy::migrate_scheduled_task`, which returns early when a task exists, so exactly one of the two ever writes. It rewrites rather than reading the stored path back, because `schtasks /FO LIST` labels are localised and `/XML` emits UTF-16 through a pipe — a comparison that silently stops matching on a translated Windows would be worse than none. `is_registered` keeps its exact-existence meaning per BR-4. `crates/daemon/src/acl.rs` reads the DACL of the image and its directory (`GetNamedSecurityInfoW`, `GetAce`), returning three verdicts so that "unreadable" is never mistaken for "safe", and `warn_if_location_replaceable` raises the Tier-2 warning at daemon startup and on the tray toggle turning auto-start on. Two distinct right masks are load-bearing: bit `0x0004` is `FILE_APPEND_DATA` on a file but `FILE_ADD_SUBDIRECTORY` on a directory, and Windows grants the latter to `Authenticated Users` on `C:\`, so a single mask reports the drive root as unsafe.

## Retired

*(No retired business rules)*
