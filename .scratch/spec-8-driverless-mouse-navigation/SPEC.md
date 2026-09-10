---
spec: SPEC-8
release: "0.3.0"
prd: wira-desk
fr: [FR-30, FR-31, FR-32]
status: ready-for-agent
---

# SPEC-8 — Driverless Mouse Desktop Navigation

## Problem Statement

Standard productivity mice (such as Logitech M-series, MX Master, and similar ergonomic multi-button office mice) provide physical auxiliary controls — two thumb buttons and a horizontal tilt wheel — that default to generic browser back/forward and horizontal panning. Because office mice almost universally lack on-board non-volatile memory (EEPROM) for hardware-level macro storage, unlocking these controls for desktop multitasking traditionally mandates installing vendor companion suites.

However, vendor companion suites modernly rely on web-runtime stacks (Electron / Chromium Embedded Framework), consuming gigabytes of disk space and hundreds of megabytes of idle RAM, while spawning 3 to 5 persistent background processes for telemetry, ad brokering, and auto-updating. Power users and developers who rely on Wira Desk for its extreme resource discipline (<2 MB static RAM) currently have no lightweight, native way to harness mouse auxiliary inputs for virtual desktop navigation and window management.

## Solution

Wira Desk introduces native, driverless mouse desktop navigation (`CAP-17`, `FR-30`, `FR-31`, `FR-32`):
1. **Low-Level Mouse Hooking (`WH_MOUSE_LL`):** Intercepts standard USB HID thumb buttons (`WM_XBUTTONDOWN`, `WM_XBUTTONUP`) and horizontal tilt-wheel signals (`WM_MOUSEHWHEEL`) on the dedicated hook thread. Swallows mapped inputs (`return 1`) and forwards all cursor motion (`WM_MOUSEMOVE`) immediately with zero locks, allocations, or logging to guarantee zero cursor latency (`AD-15`, `LBR-WM-11`).
2. **VM/RDP Passthrough:** When the foreground window matches the active VM/RDP bypass policy, mouse inputs call `CallNextHookEx` immediately without interception, matching keyboard shortcut passthrough semantics.
3. **Tilt-Wheel Debounce:** Throttles horizontal tilt-wheel signals through a **150 ms** debounce filter so that a single physical wheel flick triggers exactly one navigation action (`SCN-04`).
4. **Decoupled Execution & Protocol Extension:** Pushes discrete `Command` opcodes into the lock-free ring buffer (extending the wire protocol with opcodes 16–19: `NextVirtualDesktop`, `PrevVirtualDesktop`, `TaskView`, `ShowDesktop`). The elevated Worker actor asynchronously synthesizes native Windows navigation keystrokes via `SendInput` (followed by `suppress_start_menu()` for Win-key combinations) or triggers internal Wira Desk cycling and snapping.
5. **Atomic Config Snapshot & Reload:** Extends `HookSnapshot` and `HookRuntime` to carry mouse mappings atomically across `WM_APP_CONFIG_SNAPSHOT` reloads, validating preset strings against allowed enums.
6. **Dual-Hook Heartbeat:** Manages both `keyboard_hook_handle` and `mouse_hook_handle` in `HookRuntime`, reinstalling and refreshing both atomically during heartbeat checks.
7. **Dedicated Settings Mouse Pane:** Provides a clean Slint UI pane (expanding sidebar navigation to 5 panes: General, Shortcuts, Mouse, VM & Exceptions, About) with a master toggle switch and curated action preset dropdown selectors per physical input.

## User Stories

1. As a developer with multiple virtual desktops, I want to click my mouse thumb buttons to switch virtual desktops and tilt my scroll wheel to open Task View, so that I can multitask across workspaces without moving my hand to the keyboard.
2. As a performance-conscious power user, I want mouse navigation to run inside Wira Desk without installing vendor companion suites, so that my system avoids 500 MB–1.5 GB of disk bloat and hundreds of megabytes of background RAM usage.
3. As a user moving my mouse cursor across the screen, I want cursor tracking to remain perfectly fluid without micro-stutter while the mouse hook is active.
4. As a user with a mechanical tilt wheel, I want a single tilt flick to switch exactly one virtual desktop instead of skipping multiple desktops due to hardware switch bounce.
5. As a user configuring mouse actions in Settings, I want to pick from curated action presets in dropdown menus without needing to manually record raw keystrokes.
6. As a user working inside Remote Desktop or a virtual machine, I want mouse thumb buttons and tilt wheels to pass through directly to the remote session without triggering host desktop transitions.

