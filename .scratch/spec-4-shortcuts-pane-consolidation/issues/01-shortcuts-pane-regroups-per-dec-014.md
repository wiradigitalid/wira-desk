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
  - app::tests::the_declared_sequence_matches_the_shared_source
  - hook::tests::the_daemon_precedence_order_matches_the_shared_source
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
      - `"Snap to half"` — `SnapLeft`, `SnapRight`, `SnapTop`, `SnapBottom`
      - `"Snap to third"` — `SnapThirdLeft`, `SnapThirdMiddle`, `SnapThirdRight`
      - `"Snap to custom"` — `SnapPercentLeft`, `SnapPercentRight`, `SnapPercentTop`, `SnapPercentBottom`
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
- [ ] A test asserts none of the four `Snap to custom` labels contain the substring `"(custom"` any more.
- [ ] Any existing collision/precedence test that pins a specific winner between two default-config
      fields where one is `SnapMaximize` is checked by hand — reordering can silently flip which field
      "wins" a shared-chord test fixture, not just require new assertions for the position change itself.
      If none exists today, say so in the ticket's closing report rather than leaving it unstated.
- [ ] Grep the corpus's own prose — `EXPERIENCE.md`, onboarding/tutorial copy, any tip or help string in
      `main_window.slint`/`shortcuts_pane.slint` — for the literal text `"(custom %)"` or `"Snap to left
      edge (custom %)"` and its siblings. The label rename only changes `ShortcutField::label()`'s return
      value; a hand-written string elsewhere that quotes the old label is not touched by that change and
      is left as a stale reference this ticket can catch now instead of `wdi-reconcile` catching it later.
## Amendment 1 — 2026-09-07, under `DEC-018` (coordinator, mandate `DEC-017`)

**Why this ticket grew.** `LBR-ST-14` says one declared sequence is the single source of the pane's draw
order, its focus order, **and the chord-collision precedence order**, and that a second independently
maintained list must not exist. Two exist: `ShortcutField::ALL` here, and `Chords::in_declared_order` plus
`resolve_chords`'s row table in `crates/daemon/src/hook.rs`. They agree only by coincidence — nothing reads
one from the other, no test compares them, and `daemon` cannot see `ShortcutField` at all.

Reordering only the `settings` list would leave `DEC-014`'s own recorded consequence undelivered (the
daemon would still give Maximize its old precedence) **and** make the pane's Tier-2 collision warning
(`DEC-009`) name a different winner than the daemon actually unbinds. That is a wrong slice, not a thin
one. Read `DEC-018` before starting.

- [ ] `crates/shared/src/constants.rs` grows `SHORTCUT_DECLARED_ORDER`: the sixteen config keys in
      declared order, exactly the strings `ShortcutField::key()` already returns. It belongs there for the
      reason that file's own header gives — a value defined separately in each crate fails silently
      instead of at compile time.
- [ ] `ShortcutField::ALL`'s keys equal `SHORTCUT_DECLARED_ORDER`, asserted
      (`the_declared_sequence_matches_the_shared_source`). Assert against the constant, never against a
      fresh literal of the same order — a second literal is the very thing `LBR-ST-14` forbids.
- [ ] `Chords::in_declared_order` and `resolve_chords`'s row table carry the same reorder, and the
      daemon's row-table key sequence equals `SHORTCUT_DECLARED_ORDER`, asserted
      (`the_daemon_precedence_order_matches_the_shared_source`). The `Chords { ... resolved[N] }` field
      mapping moves with it — those indices are positional and every one of them shifts.
- [ ] `dec_011_collision_favors_percent_snap_bottom_over_legacy_stack_default` is the collision fixture the
      ticket asked to be checked by hand. It pins `(14, 15)` and reads `resolved[14]`/`resolved[15]` as
      `snap_percent_bottom`/`stack`. Under the new order `snap_percent_bottom` is 12 and `stack` is 15, so
      its literal indices and its inline 16-entry array both move. `DEC-011`'s *outcome* must not change —
      percent-snap still beats legacy stack — only the indices that express it. No collision fixture pins
      a winner involving `SnapMaximize`; that is the answer to the ticket's "say so if none exists".

**Where `DEC-014` is imprecise, and which reading wins.** Its Why section calls Maximize's old slot
"position 5" (it is the 7th entry, index 6) and its new one "the second-to-last position" (the group
enumeration in its Decision section puts it *third*-to-last, ahead of `MoveNextMonitor` and `Stack`). The
Decision section's group membership is the normative list and this ticket's own wording matches it, so
`SnapMaximize` lands at index 13, before `MoveNextMonitor` (14) and `Stack` (15). The Why section's two
loose figures are explanatory prose and do not bind; they are corrected in `DEC-014` in the same change,
which is permitted because `DEC-014` is `accepted` and not yet `applied`.

- [ ] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- Moving `stack_width_percent` onto the `Stack` row, and retiring the `Layout` pane — `SPEC-4-02`,
  blocked by this ticket (the `Stack` row must already sit in its final group before it grows a percent
  control, so the two changes are not made to the same row twice).
- Tooltip, vertical-centring, and no-horizontal-scroll polish — `SPEC-4-03`.
