---
type: lc
id: LC-config-writer
name: Config Writer
lc_type: service
container: settings
component: settings
owner: Wira Desk Core
area: persistence-ipc
created: 2026-08-21
---

# LC-config-writer — Config Writer

## Responsibility

`LC-config-writer` is the only component allowed to mutate on-disk configuration (BR-1, BR-2). It:

1. Validates shortcut chords via `shared::shortcut::validate_shortcut` before persistence (SCN-01).
2. Performs atomic write: temp file → `replace` → fsync ordering in `persistence.rs` (LBR-ST-2, AD-5).
3. Sends `PostMessageW(FindWindowW(...), WM_APP_RELOAD_CONFIG, 0, 0)` only after the file is complete.
4. Creates or deletes the logon scheduled task via `schtasks` with `/RU %USERNAME%` and `/RL HIGHEST` (AD-13, UC-6, SCN-02).
5. Records onboarding completion flags so first-run does not repeat (LBR-ST-7).

## Depends on

- `crates/settings/src/persistence.rs` — atomic TOML write and IPC signal.
- `crates/daemon/src/autostart.rs` — shared task registration helpers (also linked from settings build).
- `shared::Config`, `shared::constants::{WM_APP_RELOAD_CONFIG, DAEMON_WINDOW_CLASS, DAEMON_WINDOW_TITLE, TASK_NAME}`.
- Win32: `FindWindowW`, `PostMessageW`.
- Windows Task Scheduler CLI: `schtasks`.

## Interface

### Inbound

| Method | Caller | Realizes |
| --- | --- | --- |
| `save(config: &Config)` | `LC-settings-shell` | UC-4 |
| `set_autostart(enabled: bool)` | `LC-settings-shell` | UC-6 |
| `write_onboarding_flags(...)` | Onboarding flow | UC-5 |

### Outbound

| Action | When |
| --- | --- |
| Atomic `config.toml` write | Before any reload signal |
| `WM_APP_RELOAD_CONFIG` | After successful write |
| `schtasks /Create` or `/Delete` | Auto-start toggle |

## Failure behaviour

| Condition | User sees | Log |
| --- | --- | --- |
| Disk permission denied | Inline save error | `error!` with path |
| Daemon window not found | Save OK, stale shortcuts until restart | `warn!` |
| `schtasks` failure | Toggle reverts (SCN-02) | `warn!` exit code |

## Notes

- **Evidence:** [PARTIAL] `crates/settings/src/persistence.rs`, `crates/daemon/src/autostart.rs`.
