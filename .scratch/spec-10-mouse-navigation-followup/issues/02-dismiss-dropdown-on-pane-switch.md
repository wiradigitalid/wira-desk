---
id: SPEC-10-02
component: settings
satisfies: [UC-14]
blocked_by: []
status: done
tests:
  - app::tests::switching_panes_while_dropdown_open_automatically_dismisses_overlay
  - app::tests::clicking_sidebar_outside_content_dismisses_preset_overlay
---

# 02: Auto-dismiss preset dropdown overlay on tab switch & full-window backdrop

**What to build:** In `crates/settings/ui/main_window.slint` and `crates/settings/src/main.rs`, ensure that when the user switches tabs/panes from the Sidebar while the preset dropdown overlay is open, the overlay is automatically dismissed (`dropdown_open = false`). Expand the dismiss backdrop so outside clicks on the sidebar and titlebar also dismiss the dropdown cleanly without changing settings.

**Blocked by:** none

## Acceptance Criteria

- [ ] In `crates/settings/ui/main_window.slint`:
      - `Sidebar.pane_selected(idx)` handler sets `root.dropdown_open = false` before notifying pane change.
      - Backdrop `TouchArea` for `dropdown_open` is lifted to root window level (or covers both sidebar and content areas at `z: 999`), ensuring any outside click anywhere on the window dismisses the dropdown.
- [ ] In `crates/settings/src/main.rs`:
      - In `on_pane_selected(idx)` callback, explicitly set `w.set_dropdown_open(false)`.
      - In `sync_model_to_ui`, ensure state transitions maintain correct dropdown closure.
- [ ] Automated tests verify:
      - Opening dropdown on Mouse pane, then switching to General or Shortcuts pane results in `dropdown_open == false`.
      - Clicking sidebar outside the dropdown menu bounds dismisses the overlay.
