---
id: SPEC-6-02
component: settings
satisfies: [UC-4, UC-9]
blocked_by: []
status: done
tests:
  - shortcut_row_slint_snapshot::tests::stepper_recovers_from_an_above_max_typed_value
  - shortcut_row_slint_snapshot::tests::stepper_recovers_from_a_below_min_typed_value
  - shortcut_row_slint_snapshot::tests::stack_row_stepper_recovers_from_an_out_of_range_typed_value
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_commits_a_typed_percentage
  - shortcut_row_slint_snapshot::tests::a_multi_digit_keystroke_sequence_keeps_each_intermediate_digit_before_departure
---

# 02: Defect DEF-17 — an out-of-range typed percentage freezes both stepper buttons

**What to build:** Fix `DEF-17` (`.control/registry/defects.yaml`). `SPEC-5-03` fixed `DEF-13` so
a multi-digit value can be typed to completion without reverting mid-keystroke. The owner's
follow-up manual test found a consequence of that fix: typing a value past the field's own bound
(e.g. `101` where the bound is 1-99) is accepted into the field as free text, exactly as intended
— but once the field holds that out-of-range value, both the `-` and `+` stepper buttons on that
row stop responding to clicks. Save-time validation already refuses the out-of-range value
correctly; only the live stepper buttons are affected. Read `DEF-17` in full before starting — the
root cause is already established there, not left for this ticket to find.

**Blocked by:** None (can start immediately; independent of `SPEC-6-01`, which touches a different
control on the same row markup)

- [x] **Confirm the established root cause before changing anything**: `step_plus` and
      `step_minus` in `crates/settings/ui/components/shortcut_row.slint` (currently lines 67 and
      74) each read `base = root.current_value()` and set `next = base` (i.e. leave the value
      UNCHANGED) whenever their boundary guard is false. Both guards assume `base` is already
      inside `[percent_min, percent_max]`, which stopped being guaranteed once `SPEC-4-04`/`
      SPEC-5-03` let free-typed text past that boundary sit in `typed_text` uncommitted — and
      `app.rs::set_percent` (currently line 981) stores whatever value is committed with no
      clamping, so an out-of-range value is not corrected even once departure commits it.
- [x] Clamp `base` into `[percent_min, percent_max]` before applying each guard, rather than
      assuming it is already there — an above-max value steps DOWN to `percent_max` on the next
      `-` press (and stays refused on `+`, correctly, since it is now exactly at the max); a
      below-min value steps UP to `percent_min` on the next `+` press symmetrically.
- [x] A new test types an ABOVE-max value (e.g. `101` on a 1-99 row) and clicks each stepper
      button, asserting the row recovers a value inside bounds rather than staying frozen.
- [x] A new test types a BELOW-min value (e.g. `0` on a 1-99 row, or below 10 on the 10-100 Stack
      row) and does the same — both directions, not only the one the owner happened to type.
      (Fixing only the above-max side because it is the one reported risks shipping a fix that
      still freezes below the minimum.)
- [x] The above-max recovery test is also run against the Stack row's 10-100 bound family, not only
      a 1-99 `Snap to custom` row — `DEF-13`'s own precedent named this exact family split as a
      candidate the fix must not assume away, and it applies here identically.
- [x] `SPEC-5-03`'s existing tests (multi-digit keystroke retention, valid-value commit-on-departure,
      out-of-range-refused-at-save) stay green and unweakened — this ticket is about the STEPPER
      buttons' own boundary guards, not the typed-text retention or save-time validation paths
      those already cover.
- [x] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- `DEF-16` (tooltip paint order) — a different control on the same row, `SPEC-6-01`.
- Save-time validation of an out-of-range percentage — already correct per the owner's own
  confirmation and already covered by `persistence::tests::an_out_of_range_percentage_typed_then_saved_is_refused`
  (`SPEC-2-01`); not touched by this ticket.