## Implementation Decisions

- **Shared Schema & Enums (`crates/shared`):**
  - Defines `MouseActionPreset` enum:
    - `NextVirtualDesktop` (`"next_virtual_desktop"`)
    - `PrevVirtualDesktop` (`"prev_virtual_desktop"`)
    - `TaskView` (`"task_view"`)
    - `ShowDesktop` (`"show_desktop"`)
    - `CycleForward` (`"cycle_forward"`)
    - `SnapLeft` (`"snap_left"`)
    - `SnapRight` (`"snap_right"`)
    - `Maximize` (`"maximize"`)
    - `Passthrough` (`"passthrough"` / `"default"`)
  - Adds `MouseConfig` to `shared::Config` with defaults: `enabled = true`, `thumb_back = "prev_virtual_desktop"`, `thumb_forward = "next_virtual_desktop"`, `tilt_left = "task_view"`, `tilt_right = "show_desktop"`.
  - Extends `shared::Command` opcodes:
    - `16 = NextVirtualDesktop`
    - `17 = PrevVirtualDesktop`
    - `18 = TaskView`
    - `19 = ShowDesktop`
    Wire values 0–15 remain frozen.
- **Hook Co-location & Dual Handles (`crates/daemon/src/hook.rs`):**
  - `HookRuntime` stores `keyboard_hook_handle: HHOOK` and `mouse_hook_handle: HHOOK`, plus `last_tilt_ms: u64` and `mouse_mapping: MouseMapping`.
  - `SetWindowsHookExW` installs both hooks during startup; `refresh_hook_on_hook_thread` verifies and refreshes both handles on heartbeat.
  - At the very top of `low_level_mouse_proc`:
    `if n_code < 0 || msg == WM_MOUSEMOVE { return CallNextHookEx(0, n_code, w_param, l_param); }`
  - When foreground matches bypass policy, calls `CallNextHookEx` immediately.
  - Tilt delta extracted via `((mouse_data >> 16) & 0xFFFF) as i16`. Negative delta = left, positive = right. Debounce window pinned to exactly **150 ms**.
  - `WM_XBUTTONUP` swallowed if corresponding `WM_XBUTTONDOWN` was mapped and swallowed.
  - Ring buffer drop-on-full increments `DROPPED_FULL` metric and swallows input.
- **Worker Execution (`crates/daemon/src/worker.rs`):**
  - Implements `execute_mouse_navigation(cmd)`:
    - `NextVirtualDesktop`: `SendInput(Ctrl + Win + Right)` + `suppress_start_menu()`.
    - `PrevVirtualDesktop`: `SendInput(Ctrl + Win + Left)` + `suppress_start_menu()`.
    - `TaskView`: `SendInput(Win + Tab)` + `suppress_start_menu()`.
    - `ShowDesktop`: `SendInput(Win + D)` + `suppress_start_menu()`.
    - Cycle & Snap opcodes reuse existing `execute_cycle()` and `execute_snap()`.
- **Settings Slint UI (`crates/settings`):**
  - Reindexes sidebar to 5 panes (General=0, Shortcuts=1, Mouse=2, VM & Exceptions=3, About=4).
  - Implements `ui/panes/mouse_pane.slint` with master toggle switch and 4 preset selector rows with UI Automation accessibility.
  - Staged edits validate against `MouseActionPreset::ALL` before saving atomically and dispatching `WM_APP_RELOAD_CONFIG`.

## Tickets

- `SPEC-8-01`: Mouse configuration schema, preset validation, and Settings Mouse pane (`settings`, satisfies `UC-14`, `FR-32`).
- `SPEC-8-02`: Low-level mouse hook, dual lifecycle, tilt debounce, and worker dispatch (`window-management`, satisfies `UC-13`, `FR-30`, `FR-31`, blocked by `SPEC-8-01`).
