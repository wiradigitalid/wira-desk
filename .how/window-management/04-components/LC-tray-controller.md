---
type: lc
id: LC-tray-controller
name: Tray Controller
lc_type: service
container: daemon
component: window-management
owner: Wira Desk Core
area: user-visible-health
created: 2026-08-21
---

# LC-tray-controller — Tray Controller

## Responsibility

`LC-tray-controller` owns every user-visible health signal outside the settings window. It runs on the daemon message loop thread and is responsible for:

1. Registering and maintaining the notification area icon (`Shell_NotifyIconW` with `NIM_ADD` / `NIM_MODIFY` / `NIM_DELETE`).
2. Rendering the three-tier visual health protocol (AD-7): normal icon, warning dot overlay (Tier 2), and critical X overlay (Tier 3).
3. Building and displaying the tray context menu in the order mandated by FR-16: an "Update to `<version>`..." item shown only when `updatecheck::snapshot()` reports one is available, then Settings, View Logs, Auto-Start, About, Exit. Clicking either the update item or Settings launches Settings, which is where the update is actually read about and installed.
4. Listening for the shell broadcast `TaskbarCreated` and re-registering the icon without user action (AD-10, UC-3).
5. Showing exactly one balloon toast when hook death escalates to Tier 3; latching `toast_sent` so repeats are suppressed.
6. Launching `wiradesk-settings.exe` via `ShellExecute` and opening the log directory for View Logs (BR-5).
7. Showing an unconditional "Now running / Listening for shortcuts" toast on every daemon start (`WM_APP_HOOK_READY`), not only the first — a fourth, always-informational notification class alongside the three-tier error protocol.

`LC-tray-controller` never blocks on configuration I/O, never installs hooks, and never performs window enumeration.

## Depends on

- `crates/daemon/src/tray.rs` — icon lifecycle, overlay states, `TaskbarCreated` handler.
- `crates/daemon/src/menu.rs` — menu item construction and command routing.
- `crates/daemon/src/health.rs` — Tier-2/Tier-3 escalation signals from hook heartbeat failures.
- `crates/daemon/src/updatecheck.rs` — `snapshot()`, read to decide whether the menu shows an update item (CAP-13).
- `shared::constants` — window class/title for settings launch coordination.
- Windows Shell APIs: `Shell_NotifyIconW`, `TrackPopupMenu`, `ShellExecuteW`.

## Interface

### Inbound

| Signal | Source | Action |
| --- | --- | --- |
| `WM_APP_TRAY_ICON` | Shell | Route menu commands |
| `WM_TASKBARCREATED` | Explorer restart | Re-register icon (UC-3) |
| `set_tray_tier(Tier)` | `health.rs` / worker | Update overlay + optional toast |
| `WM_APP_UPDATE_STATE` | `updatecheck::spawn` | New update-check result to reflect next time the menu opens |
| Startup | `main.rs` | Initial `NIM_ADD` |

### Outbound

| Action | Target | Realizes |
| --- | --- | --- |
| `ShellExecuteW(settings.exe)` | User | FR-16 Settings item, and the update item (both launch Settings) |
| `ShellExecuteW(explorer.exe, log_dir)` | User | FR-12 View Logs |
| `PostQuitMessage` | Process | Exit menu item |
| Balloon notification | User | AD-7 Tier 3 |
| "Now running" toast | User | Every start (`WM_APP_HOOK_READY`), not tied to any FR |

## Notes

- **Explorer death:** When `explorer.exe` restarts, the tray handle is invalid until `TaskbarCreated` arrives; the daemon must not crash—UC-3 covers recovery.
- **Update menu item, deliberately not "Check for Updates":** a menu closes on click, so a check started from it would have nowhere to report to; an announcement shown only when there is something to say costs the menu nothing on the days there is nothing (almost all of them). The manual check lives in Settings' About pane instead (FR-25).
- **Evidence:** [PARTIAL] `crates/daemon/src/tray.rs`, `crates/daemon/src/menu.rs`, `crates/daemon/src/health.rs`, `crates/daemon/src/updatecheck.rs`.
