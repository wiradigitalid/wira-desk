---
id: SPEC-9-05
component: settings
satisfies: [UC-14]
blocked_by: [SPEC-9-04]
status: done
tests:
  - app::tests::mouse_preset_dropdown_opens_overlay_with_groups
  - app::tests::selecting_preset_from_dropdown_updates_draft
---

# 05: Grouped preset dropdown overlay in Settings UI

**What to build:** Replace the cycle-on-click mechanism in `crates/settings/ui/panes/mouse_pane.slint` with a true window-level dropdown overlay (following the DEF-16 overlay pattern at `z: 1000` in `main_window.slint`). The dropdown displays categorized action groups, allows selecting any preset with a single click, closes on outside clicks or Escape, and correctly marks the draft as dirty upon selection.

**Blocked by:** `SPEC-9-04` (requires expanded preset catalog and categories)

## Acceptance Criteria

- [ ] In `mouse_pane.slint`:
      - Trigger button in `MouseActionRow` displays the current action label and chevron icon.
      - Clicking the trigger emits an `open_selector(int /* slot */, length /* trigger_y */)` callback instead of incrementing `selected_index`.
- [ ] In `main_window.slint`:
      - Implements `PresetDropdownOverlay` at high z-index (`z: 1000`) positioned relative to the trigger.
      - Includes a full-window transparent backdrop TouchArea to detect outside clicks and dismiss the dropdown cleanly.
      - Renders items with clear visual category headers (e.g. "Virtual Desktops", "Snap to Half", "Snap to Third", etc.).
      - Category headers are distinct and non-clickable; individual preset options are selectable and highlight on hover.
      - If content overflows, options are scrollable inside a nested `ScrollView` with max height constraint (e.g. 260px).
- [ ] In `crates/settings/src/main.rs`:
      - Builds the flattened list of entries (`is_header: bool, label: String, slug: String, selectable: bool`) from the expanded `MouseActionPreset` catalog.
      - Handles `on_preset_selected(slot, slug)`: updates the corresponding input field in `draft.mouse`, clears the active dropdown state, and triggers dirty tracking to enable the Save button.
- [ ] Automated and regression tests verify:
      - Clicking trigger opens the overlay with categorized entries.
      - Selecting an item updates the draft value and closes the overlay.
      - Outside click dismisses the overlay without mutating the draft.
