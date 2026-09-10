---
id: SPEC-8-01
component: settings
satisfies: [UC-14, FR-32]
blocked_by: []
status: open
tests:
  - config::tests::mouse_config_roundtrips_through_toml
  - config::tests::missing_mouse_section_defaults_safely
  - config::tests::mouse_action_preset_slug_parsing
  - persistence::tests::mouse_preferences_save_and_reload_signal
  - persistence::tests::invalid_mouse_preset_string_is_rejected
  - app::tests::mouse_pane_loads_configured_presets
  - app::tests::toggling_mouse_navigation_updates_draft
  - app::tests::five_pane_focus_order_includes_mouse
---

# 01: Mouse configuration, preset validation, and Settings Mouse pane

**What to build:** Add the `[mouse]` configuration section to `shared::Config`, declare the canonical `MouseActionPreset` enum, extend pre-save validation in `persistence.rs`, reindex the Settings shell from 4 panes to 5 panes, and create `ui/panes/mouse_pane.slint` with a master toggle switch and 4 preset dropdown selectors.

**Blocked by:** None (can start immediately)

## Acceptance Criteria

- [ ] `shared::Config` gains `mouse: MouseConfig` with fields:
      - `enabled: bool` (default `true`)
      - `thumb_back: String` (default `"prev_virtual_desktop"`, corresponding to physical `XBUTTON1`)
      - `thumb_forward: String` (default `"next_virtual_desktop"`, corresponding to physical `XBUTTON2`)
      - `tilt_left: String` (default `"task_view"`, horizontal wheel left)
      - `tilt_right: String` (default `"show_desktop"`, horizontal wheel right)
- [ ] `MouseActionPreset` enum declared in `crates/shared` with canonical string conversions:
      - `"next_virtual_desktop"` ("Next Virtual Desktop")
      - `"prev_virtual_desktop"` ("Previous Virtual Desktop")
      - `"task_view"` ("Task View")
      - `"show_desktop"` ("Show Desktop")
      - `"cycle_forward"` ("Cycle Same-App Window Forward")
      - `"snap_left"` ("Snap Window Left")
      - `"snap_right"` ("Snap Window Right")
      - `"maximize"` ("Maximize Window")
      - `"passthrough"` ("Default / Passthrough")
- [ ] `MouseConfig` implements `Serialize`, `Deserialize`, `Default`, and `Clone`. A test verifies that loading a configuration omitting `[mouse]` loads safe defaults without error.
- [ ] Pre-save validation in `crates/settings/src/persistence.rs` (`validate_config`) validates that preset strings parse to valid `MouseActionPreset` values, rejecting malformed hand-edited strings.
- [ ] Settings shell reindexed to 5 panes across all layers:
      - `Pane::ALL` / `Pane::from_index` in `crates/settings/src/app.rs` includes `Pane::Mouse` at index 2 (General=0, Shortcuts=1, Mouse=2, VmExceptions=3, About=4).
      - `focus_order()` updated to include Mouse pane navigation and controls (`LBR-ST-5`).
      - `sidebar.slint` adds a Mouse icon item at index 2.
      - `main_window.slint` adds pane conditional for `current_pane == 2` rendering `MousePane`.
      - Tests asserting 4-pane invariants updated to assert 5 panes.
- [ ] New Slint pane component `ui/panes/mouse_pane.slint` renders:
      - Master enable/disable toggle switch ("Enable Mouse Navigation").
      - 4 configuration rows for Thumb Button 1 (Back), Thumb Button 2 (Forward), Tilt Wheel Left, Tilt Wheel Right.
      - Each row provides an accessible dropdown selector populated from `MouseActionPreset::ALL`.
- [ ] Changing any mouse setting marks the draft as dirty and enables the Save button.
- [ ] Saving writes `%APPDATA%\WiraDesk\config.toml` atomically and emits `WM_APP_RELOAD_CONFIG` to the daemon hidden window (`WiraDeskDaemonHiddenWindow`).
- [ ] UI Automation accessibility properties (`accessible-role`, `accessible-label`) are declared across the new controls.
- [ ] All unit tests in `crates/shared` and `crates/settings` pass.
