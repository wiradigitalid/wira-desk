---
id: SPEC-4-02
component: settings
satisfies: [UC-4, UC-9]
blocked_by: [SPEC-4-01]
status: ready-for-agent
tests:
  - app::tests::pane_enum_no_longer_declares_layout
  - main::tests::overlapping_stack_row_has_percent_true
  - shortcut_row_slint_snapshot::tests::stack_row_percent_commits_on_save_click
  - persistence::tests::stack_width_percent_round_trips_through_the_shortcut_row_path
---

# 02: Layout pane retired; stack width percentage moves onto the Overlapping Stack row

**What to build:** `crates/shared/src/config.rs`'s `layout.stack_width_percent` — currently edited only in
`crates/settings/ui/panes/layout_pane.slint`, which SPEC-4-01 leaves as the sole reason `Pane::Layout`
still exists — moves onto the `Stack` (Overlapping Stack) row in the Shortcuts pane's "Resize, move &
arrange" group, using the same `has_percent`/`percent`/`percent_changed`/`editing_changed` plumbing
`ShortcutRow` already gives every `Snap to custom` row. The `Layout` pane, its nav entry, and
`layout_pane.slint` are removed entirely. Read `DEC-014`'s accepted extension paragraph (the one starting
"Extension accepted the same day...") before starting; it names this exact change and its one open cost.

**Blocked by:** `SPEC-4-01` (the `Stack` row must already sit in its final "Resize, move & arrange" group)

- [ ] `ShortcutField::has_percent()` returns `true` for `Stack`, alongside the four `SnapPercent*` fields.
- [ ] `ShortcutField::get_percent`/`set_percent` (or their equivalent for `Stack`) read and write
      `cfg.layout.stack_width_percent` — the existing field, unrenamed; nothing here is a config migration.
- [ ] `crates/settings/ui/components/shortcut_row.slint`'s `step_plus`/`step_minus` currently hardcode their
      bounds as literals (`base >= 1 && base < 99`) — correct for every `Snap to custom` row, wrong for `Stack`,
      which needs 10–100 (`layout_pane.slint`'s existing `min_percent`/`max_percent`). `ShortcutRow` gains
      per-row bound properties (e.g. `in property <int> percent_min: 1; in property <int> percent_max:
      99;`, overridden to `10`/`100` on the `Stack` row only) rather than a second hardcoded literal pair —
      setting `has_percent: true` on `Stack` without this change silently gives it the wrong range.
- [ ] `Pane` (wherever it is declared — `crates/settings/src/app.rs` and/or the Slint side) drops `Layout`.
      Every place keyed on `Pane::Layout` — focus order, theme accessible-name tables, sidebar nav data —
      is updated in the same commit, not left as a dangling reference to a removed variant. `DEC-014`'s Cost
      section names this risk explicitly: grep for `Pane::Layout` and `layout_pane` and account for every
      hit, do not rely on the compiler to find all of them (a `&str` pane key or a UIA accessible-name table
      entry will not fail to compile). This includes any hardcoded pane-COUNT, not only named references —
      a sidebar nav-items array or test asserting exactly five entries is stale the moment the pane list
      drops to four, even where nothing in it literally says `Layout`.
- [ ] The first-run onboarding wizard (`--onboarding`) copy is checked for a reference to the `Layout` pane
      by name; update or remove any step text that points a new user at a pane that no longer exists.
- [ ] `crates/settings/ui/panes/layout_pane.slint` is deleted, along with its import and instantiation in
      `main_window.slint` and the sidebar's nav item for `Layout`.
- [ ] `crates/settings/src/layout_pane_slint_snapshot.rs`'s existing tests for stack-width behaviour
      (`out_of_range_stack_width_is_refused_not_clamped` and its siblings) move to cover the same behaviour
      on the relocated control — either migrated into `shortcut_row_slint_snapshot.rs` or kept in a
      renamed file, but not left asserting against a pane that no longer exists. No existing assertion is
      weakened; the seam it runs at moves, the guard does not.
- [ ] `DEF-2`'s fix (`STACK_WIDTH_SLIDER`/`STACK_WIDTH_INPUT` distinct accessible semantics in
      `crates/settings/src/theme.rs`) is carried to the relocated control rather than silently dropped —
      the two controls (stepper/typed-field) still need distinct accessible names on their new row.
- [ ] `stack_width_percent`'s existing out-of-range-refused-not-clamped behaviour (`layout_pane.slint`'s
      `commit_typed`) is preserved exactly on the relocated control.
- [ ] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- The five-group taxonomy and label changes — `SPEC-4-01`, already landed by the time this starts.
- Tooltip, vertical-centring, and no-horizontal-scroll polish — `SPEC-4-03`, which touches this same row
  again once its final control set (stepper + keycap + toggle, all three now on `Stack`) is in place.
