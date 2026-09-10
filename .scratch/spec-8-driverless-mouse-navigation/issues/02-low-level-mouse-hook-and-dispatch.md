---
id: SPEC-8-02
component: window-management
satisfies: [UC-13, FR-30, FR-31]
blocked_by: [SPEC-8-01]
status: done
tests:
  - commands::tests::frozen_command_wire_values_extended_for_mouse
  - hook::tests::mouse_hook_passes_mousemove_without_interception
  - hook::tests::mouse_hook_swallows_mapped_xbuttons_down_and_up
  - hook::tests::unmapped_mouse_buttons_pass_through
  - hook::tests::vm_bypass_passes_mouse_events_through
  - hook::tests::tilt_wheel_debounce_drops_rapid_burst_ticks_at_150ms
  - hook::tests::tilt_wheel_satisfying_debounce_enqueues_command
  - hook::tests::ring_full_increments_dropped_metric_on_mouse_event
  - hook::tests::dual_hook_heartbeat_refreshes_both_handles
  - worker::tests::mouse_virtual_desktop_commands_dispatch_safely
---

# 02: Low-level mouse hook, dual lifecycle, tilt debounce, and worker dispatch

**What to build:** Extend the frozen `Command` wire protocol with desktop navigation opcodes (16–19), install `WH_MOUSE_LL` co-located on the hook thread, handle dual-hook lifecycle and heartbeat refresh, implement fast-path `WM_MOUSEMOVE` passthrough, evaluate VM/RDP bypass, debounce horizontal tilt-wheel signals at exactly 150 ms, enqueue mapped commands to the ring buffer, and execute actions (including elevated `SendInput` with `suppress_start_menu()`) on the worker actor.

**Blocked by:** `SPEC-8-01` (requires `shared::Config.mouse` schema and `MouseActionPreset`)

## Acceptance Criteria

- [x] `shared::Command` opcodes extended with new values:
      - `16 = NextVirtualDesktop`
      - `17 = PrevVirtualDesktop`
      - `18 = TaskView`
      - `19 = ShowDesktop`
      Wire values 0–15 remain frozen. The test `frozen_command_wire_values` is updated to verify values 0–19.
- [x] `HookSnapshot` and `HookRuntime` extended:
      - `HookRuntime` stores both `keyboard_hook_handle: HHOOK` and `mouse_hook_handle: HHOOK`.
      - `HookRuntime` stores `last_tilt_ms: u64` (timestamp of last accepted tilt event).
      - `HookSnapshot` carries `mouse: MouseMapping` resolved from `shared::Config.mouse`.
- [x] `HookThread` installs both `WH_KEYBOARD_LL` and `WH_MOUSE_LL` during startup; `refresh_hook_on_hook_thread` verifies and refreshes both handles on 10s heartbeat checks.
- [x] At the entry of `low_level_mouse_proc`:
      `if n_code < 0 || msg == WM_MOUSEMOVE { return CallNextHookEx(0, n_code, w_param, l_param); }`
      runs within 1–2 CPU instructions with zero locks, heap allocations, or logging (`LBR-WM-11`, `NFR-2`).
- [x] VM/RDP bypass check evaluated for auxiliary mouse inputs: if the foreground window matches `BypassPolicy`, calls `CallNextHookEx` immediately without interception.
- [x] Input extraction and mapping:
      - For `WM_XBUTTONDOWN`: resolves button from `mouseData >> 16` (`XBUTTON1` = back, `XBUTTON2` = forward).
      - For `WM_MOUSEHWHEEL`: extracts signed 16-bit delta via `((mouse_data >> 16) & 0xFFFF) as i16` (negative = left, positive = right).
- [x] Tilt-wheel debounce:
      - A constant `TILT_WHEEL_DEBOUNCE_MS = 150` is enforced.
      - Rapid hardware burst ticks arriving within 150 ms of `last_tilt_ms` are swallowed (`return 1`) without enqueuing duplicate commands to the ring buffer (`SCN-04`).
- [x] Swallowing semantics:
      - Mapped and debounced events are swallowed (`return 1`).
      - For `WM_XBUTTONUP`: swallowed (`return 1`) if the corresponding button is mapped, preventing application-level state confusion; passed through if unmapped.
      - Unmapped inputs or actions set to `"passthrough"` invoke `CallNextHookEx`.
- [x] Ring buffer capacity:
      - When ring buffer is full, increments `DROPPED_FULL` metric and returns `1` without blocking the hook thread (`LBR-WM-4`).
- [x] Worker execution in `crates/daemon/src/worker.rs`:
      - Drains opcodes 16–19 and dispatches via `execute_mouse_navigation(cmd)`:
        - `NextVirtualDesktop`: `SendInput(Ctrl + Win + Right)` + `suppress_start_menu()`.
        - `PrevVirtualDesktop`: `SendInput(Ctrl + Win + Left)` + `suppress_start_menu()`.
        - `TaskView`: `SendInput(Win + Tab)` + `suppress_start_menu()`.
        - `ShowDesktop`: `SendInput(Win + D)` + `suppress_start_menu()`.
        - Cycling and snapping opcodes dispatch through existing `execute_cycle()` and `execute_snap()`.
- [x] Clean shutdown: `UnhookWindowsHookEx` called for both keyboard and mouse hook handles.
- [x] Pure helper functions (`resolve_mouse_event`, `apply_tilt_debounce`, `map_preset_to_command`) extracted for unit testing.
- [x] All unit tests pass in `crates/daemon`.
