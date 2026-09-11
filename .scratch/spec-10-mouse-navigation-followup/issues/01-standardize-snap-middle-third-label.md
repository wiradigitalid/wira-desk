---
id: SPEC-10-01
component: settings
satisfies: [UC-14]
blocked_by: []
status: done
tests:
  - config::tests::snap_middle_third_display_label_matches_shortcuts_pane
  - app::tests::mouse_preset_dropdown_displays_snap_to_middle_third
---

# 01: Standardize mouse preset text to "Snap to middle third"

**What to build:** Harmonize the display label for `MouseActionPreset::SnapThirdCenter` in `crates/shared/src/config.rs` from `"Snap Center Third"` to `"Snap to middle third"` to match the Shortcuts tab (`ShortcutField::SnapThirdMiddle`) and the user interface standard. Update all related unit tests and dropdown model assertions.

**Blocked by:** none

## Acceptance Criteria

- [ ] In `crates/shared/src/config.rs`:
      - `MouseActionPreset::SnapThirdCenter.display_label()` returns `"Snap to middle third"`.
      - Serialization/deserialization slug remains `"snap_third_center"` (preserving config compatibility).
- [ ] In `crates/settings/ui/panes/mouse_pane.slint` & `crates/settings/src/main.rs`:
      - The dropdown selector option for center third displays as `"Snap to middle third"`.
      - When selected, the trigger button label updates to `"Snap to middle third"`.
- [ ] Automated tests verify:
      - `config::tests::snap_middle_third_display_label_matches_shortcuts_pane` proves display string equality.
      - Settings snapshot/unit tests verify the option renders with the standardized text.
