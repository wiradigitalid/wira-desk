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

## Return trip 1/2 — panel must-fix, `commit fa3dddc`

**Cross-row typed-value loss when departure is triggered by a *different* row's own commit path.**
`crates/settings/src/main.rs:397-408` (`on_percent_changed`) unconditionally clears the shared
`uncommitted_percent` slot (`*uncommitted_pct.borrow_mut() = None;`, line 398) *before* checking whether
the value it is about to commit belongs to the field the slot was holding. The slot
(`main.rs:291-292`) is one `Option<(ShortcutField, u32)>` for the whole Shortcuts pane, not per-row.

Reproduction: type `70` into row A's percent field without blurring it (`TouchArea`s never steal keyboard
focus — that is exactly why steppers/Save/Revert cannot blur the field on their own). Click `+`/`-` on a
*different* row B. Row B's own commit path fires `on_percent_changed(idx=B, ...)`, which wipes the slot
holding A's pending `(A, 70)` without ever applying it, then `sync_model_to_ui` rebuilds the row list from
the unchanged model, silently reverting A's field. Clicking Save afterward saves A's *old* value. Every
other departure handler in this file (`on_start_capture`, `on_swap_shortcuts`, `on_pane_selected`,
`main.rs:317-323`, `359-373`, `375-391`) correctly does
`if let Some((field, val)) = uncommitted_pct.borrow_mut().take() { m.set_percent(field, val); }` —
take-and-apply. `on_percent_changed` is the one path that discards instead.

- [ ] `on_percent_changed` drains the shared `uncommitted_percent` slot the same way its siblings do —
      if the slot holds a *different* field's pending value, that value is applied to the model (not
      discarded) before the just-changed field's own new value is applied. A new test reproduces the
      cross-row scenario above (type in row A, trigger a stepper/commit on row B, assert row A's typed
      value survived) and must fail before the fix and pass after.

**Bundled while in this file (Standards-axis follow-up, not itself a return-trip trigger but cheap to fix
alongside):** `crates/shared/src/constants.rs` adds `DEFAULT_STACK_WIDTH_PERCENT` but nothing references
it — `crates/shared/src/config.rs:172`'s `LayoutConfig::default()` still hardcodes `stack_width_percent: 50`
as a bare literal, unlike its sibling `SnappingConfig::default()` three lines above
(`config.rs:160-163`), which correctly uses `crate::constants::DEFAULT_SNAP_PERCENT`. Wire the new
constant into that default so it matches the established pattern in the same file.
