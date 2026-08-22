# Contract inventory — window-management

| # | Contract | Direction | Boundary | LC |
| --- | --- | --- | --- | --- |
| 1 | *(In-process Ring Buffer)* | internal | `ring::push(u8)` / `ring::pop()` atomic queue + `WM_APP_COMMAND_READY` wake-up | `LC-hook-thread` → `LC-worker-thread` |
| 2 | `contract-reload-config.md` | consumed (caller: settings) | `WM_APP_RELOAD_CONFIG` → daemon hidden window (`WiraDeskDaemonHiddenWindow`) | `LC-tray-controller` |
| 3 | `contract-hook-heartbeat.md` | internal | `WM_APP_HOOK_CHECK` / `WM_APP_HOOK_DEAD` / `WM_APP_HOOK_REFRESH_OK` / `WM_APP_HOOK_SHUTDOWN` | `health::heartbeat` ↔ `LC-hook-thread` ↔ `LC-tray-controller` |

This component exposes no external network or cross-process IPC endpoints directly. It consumes the platform IPC message `WM_APP_RELOAD_CONFIG` (`shared::constants::WM_APP_RELOAD_CONFIG = 0x8001`) from `settings` via the hidden host window, and communicates internally via a static 16-slot lock-free ring buffer and thread message queues.
