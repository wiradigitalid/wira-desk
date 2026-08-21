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
3. Building and displaying the tray context menu in the order mandated by FR-16: Settings, View Logs, Auto-Start, Check for Updates, About, Exit.
4. Listening for the shell broadcast `TaskbarCreated` and re-registering the icon without user action (AD-10, UC-3).
5. Showing exactly one balloon toast when hook death escalates to Tier 3; latching `toast_sent` so repeats are suppressed.
6. Launching `wiradesk-settings.exe` via `ShellExecute` and opening the log directory for View Logs (BR-5).

`LC-tray-controller` never blocks on configuration I/O, never installs hooks, and never performs window enumeration.

## Depends on

- `crates/daemon/src/tray.rs` — icon lifecycle, overlay states, `TaskbarCreated` handler.
- `crates/daemon/src/menu.rs` — menu item construction and command routing.
- `crates/daemon/src/health.rs` — Tier-2/Tier-3 escalation signals from hook heartbeat failures.
- `shared::constants` — window class/title for settings launch coordination.
- Windows Shell APIs: `Shell_NotifyIconW`, `TrackPopupMenu`, `ShellExecuteW`.

## Interface

### Inbound

| Signal | Source | Action |
| --- | --- | --- |
| `WM_APP_TRAY_ICON` | Shell | Route menu commands |
| `WM_TASKBARCREATED` | Explorer restart | Re-register icon (UC-3) |
| `set_tray_tier(Tier)` | `health.rs` / worker | Update overlay + optional toast |
| Startup | `main.rs` | Initial `NIM_ADD` |

### Outbound

| Action | Target | Realizes |
| --- | --- | --- |
| `ShellExecuteW(settings.exe)` | User | FR-16 Settings item |
| `ShellExecuteW(explorer.exe, log_dir)` | User | FR-12 View Logs |
| `PostQuitMessage` | Process | Exit menu item |
| Balloon notification | User | AD-7 Tier 3 |

## Notes

- **Explorer death:** When `explorer.exe` restarts, the tray handle is invalid until `TaskbarCreated` arrives; the daemon must not crash—UC-3 covers recovery.
- **Evidence:** [PARTIAL] `crates/daemon/src/tray.rs`, `crates/daemon/src/menu.rs`, `crates/daemon/src/health.rs`.
