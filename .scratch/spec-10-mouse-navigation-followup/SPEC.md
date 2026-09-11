---
spec: SPEC-10
release: "0.3.0"
prd: wira-desk
fr: [FR-30, FR-31, FR-32]
status: draft
---

# SPEC-10 — Mouse Navigation Follow-up & Hardware Lifecycle Polish

## Problem Statement

Manual user hardware testing of SPEC-9 (mandate DEC-023 delivery) identified four specific issues requiring resolution:

1. **Preset Naming Inconsistency:** In `crates/shared/src/config.rs`, the display label for `MouseActionPreset::SnapThirdCenter` is `"Snap Center Third"`, whereas the Shortcuts pane and application standard uses `"Snap to middle third"`.
2. **Dropdown Remains Open on Tab Navigation:** In `crates/settings/ui/main_window.slint`, clicking a different navigation item in the `Sidebar` while the `PresetDropdownOverlay` is open switches the visible pane underneath, but does not close the dropdown overlay. Additionally, the backdrop `TouchArea` is bounded within the right-hand mica content area rather than the entire window, allowing clicks on the sidebar to bypass the dismissal backdrop.
3. **Daemon Binary Mismatch & Preset Reload Rejection:** During manual testing, the active daemon in memory was an older process (PID 6312, built prior to SPEC-9). When Settings attempted to save any of the 11 new mouse presets (`snap_top`, `snap_third_left`, `snap_third_middle`, `snap_third_right`, `snap_percent_*`, `overlapping_stack`, `move_next_monitor`), the running daemon logged `Config reload skipped: shortcut is not parseable; keeping current settings` and refused the reload because `parse_slug` in the old binary did not recognize the new presets. This caused all newly introduced presets to appear non-functional.
4. **Hardware Tilt Hold Repeat on Stale Daemon:** The manual test checklist for sustained tilt wheel hold (`SPEC-9-06`) failed because the outdated running daemon did not contain the two-phase gesture lockout state machine (`TILT_QUIET_MS = 400 ms`) implemented in SPEC-9-06, continuing to fire hotkeys on hardware auto-repeat.

## Solution

1. **Standardize Preset Label (`SPEC-10-01`):** Change `MouseActionPreset::SnapThirdCenter.display_label()` to `"Snap to middle third"` to match the Shortcuts tab and documentation.
2. **Auto-Dismiss Dropdown on Tab Switch & Full-Window Backdrop (`SPEC-10-02`):** Update `main_window.slint` so `Sidebar.pane_selected` resets `root.dropdown_open = false`, lift the backdrop TouchArea to window root level, and update `on_pane_selected` in `main.rs` to ensure dropdown dismissal on pane departure.
3. **Daemon Process Lifecycle & Preset Reload Parity (`SPEC-10-03`):** Ensure the daemon reload path accepts all 20 presets cleanly with comprehensive test coverage, document process reload procedures for manual and automated runs, and verify window arrangement dispatch across all presets on external windows.
4. **Hardware Tilt Lockout Synthetic Burst & Hold Verification (`SPEC-10-04`):** Add synthetic repeat-rate hardware tests simulating mouse drivers that generate `WM_MOUSEHWHEEL` at 50ms, 100ms, and 200ms intervals, proving exact single-actuation behavior under continuous physical hold.

## User Stories

1. As a user configuring mouse navigation, I want the label for the center third preset to read "Snap to middle third", consistent with the keyboard shortcuts pane.
2. As a user, when I open a preset dropdown and click a different tab in the sidebar, I want the dropdown menu to close immediately rather than remaining on top of the new tab.
3. As a user saving new mouse actions, I want all 20 preset actions (including thirds, custom %, and top/bottom halves) to apply to the running daemon without reload rejection.
4. As a user holding the tilt wheel horizontally, I want the desktop or task view to toggle once and not flicker repeatedly.

## Implementation Decisions

- **Consistent Terminology:** `"Snap to middle third"` is canonical across `shared`, `settings`, and `daemon`.
- **Root Backdrop:** Overlay dismissal must capture clicks on the sidebar and titlebar, not only the mica content area.
- **Daemon Lifecycle Hygiene:** Build scripts and test harnesses must ensure running daemon binaries match the current compilation target.
