---
id: SPEC-1-02
component: window-management
satisfies: [UC-10, FR-27]
blocked_by: []
status: ready-for-review
tests:
  - arrangement::thirds::tests::thirds_tile_the_work_area_without_gap_or_overlap
  - arrangement::thirds::tests::remainder_width_goes_to_the_middle_column
  - arrangement::thirds::tests::narrow_work_area_is_refused
  - config::tests::third_snap_fields_roundtrip_through_toml
  - app::tests::field_declaration_order_includes_third_snap_fields
  - commands::tests::roundtrip_all_commands
---

# 02: Snap to thirds

**What to build:** Pressing `Ctrl+Alt+1/2/3` snaps the active window to the left, middle, or right third of
the current monitor's work area. Independent of the percentage-snap feature (SPEC-1-01) — shares only the
existing arrangement pipeline and the existing declared shortcut-sequence list, not any shortcut tier.

**Blocked by:** None (can start immediately, in parallel with SPEC-1-01)

- [x] Three new wire commands exist for the thirds columns (left, middle, right), extending the existing
      command enum without renumbering anything already assigned — including whatever SPEC-1-01 already
      added, if it landed first.
- [x] Three new shortcut-binding fields exist in the persisted configuration schema, round-tripping through
      save/reload unchanged, with defaults in the unused digit-chord space (`Ctrl+Alt+1/2/3`).
- [x] A thirds planner: given a work area, returns the target rectangle for the named column (left, middle,
      right) by dividing the work area's width into three columns computed fresh on every call — with no
      side effects and no monitor/Win32 dependency. Covered at minimum by: the three columns exactly tile
      the work area with no gap and no overlap; a width not evenly divisible by three gives its remainder
      to the **middle** column specifically (not the first); refusal (no plan) when the work area is too
      narrow to produce three non-zero columns; correct geometry at a negative-origin work area, matching
      the boundary coverage the existing half-snap planner already has.
    A test that passes before this planner exists asserts nothing, so seed a stub returning a fixed
      degenerate result before writing it.
- [x] The chord-to-command translation gains three new arms for these commands, alongside the existing
      ones, with no change to how any existing chord is translated.
- [x] The three new shortcut fields appear in the Settings pane's one declared list of editable actions —
      the same list already driving that pane's draw order, focus order, and chord-collision precedence.
- [x] The existing declared-sequence/precedence-order test gains rows for the three new fields.
- [x] A window with an enforced minimum size larger than one third of the work area is positioned flush to
      the named column at its enforced minimum, via the existing minimum-size enforcement path (not
      reimplemented).
- [x] Pressing any of the three new chords while Wira Desk's own window is foreground resolves no target
      and arranges nothing, via the existing arrangement-target eligibility guard (not reimplemented).
- [x] Full test suite green once, not only this ticket's own tests.
