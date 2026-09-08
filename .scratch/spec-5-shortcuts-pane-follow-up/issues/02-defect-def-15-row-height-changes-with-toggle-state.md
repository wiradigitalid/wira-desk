---
id: SPEC-5-02
component: settings
satisfies: [UC-4]
blocked_by: [SPEC-5-01]
status: done
tests:
  - shortcut_row_slint_snapshot::tests::row_height_is_identical_toggle_on_and_toggle_off
---

# 02: Defect DEF-15 — row container height changes between toggle-on and toggle-off

**What to build:** Fix `DEF-15` (`.control/registry/defects.yaml`). Switching a shortcut row's toggle off
shows a "Disabled" caption under the row's title, which is wanted, but the row's own container currently
grows taller to fit that caption and shrinks back when the toggle is switched back on — every row's height
changes with its own toggle state instead of staying constant. Read `DEF-15` in full first; the owner's
screenshot (attached to the finding this ticket was opened from) shows the same "Switching" group at a
visibly shorter height with both toggles on than with both off.

**Blocked by:** `SPEC-5-01` — both tickets touch the same row markup
(`crates/settings/ui/components/shortcut_row.slint`) that `SPEC-4-03` last touched; landing the tooltip fix
first and building this one on its shape is a smaller conflict surface than the reverse.

- [x] **This is an idea from the owner, not a decided design — read `DEF-15`'s `fix_direction` for the
      shape of it, then use your own judgement for the concrete height.** The owner does NOT want the
      current toggle-on height kept as-is with "Disabled" added on top (that is what makes toggle-off so much
      taller today); they also do not want the container height to change at all between the two states. Land
      on one height that is a little taller than today's toggle-on height, fits "Disabled" close against the
      row's lower edge when it appears, and does not need to grow any further — then hold every row at that
      one height regardless of toggle state.
- [x] A row's rendered height is identical whether its toggle is on or off. A new snapshot test toggles one
      row both ways and asserts the row's rendered height is unchanged.
- [x] The "Disabled" caption still renders, in the same place relative to the title, when its row's toggle is
      off — this ticket fixes the height reacting to it, not whether it appears at all.
- [x] `SPEC-4-03`'s own row-height guard
      (`shortcut_row_slint_snapshot::tests::row_height_is_independent_of_description_length`) stays green —
      it measures height independent of description length at one toggle state; this ticket adds the
      cross-toggle-state guard that state did not cover, and must not weaken the existing one.
- [x] A slightly taller per-row height, applied across all sixteen rows, does not silently reintroduce
      vertical scrolling in the Shortcuts pane at the shell's default window size — `SPEC-4-03`'s own
      no-scroll guard
      (`shortcuts_pane_slint_snapshot::tests::five_groups_fit_the_default_window_width_with_no_horizontal_scroll`)
      is about width, not height, so it will not catch this on its own. Check the pane's total rendered
      content height at the default window size before and after, and say plainly in the closing report
      whether it still fits — this is not the ticket to fix a vertical-fit regression if one appears, only to
      surface it rather than ship it unnoticed. (Edge-case-hunter finding, resolved in this pass.)
- [x] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- `DEF-14` (tooltip size and placement) — landed just before this ticket, `SPEC-5-01`.
- `DEF-13` (percentage field reverting mid-keystroke) — unrelated control on the same row, `SPEC-5-03`.
- Choosing a specific pixel height beyond "identical in both states, and comfortably fits the caption" — the
  owner left the exact number to the builder.
