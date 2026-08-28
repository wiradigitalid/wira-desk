---
type: contract
component: settings
lc: LC-config-writer
direction: exposed
created: 2026-08-21
updated: 2026-08-28
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
| PostMessage fails | `signal_reload` reports `Refused`; not retried | Report it; the file is already written, so the daemon picks the change up on its next start. A retry is deliberately absent — the post is a nudge to re-read, not the delivery of the change |
| **Sender below the daemon's integrity level (UIPI)** | Windows discards the post silently; `PostMessageW` returns 0, indistinguishable at the call site from any other refusal | Report it as a **refusal**, never as "the daemon is not running". The two look identical to the sender and mean opposite things to the user, and merging them is what let this defect survive: Settings launched from Explorer saved to disk and never reached the daemon, under a status line that read like success |
| Partial write before signal | Prevented by module ordering in `persistence.rs` | N/A — signal is last |

## Compatibility

Changing `WM_APP_RELOAD_CONFIG` value requires synchronized update in `shared` and both binaries.

## Constraints

- Must run after atomic replace of `config.toml` completes.
- No payload in `wParam`/`lParam`; daemon reads file from known path only (AD-5).
- **The daemon MUST admit this message through UIPI** with `ChangeWindowMessageFilterEx(hwnd,
  WM_APP_RELOAD_CONFIG, MSGFLT_ALLOW, ...)` on its hidden window, and the same for
  `WM_APP_CAPTURE_LEASE`. Without it the contract holds only when the sender is itself elevated,
  which is true of Settings launched from the tray and false of Settings launched from Explorer —
  the same binary, two integrity levels, two behaviours.

  Widening the filter for these two messages grants nothing new. `WM_APP_RELOAD_CONFIG` carries no
  payload and only asks the daemon to re-read a file any standard user can already write.
  `WM_APP_CAPTURE_LEASE` is already validated against the foreground window's process id on the
  daemon side (`DEC-004`, `DEF-3`), so admitting the message does not admit the lease. The
  precedent is the `TaskbarCreated` filter already in `tray.rs`, added for this same boundary.
