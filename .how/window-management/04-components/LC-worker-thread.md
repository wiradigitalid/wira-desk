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
1. Draining atomic `u8` wire command bytes (`shared::Command`: `Cycle`, `SnapLeft`, `SnapRight`, `SnapMaximize`, `OverlappingStack`, `SnapTop`, `SnapBottom`, `MoveToNextMonitor`) from the static 16-slot lock-free ring buffer (`ring::pop()`) upon receiving `WM_APP_COMMAND_READY` wake-up messages.
2. Executing stateless live Z-order window enumeration via Win32 `EnumWindows` (`crates/daemon/src/cycling/source.rs`, AD-3) without maintaining internal window caches or trees.
3. Evaluating candidate window eligibility against frozen rules (`crates/daemon/src/cycling/eligibility.rs`): excluding hidden windows, shell surfaces (`Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `Progman`, `WorkerW`), cloaked compositor windows (`DwmGetWindowAttribute`), iconic/minimized windows, `WS_EX_TOOLWINDOW`, ghost windows (`Ghost`), and processes not matching the active application's executable basename (`QueryFullProcessImageNameW`, AD-4).
4. Enforcing multi-monitor and virtual desktop spatial containment (`crates/daemon/src/context/`): querying `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` (AD-9) and restricting candidates to the active foreground monitor, failing closed when COM is unavailable.
5. Selecting target windows using least-recently-used (LRU) reversal order (`cycle_order`) across the live Z-order, preventing 2-window oscillation loops on 3+ window stacks.
6. Activating chosen windows (`Win32Activator`) via `SetForegroundWindow`, confirmed by polling for up to ~20 ms; when refused, attaching this thread's input queue to the actual foreground thread (not the target — the reverse attachment an earlier revision made) and retrying once, ~40 ms worst case. One activation attempt per target either way — no responsiveness probe, restore, or per-target retry loop, so a hung window gets the same treatment as a healthy one.
7. Injecting unassigned `VK_NONAME` (`suppress_start_menu()`) while the `Win` key is held, preventing the Windows Start Menu from popping upon key release without swallowing modifier key-ups (preventing sticky-modifier bugs).
8. Dispatching snap and stack layout commands to `LC-arrangement-engine` (`snap::plan_snap_*`, `stack::plan_stack`, `monitor::plan_move_to_monitor`), applying computed placements via `Win32WindowMover` (`SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`). A maximize command first tries a native `ShowWindowAsync(SW_MAXIMIZE)` and falls back to the geometric plan only when the window's style forbids maximizing.
9. Managing Worker configuration snapshots (`WorkerSnapshot`) installed via `WM_APP_RELOAD_CONFIG`.

## Depends on

- `LC-arrangement-engine` — geometry planning for half-screen snaps and overlapping cascade stacks.
- `crates/daemon/src/ring.rs` — lock-free static 16-slot ring buffer for polling command bytes.
- `crates/daemon/src/cycling/` (`source.rs`, `eligibility.rs`, `selection.rs`, `activation.rs`) — window candidate enumeration, filtering, LRU ordering, and foreground activation (direct call plus one `AttachThreadInput` retry).
- `crates/daemon/src/context/` (`virtual_desktop.rs`, `spatial.rs`) — `IVirtualDesktopManager` COM isolation and monitor work area resolution.
- `shared::Command`, `shared::Config`, `shared::config::LayoutConfig` — shared command opcodes and configuration structures.
- Windows Subsystems: User32 (`EnumWindows`, `GetForegroundWindow`, `SetForegroundWindow`, `GetWindowLongPtrW`, `IsWindowVisible`, `GetWindowPlacement`, `SendInput`, `GetAsyncKeyState`), Shell32 (`IVirtualDesktopManager`), DWM (`DwmGetWindowAttribute`), Kernel32 (`OpenProcess`, `QueryFullProcessImageNameW`, `CloseHandle`).

## Interface

### Inbound Signals & Message Triggers
- `WM_APP_COMMAND_READY`: Signals `drain_commands()` to drain and process all pending commands in the ring buffer.
- `install_config_snapshot(snapshot: WorkerSnapshot)`: Updates thread-local `WORKER_CONFIG` upon configuration reload (`WM_APP_RELOAD_CONFIG`).

### Execution Methods
- `drain_commands()`: Drains the static ring buffer and dispatches to `execute_cycle()`, `execute_snap()`, `execute_stack()`, or `execute_monitor_move()`.
- `execute_cycle()`: Captures active context & spatial bounds, drives `run_context_safe_cycle()`, activates target, and suppresses Start menu.
- `execute_snap(command)`: Resolves monitor context and applies single-window snap geometry (`PlacementPlan`); `SnapMaximize` tries a native maximize first, falling back to the geometric plan.
- `execute_stack()`: Plans and applies overlapping cascade stack for up to 3 live same-app windows on the origin monitor.
- `execute_monitor_move()`: Enumerates the live monitor set, maps the window's share of its source work area onto the destination, and applies the plan — an empty plan on a single-monitor machine is a successful no-op (`LBR-WM-7`, `DEC-007`, `DEC-010`).

### Outbound Win32 Calls
- `SetForegroundWindow(hwnd)`: Transitions focus to target window.
- `AttachThreadInput(current, foreground, TRUE/FALSE)` + retry `SetForegroundWindow`: Fallback path when the direct call is refused — attaches to the actual foreground thread's input queue, retries once, then detaches.
- `SetFocus(hwnd)`: Best-effort call after a successful `AttachThreadInput` retry.
- `ShowWindowAsync(hwnd, SW_MAXIMIZE)`: Native maximize, tried before the geometric plan for `SnapMaximize`.
- `SetWindowPos(hwnd, 0, x, y, cx, cy, SWP_NOACTIVATE | SWP_NOZORDER)`: Updates window geometry without focus disruption.
- `SendInput(...)` with `VK_NONAME`: Injects neutral keystroke to prevent Start menu activation.

## Notes

- **LRU Z-Order Traversal:** Naive "next in Z-order" (first candidate below foreground) causes a ping-pong bug between top 2 windows in a 3+ window stack because raising a window puts it at top. Reversing the rotated slice (`cycle_order`) ensures deterministic full-cycle rotation.
- **Worker-Thread COM Apartment:** `VirtualDesktopManager` is wrapped in `thread_local!` to guarantee COM apartment ownership and single-threaded initialization once per worker lifecycle, avoiding ~19 ms overhead per keystroke.
- **Activation retry, and why it exists:** `SetForegroundWindow` is refused by Windows unless the caller already owns the foreground or is otherwise privileged, so a direct call alone silently fails roughly every other cycle. The fallback attaches this thread's input queue to the thread that *owns* the foreground window and retries once — attaching to the target thread instead, an earlier revision's mistake, leaves the caller with no rights at all. Every wait in this path (~20 ms per attempt, ~40 ms worst case across both) is on Windows applying the focus change, never on the target application responding, which is what keeps a hung window's treatment identical to a responsive one's.
- **Evidence:** Verified against `crates/daemon/src/worker.rs`, `crates/daemon/src/cycling/` (including `activation.rs`), `crates/daemon/src/context/`, and `crates/daemon/src/arrangement/` (`win32.rs`, `monitor.rs`).
