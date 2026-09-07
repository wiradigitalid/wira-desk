---
id: SPEC-4-01
component: settings
satisfies: [UC-4]
blocked_by: []
status: ready-for-agent
tests:
  - app::tests::shortcut_field_group_declares_five_taxonomic_groups
  - app::tests::snap_maximize_now_sorts_after_every_snap_variant
  - app::tests::grouping_never_reorders_the_declared_sequence
  - app::tests::snap_custom_labels_no_longer_repeat_the_group_name
---

# 01: Shortcuts pane regroups per DEC-014, and Snap-custom row labels drop their group's own name

**What to build:** `ShortcutField`'s declared sequence (`crates/settings/src/app.rs`, `LBR-ST-14`) and its
`group()` function re-cut from three groups to the five `DEC-014` names, and its `ALL`/`label()` for the
four `SnapPercent*` fields drop the `(custom %)` suffix now that the group heading carries that meaning.
`DEC-014` is the accepted decision this ticket applies; read it in full before starting —
`.control/decisions/DEC-014-the-shortcuts-pane-regroups-into-five-taxonomic-groups.md`.

**Blocked by:** None (can start immediately)

- [ ] `ShortcutField::group()` returns exactly these five strings, and every field maps to the group
      `DEC-014` names it under:
      - `"Switching"` — `Switcher`, `Fallback` (unchanged)
      - `"Snap half"` — `SnapLeft`, `SnapRight`, `SnapTop`, `SnapBottom`
      - `"Snap third"` — `SnapThirdLeft`, `SnapThirdMiddle`, `SnapThirdRight`
      - `"Snap custom"` — `SnapPercentLeft`, `SnapPercentRight`, `SnapPercentTop`, `SnapPercentBottom`
      - `"Resize, move & arrange"` — `SnapMaximize`, `MoveNextMonitor`, `Stack`
- [ ] `ShortcutField::ALL`'s declared order changes exactly as `DEC-014` describes: `SnapMaximize` moves out
      of its old position (ahead of the thirds) to sit inside the new last group, after every snap variant
      and before `MoveNextMonitor`/`Stack`. This is a precedence change for chord-collision resolution, not
      cosmetic — re-read `DEC-014`'s "Why" section before touching the order, and do not reorder anything
      `DEC-014` does not name.
- [ ] `ShortcutField::label()` for the four `SnapPercent*` fields drops the parenthetical: `"Snap to left
      edge (custom %)"` becomes `"Snap to left edge"`, and likewise for right/top/bottom. This changes
      `label()`'s return value, which is also the reverse-lookup key (`ShortcutField::from_label`) — update
      any place that keys off the old string literally, not just the display path.
- [ ] `crates/settings/src/main.rs`'s three `group_rows("...")` calls become five, matching the new group
      names, and are wired to five `ShortcutRowData` model vectors instead of three (`rows_switching`,
      `rows_snap_half`, `rows_snap_third`, `rows_snap_custom`, `rows_arrange` — exact identifiers are this
      ticket's to choose, consistent with the existing `rows_switching`/`rows_snap`/`rows_move` naming).
- [ ] `crates/settings/ui/panes/shortcuts_pane.slint` grows a `ShortcutGroup` per new group, heading text
      exactly matching `group()`'s five strings, in declared order.
- [ ] The existing test asserting concatenating the pane groups reproduces `ShortcutField::ALL` exactly
      (`grouping_never_reorders_the_declared_sequence`) is updated for five groups and still holds — this is
      the guard that keeps a group's members contiguous in the declared sequence; it must not be weakened,
      only extended to the new shape.
- [ ] A test asserts `SnapMaximize`'s new position relative to every `Snap*` field (its precedent test
      `assert!((ShortcutField::SnapMaximize as usize) < (ShortcutField::SnapThirdLeft as usize))` inverts —
      write the new relative-position assertion, do not just delete the old one).
- [ ] A test asserts none of the four `Snap custom` labels contain the substring `"(custom"` any more.
- [ ] Any existing collision/precedence test that pins a specific winner between two default-config
      fields where one is `SnapMaximize` is checked by hand — reordering can silently flip which field
      "wins" a shared-chord test fixture, not just require new assertions for the position change itself.
      If none exists today, say so in the ticket's closing report rather than leaving it unstated.
- [ ] Grep the corpus's own prose — `EXPERIENCE.md`, onboarding/tutorial copy, any tip or help string in
      `main_window.slint`/`shortcuts_pane.slint` — for the literal text `"(custom %)"` or `"Snap to left
      edge (custom %)"` and its siblings. The label rename only changes `ShortcutField::label()`'s return
      value; a hand-written string elsewhere that quotes the old label is not touched by that change and
      is left as a stale reference this ticket can catch now instead of `wdi-reconcile` catching it later.
- [ ] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- Moving `stack_width_percent` onto the `Stack` row, and retiring the `Layout` pane — `SPEC-4-02`,
  blocked by this ticket (the `Stack` row must already sit in its final group before it grows a percent
  control, so the two changes are not made to the same row twice).
- Tooltip, vertical-centring, and no-horizontal-scroll polish — `SPEC-4-03`.
