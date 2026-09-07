---
id: SPEC-2-01
component: settings
satisfies: [UC-4, UC-9]
blocked_by: []
status: ready-for-agent
tests:
  - shortcut_row_slint_snapshot::tests::typed_percentage_commits_on_save_click
  - shortcut_row_slint_snapshot::tests::typed_percentage_commits_on_stepper_click
  - shortcut_row_slint_snapshot::tests::typed_percentage_commits_on_focus_change_to_another_row
  - persistence::tests::an_out_of_range_percentage_typed_then_saved_is_refused
  - layout_pane_slint_snapshot::tests::out_of_range_stack_width_is_refused_not_clamped
---

# 01: Commit-on-departure for percentage fields, and stepper validation parity

**What to build:** A typed percentage value must commit on any departure from its field — not only on
Enter. Today `crates/settings/ui/components/shortcut_row.slint`'s percentage `TextInput` only commits via
its own `accepted` (Enter) and `changed has-focus` handlers, and nothing else in the row can take keyboard
focus: `Save`/`Revert` (`crates/settings/ui/main_window.slint:435-455`) and the `+`/`-` steppers are bare
`Rectangle` + `TouchArea`, so clicking any of them never fires `has-focus`'s blur transition. The typed
value is silently discarded — clicking Save after typing `70` saves the old value, and clicking `+` after
typing `70` computes from the old value (`50 + 1 = 51`, not `71`).

Separately, `crates/settings/ui/panes/layout_pane.slint`'s `stack_width_percent` stepper (visually identical
to the Shortcuts-pane percentage control) silently clamps an out-of-range value (`Math.clamp`) instead of
refusing it — the opposite philosophy from the percentage field's own already-reviewed behavior
(`SPEC-1-01`'s Return-trip-1 fix: refuse with a message, never silently clamp). Two controls that look the
same must validate the same way; refuse-and-explain is the one already reviewed and correct.

While in this file: `layout_pane.slint`'s in-pane title still reads `"Layout & Snapping"`, contradicting
`Pane::label()`'s own comment in `crates/settings/src/app.rs` stating the tab is named `"Layout"` specifically
*because* this pane holds no chord — `EXPERIENCE.md`/`DESIGN.md` already document the pane as `Layout`. Fix
the string to match what every other document already says.

**Blocked by:** None (can start immediately)

- [ ] Typing a percentage value and clicking **Save** (with no Enter pressed first) commits the typed value
      — verified by a test driving the row's model without going through Enter.
- [ ] Typing a percentage value and clicking `+` or `-` computes from the just-typed value, not the last
      committed one (typing `70` then clicking `+` yields `71`, not `51`).
- [ ] Typing a percentage value and clicking anywhere that moves focus away from the field — another row's
      shortcut button, another pane's tab — commits the typed value, the same as pressing Enter or the
      existing blur handler already does.
- [ ] Typing a percentage value and pressing Tab or Shift+Tab to move focus away commits the typed value —
      keyboard-only navigation must not be a second path that still discards it (`FR-20`, `FR-21`).
- [ ] An out-of-range value typed into the percentage field and committed via any of the above departures is
      refused with the existing actionable message, not silently accepted or clamped — the same behavior
      `SPEC-1-01` already established for Enter/blur, now reachable from every departure path.
- [ ] `layout_pane.slint`'s `stack_width_percent` stepper refuses an out-of-range value with an actionable
      message instead of silently clamping it via `Math.clamp` — matching the percentage field's philosophy,
      not the reverse. `LBR-ST-13` is the closest prior art for "a boundary is enforced by refusal, not by
      silently coercing the value" in this same pane family.
- [ ] `layout_pane.slint`'s in-pane title reads `"Layout"`, matching `Pane::label()`'s tab name and the
      reasoning already documented there and in `EXPERIENCE.md`/`DESIGN.md`.
- [ ] Full test suite green once, not only this ticket's own tests.
