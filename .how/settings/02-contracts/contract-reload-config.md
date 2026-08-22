---
type: contract
component: settings
lc: LC-config-writer
direction: exposed
created: 2026-08-21
updated: 2026-08-21
---

# Contract — Config reload signal

## Source of truth

`crates/shared/src/constants.rs` — `WM_APP_RELOAD_CONFIG`, `DAEMON_WINDOW_CLASS`, `DAEMON_WINDOW_TITLE`

## Purpose

Notify the daemon that `config.toml` was written atomically and may be re-read. Realizes UC-4 save path.

## Operations

| Operation | Purpose | Realizes |
| --- | --- | --- |
| `PostMessageW(FindWindowW(...), WM_APP_RELOAD_CONFIG, 0, 0)` | Signal reload | UC-4 step 6 |

## Error behaviour

| Condition | Response | Caller should |
| --- | --- | --- |
| Daemon window not found | `FindWindowW` returns null; reload not sent | Log warning; user may restart daemon |
| PostMessage fails | `signal_reload` returns `false`; not retried | Report it; the file is already written, so the daemon picks the change up on its next start. A retry is deliberately absent — the post is a nudge to re-read, not the delivery of the change |
| Partial write before signal | Prevented by module ordering in `persistence.rs` | N/A — signal is last |

## Compatibility

Changing `WM_APP_RELOAD_CONFIG` value requires synchronized update in `shared` and both binaries.

## Constraints

- Must run after atomic replace of `config.toml` completes.
- No payload in `wParam`/`lParam`; daemon reads file from known path only (AD-5).
