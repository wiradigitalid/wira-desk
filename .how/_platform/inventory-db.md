---
type: inventory
kind: db
derived_from: code
---

# Data & Persistence Inventory

Wira Desk does not use an RDBMS or embedded SQL database. All persistence uses local files and in-memory runtime data structures.

## Persistent Stores

| Store | Format | Owner | Path / Location | Schema & Keys | Lifetime & Access Pattern |
| --- | --- | --- | --- | --- | --- |
| **User Configuration** | TOML | `_platform` (shared) | `%APPDATA%\WiraDesk\config.toml` | **Sections:**<br/>- `[general]`: `auto_start` (bool)<br/>- `[switcher]`: `shortcut`, `fallback_shortcut`<br/>- `[snapping]`: `snap_half_left`, `snap_half_right`, `snap_maximize`<br/>- `[layout]`: `enable_overlapping_stack`, `stack_width_percent`, `stack_shortcut`<br/>- `[vm_bypass]`: `bypass_processes` (vec), `bypass_classes` (vec) | Written atomically by `settings` process via temp file + rename. Read on startup and on `WM_APP_RELOAD_CONFIG` by `daemon`. |
| **Diagnostic Log** | Plaintext (append-only) | `_platform` (shared) | `%APPDATA%\WiraDesk\wiradesk.log` | Formatted log lines: `[YYYY-MM-DD HH:MM:SS.mmm] [LEVEL] Message` | Append-only. Written by `daemon` on warnings/errors. Opened as a file in `notepad.exe` via tray "View Logs". |
| **Legacy Config (Migration)** | TOML | `_platform` (shared) | `%APPDATA%\WinTick\config.toml` | Legacy schema (WinTick keys). | Read-only during one-time bootstrap migration if `%APPDATA%\WiraDesk\config.toml` does not yet exist. |

## In-Memory Runtime State

| State Structure | Owner | Container | Concurrency & Access | Notes |
| --- | --- | --- | --- | --- |
| **Command Ring Buffer** | `window-management` | `daemon` | Lock-free, Single-Producer Single-Consumer (SPSC), `16` slots of `u8`. | Hook Thread pushes commands; Worker Thread drains and executes. |
| **Tray Icon State Machine** | `window-management` | `daemon` | Mutex / message-loop owned (`Normal`, `Warning`, `Critical`). | Reflects 3-Tier error protocol visually. |
| **Hook Health Status** | `window-management` | `daemon` | Atomic / 10s timer state in `health.rs`. | Heartbeat counter tracking consecutive hook check failures. |
| **Settings Working Copy** | `settings` | `settings` | UI thread mutable state (`Config` struct). | Edits are held in GUI memory until the user clicks Save / Applies changes. |

## Rows

| No | Table | Owning component | What it holds | Key columns | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | `config.toml [general]` | `_platform` | Persisted user configuration for general | `auto_start` | active |
| 2 | `config.toml [switcher]` | `_platform` | Persisted user configuration for switcher | `shortcut`, `fallback_shortcut` | active |
| 3 | `config.toml [snapping]` | `_platform` | Persisted user configuration for snapping | `snap_half_left`, `snap_half_right`, `snap_maximize` | active |
| 4 | `config.toml [layout]` | `_platform` | Persisted user configuration for layout | `enable_overlapping_stack`, `stack_width_percent`, `stack_shortcut` | active |
| 5 | `config.toml [vm_bypass]` | `_platform` | Persisted user configuration for vm bypass | `bypass_processes`, `bypass_classes` | active |
