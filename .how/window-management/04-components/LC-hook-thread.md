---
type: lc
id: LC-hook-thread
name: Hook Thread
lc_type: service
container: daemon
component: window-management
owner: Wira Desk Core
area: input-boundary
created: 2026-08-21
---

# LC-hook-thread — Hook Thread

## Responsibility

`LC-hook-thread` runs on a dedicated OS thread elevated to `THREAD_PRIORITY_TIME_CRITICAL` and is the sole component that installs and manages the global Win32 `WH_KEYBOARD_LL` input hook (`crates/daemon/src/hook.rs`). It is responsible for:
1. Intercepting raw low-level keyboard messages (`WM_KEYDOWN`, `WM_SYSKEYDOWN`, `WM_KEYUP`, `WM_SYSKEYUP`) delivered by Windows.
2. Tracking modifier key states (`Win`, `Ctrl`, `Alt`, `Shift`) without calling `GetAsyncKeyState`.
3. Enforcing the 50 ms anti-macro throttle window (`ANTI_MACRO_THROTTLE_MS`) using high-resolution QPC timestamps to discard noisy repeated inputs.
4. Performing bounded, allocation-free VM/RDP bypass queries against the active foreground window (`crates/daemon/src/context/vm_bypass.rs`). If matched, key chords pass through untouched with zero latency.
5. Translating valid, non-bypassed shortcut matches into atomic `u8` wire command bytes (`shared::Command`).
6. Enqueuing command bytes to the lock-free static 16-slot ring buffer (`ring::push(u8)`) and posting an asynchronous wake-up message (`WM_APP_COMMAND_READY`) to the worker thread.
7. Responding to periodic heartbeat checks (`WM_APP_HOOK_CHECK`) from `health::heartbeat` and managing hook re-registration and failure escalation.

`LC-hook-thread` never performs heap allocations during keypress processing, never invokes blocking kernel or COM APIs, never executes window enumeration, and never waits for worker thread execution.

## Depends on

- `crates/daemon/src/ring.rs` — static lock-free ring buffer for pushing `u8` commands.
- `crates/daemon/src/context/vm_bypass.rs` — allocation-free foreground identity matching using reusable stack buffers.
- `shared::constants` and `shared::Command` — message constants and wire protocol definitions.
- Windows Subsystems: User32 (`SetWindowsHookExW`, `UnhookWindowsHookEx`, `CallNextHookEx`, `PeekMessageW`, `GetMessageW`, `PostMessageW`, `TranslateMessage`, `DispatchMessageW`), Kernel32 (`QueryPerformanceCounter`, `QueryPerformanceFrequency`, `GetTickCount64`).

## Interface

### Inbound Messages (Thread Message Loop)
- `WM_APP_HOOK_CHECK` (from `health::heartbeat`): Triggers `refresh_hook_on_hook_thread()`.
- `WM_APP_CONFIG_SNAPSHOT` (from `LC-tray-controller`): Atomically collects a staged `HookSnapshot` (`PENDING_SNAPSHOT`) updating shortcut definitions and VM/RDP bypass policies.
- `WM_APP_HOOK_SHUTDOWN` (from `LC-tray-controller`): Unhooks `WH_KEYBOARD_LL` and posts `WM_QUIT` to terminate the thread loop.

### Outbound Signals
- `ring::push(u8)`: Pushes `Command` opcode (`Cycle = 1`, `SnapLeft = 2`, `SnapRight = 3`, `SnapMaximize = 4`, `OverlappingStack = 5`) into the lock-free ring buffer.
- `PostMessageW(worker_hwnd, WM_APP_COMMAND_READY, 0, 0)`: Signals the worker thread that commands are ready for draining.
- `PostMessageW(worker_hwnd, WM_APP_HOOK_READY, thread_id, 0)`: Notifies startup readiness with the hook thread ID.
- `PostMessageW(worker_hwnd, WM_APP_HOOK_INIT_FAILED, 0, 0)`: Signals fatal hook installation failure.
- `PostMessageW(worker_hwnd, WM_APP_HOOK_REFRESH_OK, 0, 0)`: Signals successful hook refresh after heartbeat verification.
- `PostMessageW(worker_hwnd, WM_APP_HOOK_DEAD, 0, 0)`: Signals 3 consecutive hook refresh failures to trigger Tier-3 Critical error escalation.

## Notes

- **Sticky Modifier Prevention:** Swallowing `Win` key releases causes Windows to believe `Win` is permanently pressed, corrupting subsequent input. `LC-hook-thread` specifically passes all modifier `key_up` events (`VK_LWIN`, `VK_RWIN`, `VK_LCONTROL`, `VK_LMENU`, `VK_LSHIFT`) to `CallNextHookEx`, while swallowing only the main chord key down/up events (`VK_BACKTICK` or configured key).
- **Pointer Provenance & Soundness:** The Hook thread's runtime address is published to a static `AtomicPtr<HookRuntime>` using `&raw mut` without creating mutable aliases. Windows guarantees callback delivery on the hook thread during message retrieval, preventing concurrent access and data races.
- **Evidence:** Verified against `crates/daemon/src/hook.rs`, `crates/daemon/src/ring.rs`, and `crates/daemon/src/context/vm_bypass.rs`.
