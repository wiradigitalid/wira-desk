# 03: Save Changes button dirty state visual affordance

**What to build:** Align the visual presentation and interactive behavior of the "Save Changes" button in the Settings footer with `root.is_dirty` state, providing clear visual feedback on whether unsaved changes exist in the active draft.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] In `crates/settings/ui/main_window.slint`, when `root.is_dirty == false` (clean state), the Save Changes button renders in a subtle/disabled state (`Palette.bg_subtle`, text color `Palette.text_tertiary`, `enabled: false`) matching the clean state behavior of the Revert button.
- [x] When `root.is_dirty == true` (dirty state), the Save Changes button activates with vibrant primary accent styling (`Palette.accent_primary`), hover background (`Palette.accent_hover`), and active click handling.
- [x] When clean, clicking the Save Changes button is disabled.
- [x] The Rectangle's `accessible-action-default => { root.save_clicked(); }` is also gated on `root.is_dirty` (e.g. `if root.is_dirty { root.save_clicked(); }`). Today this binding sits directly on the Rectangle, not on `save_touch`, so it bypasses the `TouchArea`'s `enabled` state entirely — without this fix, assistive-technology users could still trigger a save while clean even though the mouse path is disabled.
- [x] When dirty, clicking Save Changes persists the draft configuration, notifies the daemon via IPC reload signal, clears the dirty state, and transitions the button back to the subtle clean state.
- [x] Automated tests verify the Save Changes button enabled and visual state correctly reflects `is_dirty()` transitions.
