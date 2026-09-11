---
spec: SPEC-9
release: "0.3.0"
prd: wira-desk
fr: [FR-30, FR-31, FR-32]
status: closed
---

# SPEC-9 — Mouse Navigation Polish, Preset Catalog Expansion & Settings UX Consistency

## Problem Statement

Following the initial delivery and manual hardware testing of SPEC-8 (Driverless Mouse Desktop Navigation, mandate DEC-022), practical usage revealed several UX inconsistencies, hardware edge cases, and architectural gaps:

1. **Preset Selector is Cycle-on-Click Instead of a True Dropdown:** In `mouse_pane.slint`, clicking a preset control simply increments `selected_index` mod N instead of opening a selectable dropdown list. Furthermore, the preset options are limited to only 9 actions, omitting numerous popular window management commands (such as snap quarters, thirds, halves top/bottom, custom percentages, and overlapping stack) and lacking logical visual grouping.
2. **Tilt Wheel Continuous Hold Spamming:** Physical tilt wheels on many mice continuously emit `WM_MOUSEHWHEEL` ticks when pushed and held. The initial 150 ms debounce filter only collapses burst bounce within a single flick; holding the wheel causes repeated triggering every 150 ms (e.g. rapid flashing/toggling of Show Desktop or Task View), violating the single-actuation intent of `SCN-04`.
3. **Inverted Tilt Wheel Defaults:** Default settings mapped `tilt_left` to `task_view` and `tilt_right` to `show_desktop`. User ergonomic testing confirms that tilting left intuitively maps to showing desktop and tilting right maps to Task View.
4. **Vertical Misalignment of Toggle Switches:** `ToggleSwitch` controls in `general_pane.slint`, `mouse_pane.slint`, and `about_pane.slint` lack clean vertical centering relative to their adjacent multi-line title and description text blocks.
5. **Inconsistent Card Dividers:** Cards in `shortcuts_pane.slint` use zero container padding with full-bleed dividers, whereas `general_pane.slint` and `about_pane.slint` use 16px container padding causing dividers to be indented by 16px, and `mouse_pane.slint` uses 4px padding.
6. **Unbounded Debug Trace Log Growth:** In development and test runs, `append_debug_trace` continuously writes to `%APPDATA%\WiraDesk\wiradesk-debug-trace.log` without the 1 MB size cap and rotation mechanism enforced on `wiradesk.log`.

## Solution

SPEC-9 addresses these findings comprehensively across the daemon, shared schema, and settings UI:

1. **Grouped Dropdown Overlay (`SPEC-9-05`):** Replace the cycle-on-click control with a true window-level dropdown overlay (leveraging the DEF-16 overlay pattern at high z-index), displaying categorized action groups without popup focus stealing.
2. **Preset Catalog Expansion (`SPEC-9-04`):** Expand `MouseActionPreset` from 9 to ~20 actions, covering virtual desktops, Windows shell actions, window switching, all snapping variants (halves, thirds, custom edge %, maximize), overlapping stack, and monitor hopping.
3. **Tilt Wheel Gesture Lockout (`SPEC-9-06`):** Implement a two-phase filter on `HookRuntime`: Phase A (150 ms bounce filter for mechanical contact bounce) and Phase B (gesture lockout that suppresses all subsequent ticks during a sustained hold until a 400 ms quiet window elapses), ensuring exactly one action fires per physical hold or flick.
4. **Correct Inverted Tilt Defaults (`SPEC-9-06`):** Update `MouseConfig::default()` so `tilt_left` defaults to `"show_desktop"` and `tilt_right` defaults to `"task_view"`.
5. **Unified `SettingToggleRow` (`SPEC-9-02`):** Extract a reusable component wrapping `ToggleSwitch` in `VerticalLayout { alignment: center }`, eliminating manual offsets and standardizing vertical centering across all panes.
6. **Full-Bleed Divider Standard (`SPEC-9-03`):** Standardize multi-row cards to zero container padding with row-level internal padding (16px), ensuring `CardDivider` renders uniformly edge-to-edge.
7. **Debug Trace Rotation & AppData Inventory (`SPEC-9-01`):** Extract a shared rotation helper (`rotate_at_cap`) applying a 1 MB cap + `.old` backup to `wiradesk-debug-trace.log` in debug builds, and document the complete AppData file lifecycle.

## User Stories

1. As a user configuring mouse navigation, I want to click an input row and choose from an organized, categorized dropdown menu rather than repeatedly clicking to cycle through a flat list.
2. As a user, I want mouse buttons and tilt directions to trigger any window management shortcut (such as snap to thirds, custom edge percentages, or overlapping stack), not just basic halves.
3. As a user holding the tilt wheel horizontally, I want Wira Desk to trigger the assigned action exactly once without spamming or toggling repeatedly.
4. As a user tilting the wheel left, I want the desktop to reveal cleanly by default, and tilting right to open Task View.
5. As a user viewing Settings, I want toggle switches to be mathematically centered vertically with their feature descriptions across all screens.
6. As a developer running debug builds and tests, I want debug trace logs bounded to ~2 MB with automatic rotation so that local storage does not grow indefinitely.

## Implementation Decisions

- **Architecture of Dropdown Overlay:** In Slint, direct `PopupWindow` steals keyboard focus and breaks `FocusScope` traversal (`DEF-16`). The dropdown will be implemented as a floating window-level overlay in `main_window.slint` positioned below the trigger row at `z: 1000`, dismissed by clicking outside or pressing Escape.
- **Two-Phase Tilt State Machine:**
  - `last_tilt_ms`: timestamp of last accepted command.
  - `last_tilt_event_ms`: timestamp of most recent `WM_MOUSEHWHEEL` message.
  - `tilt_gesture_armed`: boolean flag armed upon command dispatch. While armed, all tilt events are swallowed (`return 1`).
  - Disarming rule: when `now - last_tilt_event_ms >= 400 ms` (quiet period), `tilt_gesture_armed` is reset to `false`.
- **Expanded Preset Enum & Mapping:**
  - Extends `shared::MouseActionPreset` with full variants.
  - Maps to existing `shared::Command` wire values (0–19).
- **Shared Log Rotation:**
  - Refactors `rotate` and `write_line_to` logic into `shared` or a reusable daemon helper `rotate_at_cap(path, max_bytes)`.
  - Applies identical 1 MB ceiling and `.old` rotation to `wiradesk-debug-trace.log`.
