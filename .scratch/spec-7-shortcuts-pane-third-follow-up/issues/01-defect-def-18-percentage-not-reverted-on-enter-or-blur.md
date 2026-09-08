---
id: SPEC-7-01
component: settings
satisfies: [UC-4, UC-9]
blocked_by: []
status: open
tests:
  - shortcut_row_slint_snapshot::tests::typed_percentage_above_max_reverts_on_enter
  - shortcut_row_slint_snapshot::tests::typed_percentage_below_min_reverts_on_enter
  - shortcut_row_slint_snapshot::tests::typed_percentage_out_of_range_reverts_on_blur
  - shortcut_row_slint_snapshot::tests::stack_row_typed_percentage_out_of_range_reverts_on_enter
---

# 01: Defect DEF-18 — an out-of-range typed percentage is not reverted at Enter/blur, only at save

**What to build:** Fix `DEF-18` (`.control/registry/defects.yaml`). `SPEC-6-02` fixed `DEF-17` so the
STEPPER buttons on a percentage row recover from an out-of-range typed value (clicking `-`/`+` clamps
back into bounds). The owner's follow-up manual test asks for a second, independent recovery path: when
the field itself departs — pressing Enter, or the field losing focus (clicking or tabbing away) — an
out-of-range typed value MUST revert immediately to the row's previous value, as if no change had been
made at all, rather than sitting committed in the draft until a save attempt refuses it. Read `DEF-18`
in full before starting — the root cause is already established there, not left for this ticket to find.

**Blocked by:** None (can start immediately; independent of any other open ticket)

- [ ] **Confirm the established root cause before changing anything**: in
      `crates/settings/ui/components/shortcut_row.slint`, the `pct_input` `TextInput`'s `accepted`
      callback (currently line 285), its `changed has-focus` callback (line 293, the blur path), and its
      `key-pressed` `Tab`/`Backtab` handler (line 302) all call
      `root.percent_changed(Math.round(root.typed_text.to-float()))` whenever `typed_text` parses as a
      float — with NO bounds check against `root.percent_min`/`root.percent_max`. Only a value that
      fails to parse as a float at all falls back to `root.typed_text = "\{root.percent}"`. On the Rust
      side, `Settings::set_percent` (`crates/settings/src/app.rs`, currently line 981) stores whatever
      `value` it is given into `self.draft` with no clamping — so an out-of-range value (e.g. `101` on a
      1-99 row) is committed straight into the draft the moment Enter, blur, or Tab fires, and is
      refused only later, at actual save (`persistence::validate_config`).
- [ ] In each of the three call sites above (`accepted`, `changed has-focus`, `key-pressed` Tab/Backtab),
      add the same bounds check: if the parsed value is inside `[root.percent_min, root.percent_max]`,
      keep firing `percent_changed` exactly as today; if it is OUTSIDE that range, do NOT fire
      `percent_changed` at all — instead reset `root.typed_text = "\{root.percent}"`, reverting the field
      to its previous value, the same way an unparseable value already does.
- [ ] A new test types an ABOVE-max value (e.g. `101` on a 1-99 row), presses Enter, and asserts the
      field's `typed_text` reverts to the row's previous `percent` and that no `percent_changed` signal
      was fired for the out-of-range value.
- [ ] A new test types a BELOW-min value (e.g. `0` on a 1-99 row, or below 10 on the 10-100 Stack row)
      and does the same for Enter — both directions, not only the one the owner happened to type.
- [ ] A new test covers the BLUR path (focus lost, not Enter) with an out-of-range value and asserts the
      same revert — Enter and blur are two different callbacks in the Slint source and neither test
      substitutes for the other.
- [ ] The above-max and below-min revert tests are also run against the Stack row's 10-100 bound family,
      not only a 1-99 `Snap to custom` row — `DEF-17`'s own precedent named this exact family split as a
      candidate a fix must not assume away, and it applies here identically.
- [ ] `SPEC-2-01`'s existing commit-on-departure tests and `SPEC-5-03`'s/`SPEC-6-02`'s existing
      multi-digit and stepper-recovery tests stay green and unweakened — this ticket only changes what
      happens when the departing value is OUT OF RANGE; an in-range value must still commit on departure
      exactly as before.
- [ ] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- The STEPPER buttons' own recovery (`-`/`+` clamping into bounds on an out-of-range value) — already
  fixed, `DEF-17` / `SPEC-6-02`. This ticket does not touch `step_plus`/`step_minus`.
- Save-time validation of an out-of-range percentage — already correct per the owner's own confirmation
  in `DEF-17` and covered by
  `persistence::tests::an_out_of_range_percentage_typed_then_saved_is_refused` (`SPEC-2-01`); not touched
  by this ticket. This ticket makes that state unreachable in the ordinary Enter/blur flow, it does not
  change what happens if it were ever reached.
- `DEF-16` (tooltip paint order) — unrelated control, already fixed in `SPEC-6-01`.
