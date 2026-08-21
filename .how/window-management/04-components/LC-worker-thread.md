---
type: lc
id: LC-worker-thread
name: Worker Thread
lc_type: service
container: daemon
component: window-management
owner: Wira Desk Core
area: execution-core
created: 2026-08-21
---

# LC-worker-thread — Worker Thread

## Responsibility

`LC-worker-thread` is the core execution actor running on the daemon's main thread. It is responsible for:
1. Draining atomic `u8` wire command bytes (`shared::Command`: `Cycle`, `SnapLeft`, `SnapRight`, `SnapMaximize`, `OverlappingStack`) from the static 16-slot lock-free ring buffer (`ring::pop()`) upon receiving `WM_APP_COMMAND_READY` wake-up messages.
2. Executing stateless live Z-order window enumeration via Win32 `EnumWindows` (`crates/daemon/src/cycling/source.rs`, AD-3) without maintaining internal window caches or trees.
3. Evaluating candidate window eligibility against frozen rules (`crates/daemon/src/cycling/eligibility.rs`): excluding hidden windows, shell surfaces (`Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `Progman`, `WorkerW`), cloaked compositor windows (`DwmGetWindowAttribute`), iconic/minimized windows, `WS_EX_TOOLWINDOW`, ghost windows (`Ghost`), and processes not matching the active application's executable basename (`QueryFullProcessImageNameW`, AD-4).
4. Enforcing multi-monitor and virtual desktop spatial containment (`crates/daemon/src/context/`): querying `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` (AD-9) and restricting candidates to the active foreground monitor, failing closed when COM is unavailable.
5. Selecting target windows using least-recently-used (LRU) reversal order (`cycle_order`) across the live Z-order, preventing 2-window oscillation loops on 3+ window stacks.
6. Activating chosen windows via `SetForegroundWindow` (`Win32Activator`) with confirmation polling.
7. Injecting unassigned `VK_NONAME` (`suppress_start_menu()`) while the `Win` key is held, preventing the Windows Start Menu from popping upon key release without swallowing modifier key-ups (preventing sticky-modifier bugs).
8. Dispatching snap and stack layout commands to `LC-arrangement-engine` (`snap::plan_snap_*`, `stack::plan_stack`), applying computed placements via `Win32WindowMover` (`SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`).
9. Managing Worker configuration snapshots (`WorkerSnapshot`) installed via `WM_APP_RELOAD_CONFIG`.

## Depends on

- `LC-arrangement-engine` — geometry planning for half-screen snaps and overlapping cascade stacks.
- `crates/daemon/src/ring.rs` — lock-free static 16-slot ring buffer for polling command bytes.
- `crates/daemon/src/cycling/` (`source.rs`, `eligibility.rs`, `selection.rs`, `activation.rs`) — window candidate enumeration, filtering, LRU ordering, and foreground activation.
- `crates/daemon/src/context/` (`virtual_desktop.rs`, `spatial.rs`) — `IVirtualDesktopManager` COM isolation and monitor work area resolution.
- `shared::Command`, `shared::Config`, `shared::config::LayoutConfig` — shared command opcodes and configuration structures.
- Windows Subsystems: User32 (`EnumWindows`, `GetForegroundWindow`, `SetForegroundWindow`, `GetWindowLongPtrW`, `IsWindowVisible`, `GetWindowPlacement`, `SendInput`, `GetAsyncKeyState`), Shell32 (`IVirtualDesktopManager`), DWM (`DwmGetWindowAttribute`), Kernel32 (`OpenProcess`, `QueryFullProcessImageNameW`, `CloseHandle`).

## Interface

### Inbound Signals & Message Triggers
- `WM_APP_COMMAND_READY`: Signals `drain_commands()` to drain and process all pending commands in the ring buffer.
- `install_config_snapshot(snapshot: WorkerSnapshot)`: Updates thread-local `WORKER_CONFIG` upon configuration reload (`WM_APP_RELOAD_CONFIG`).

### Execution Methods
- `drain_commands()`: Drains the static ring buffer and dispatches to `execute_cycle()`, `execute_snap()`, or `execute_stack()`.
- `execute_cycle()`: Captures active context & spatial bounds, drives `run_context_safe_cycle()`, activates target, and suppresses Start menu.
- `execute_snap(command)`: Resolves monitor context and applies single-window snap geometry (`PlacementPlan`).
- `execute_stack()`: Plans and applies overlapping cascade stack for up to 3 live same-app windows on the origin monitor.

### Outbound Win32 Calls
- `SetForegroundWindow(hwnd)`: Transitions focus to target window.
- `SetWindowPos(hwnd, 0, x, y, cx, cy, SWP_NOACTIVATE | SWP_NOZORDER)`: Updates window geometry without focus disruption.
- `SendInput(...)` with `VK_NONAME`: Injects neutral keystroke to prevent Start menu activation.

## Notes

- **LRU Z-Order Traversal:** Naive "next in Z-order" (first candidate below foreground) causes a ping-pong bug between top 2 windows in a 3+ window stack because raising a window puts it at top. Reversing the rotated slice (`cycle_order`) ensures deterministic full-cycle rotation.
- **Worker-Thread COM Apartment:** `VirtualDesktopManager` is wrapped in `thread_local!` to guarantee COM apartment ownership and single-threaded initialization once per worker lifecycle, avoiding ~19 ms overhead per keystroke.
- **Evidence:** Verified against `crates/daemon/src/worker.rs`, `crates/daemon/src/cycling/`, `crates/daemon/src/context/`, and `crates/daemon/src/arrangement/win32.rs`.
