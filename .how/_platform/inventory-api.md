# API & IPC Inventory

Wira Desk contains no HTTP, REST, GraphQL, or RPC network APIs. All inter-process and inter-thread boundaries use Win32 IPC, OS messages, and in-memory lock-free channels.

## IPC & Message Interfaces

| Interface / Message | Code / Identifier | Owner | Source → Destination | Transport | Payload & Parameters | Semantics & Handling |
| --- | --- | --- | --- | --- | --- | --- |
| **Config Reload IPC** | `WM_APP_RELOAD_CONFIG`<br/>(`0x8001`) | `_platform` | `settings` → `daemon` | Win32 `SendMessageW` / `PostMessageW` to hidden daemon window (`WiraDeskDaemonHiddenWindow`) | `wParam = 0`, `lParam = 0` | Triggers immediate file re-read of `%APPDATA%\WiraDesk\config.toml` by daemon main thread. |
| **Command Ready (Internal)** | `WM_APP_COMMAND_READY`<br/>(`0x8002`) | `window-management` | `Hook Thread` → `Worker Thread` | Win32 `PostThreadMessageW` or event wake-up | `wParam = 0`, `lParam = 0` | Wakes the worker thread loop to drain commands from the ring buffer. |
| **Hook Dead Escalation** | `WM_APP_HOOK_DEAD`<br/>(`0x8003`) | `window-management` | `health.rs` → Main Thread | Win32 message loop dispatch | `wParam = 0`, `lParam = 0` | Transitions tray state machine to Tier 3 Critical (Red X) and triggers 1x toast notification. |
| **Log Warning Alert** | `WM_APP_LOG_WARNING`<br/>(`0x8004`) | `window-management` | Logger → Main Thread | Win32 message loop dispatch | `wParam = 0`, `lParam = 0` | Transitions tray icon to Tier 2 Warning (Red Dot overlay). |
| **Hook Check / Heartbeat** | `WM_APP_HOOK_CHECK`<br/>(`0x8005`) | `window-management` | `health.rs` → Hook Thread | Win32 `PostThreadMessageW` | `wParam = 0`, `lParam = 0` | Instructs the Hook Thread to verify active hook state and increment health counters. |
| **Hook Thread Ready** | `WM_APP_HOOK_READY`<br/>(`0x8006`) | `window-management` | `Hook Thread` → Main Thread | Win32 message loop dispatch | `wParam = thread_id`, `lParam = 0` | Reports successful hook installation and thread initialization. |
| **Hook Init Failed** | `WM_APP_HOOK_INIT_FAILED`<br/>(`0x8007`) | `window-management` | `Hook Thread` → Main Thread | Win32 message loop dispatch | `wParam = error_code`, `lParam = 0` | Reports fatal hook initialization failure (escalates to Tier 1 fatal popup). |
| **Hook Config Snapshot** | `WM_APP_CONFIG_SNAPSHOT`<br/>(`0x801E`) | `window-management` | Main Thread → Hook Thread | Win32 `PostThreadMessageW` | `lParam = Box::into_raw(HookSnapshot)` | Transfers owned, immutable snapshot of bypass rules and shortcut mappings to Hook Thread. |
| **Ring Buffer Command** | `Command` enum (`0..=5`) | `window-management` | `Hook Thread` → `Worker Thread` | In-process lock-free static ring buffer (`16` slots) | Raw `u8` byte (`1`=Cycle, `2`=SnapLeft, `3`=SnapRight, `4`=SnapMax, `5`=Stack) | Fire-and-forget command transfer with zero heap allocation. |
| **Settings Spawn** | Executable invocation | `_platform` | `daemon` → `settings` | Win32 `ShellExecuteW` | Path: `wiradesk-settings.exe`, optional arg: `--onboarding` | Spawns GUI process on demand inheriting elevation. |

## External Operating System APIs

| OS API Category | Win32 API Functions | Consumer | Purpose |
| --- | --- | --- | --- |
| **Keyboard Hooks** | `SetWindowsHookExW`, `UnhookWindowsHookEx`, `CallNextHookEx` | `daemon::hook` | Low-level global keyboard interception (`WH_KEYBOARD_LL`). |
| **Window Enumeration** | `EnumWindows`, `IsWindowVisible`, `GetWindowLongPtrW`, `GetClassNameW` | `daemon::worker`, `daemon::cycling` | Live, non-blocking Z-order traversal and candidate filtering. |
| **Process Identity** | `GetWindowThreadProcessId`, `OpenProcess`, `QueryFullProcessImageNameW` | `daemon::cycling` | Multi-process executable filename matching (AD-4). |
| **Focus & Positioning** | `SetForegroundWindow`, `SetWindowPos`, `ShowWindowAsync` | `daemon::worker`, `daemon::arrangement` | Active window focus switching and DPI-aware snapping. |
| **Virtual Desktops** | `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` | `daemon::context` | Isolation of cycling within the active virtual desktop (AD-9). |
| **Tray & Notification** | `Shell_NotifyIconW`, `RegisterWindowMessageW("TaskbarCreated")` | `daemon::tray` | Tray icon lifecycle, status overlays, and explorer restart recovery (AD-10). |
| **Task Scheduler** | `schtasks.exe /Create /Query /Delete` | `daemon::autostart`, `settings::app` | Elevated logon auto-start management (AD-13). |
| **UI Automation (a11y)** | AccessKit Windows backend | `settings` | Screen reader accessibility tree publication for egui controls (AD-11a). |
