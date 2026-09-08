---
id: SPEC-5-03
component: settings
satisfies: [UC-4, UC-9]
blocked_by: [SPEC-5-02]
status: done
tests:
  - shortcut_row_slint_snapshot::tests::a_multi_digit_keystroke_sequence_keeps_each_intermediate_digit_before_departure
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_commits_a_typed_percentage
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_out_of_range_is_refused
---

# 03: Defect DEF-13 — a percentage field reverts mid-keystroke before a multi-digit value can be finished

**What to build:** Fix `DEF-13` (`.control/registry/defects.yaml`). `SPEC-4-04` fixed `DEF-5` — typed digits
now reach a Shortcuts-pane percentage field's value at all. The owner's follow-up manual test found a
different symptom on the same control: typing a multi-digit value by hand (e.g. `5` then `5` again, meaning
to reach `55`) does not reach the second digit — the field snaps back to its last committed value as soon as
the first digit lands, before a second keystroke can be typed. Read `DEF-13` in full before starting.

**Blocked by:** `SPEC-5-02` — this ticket, like `SPEC-4-04` after `SPEC-4-03`/`05`, touches
`crates/settings/ui/components/shortcut_row.slint`'s percentage control; landing the two row-polish tickets
first and rebasing this defect fix onto their shape is the smaller conflict surface, matching `SPEC-4`'s own
precedent of sequencing its defect fix last.

- [x] **Root-cause with `wdi-systematic-debugging` before writing any fix.** This repo's own standing rule
      for a bug with an unknown cause applies here without exception. `DEF-13`'s own `why_it_hid` names one
      candidate worth ruling in or out, not a prescription of which is right: `pct_input`'s `edited` callback
      fires on every keystroke and calls `root.editing_changed(...)` whenever the in-progress text already
      parses as a float — a lone first digit already does — and whether anything downstream of that
      per-keystroke callback writes back into the field before the user finishes typing is not yet
      established. Other candidates worth checking: whether `root.typed_text`'s binding to the model reacts
      to `editing_changed` in a way that clobbers the in-progress text, and whether this reproduces
      identically on every percentage row or only ones with a tighter minimum (e.g. the `Stack` row's 10–100
      bound versus a `Snap to custom` row's 1–99).
- [x] Once root-caused, the fix makes a real simulated multi-digit keystroke sequence (typing each digit as
      its own key event, not one string set at once) leave the field showing the FULL in-progress text after
      every keystroke before the last — not reverted to the prior committed value partway through.
- [x] A new test types a multi-digit value one keystroke at a time and asserts the field's displayed text
      after each intermediate keystroke, not only the final result — this is the exact window `SPEC-4-04`'s
      own tests never exercised.
- [x] The multi-digit test above is run against a row from EACH percent-bound family this pane ships — a
      `Snap to custom` row (1–99) and the `Stack` row (10–100, folded onto Overlapping Stack by `SPEC-4-02`)
      — not only one. `DEF-13`'s own `why_it_hid` names the bound difference as an open candidate for why the
      symptom might not reproduce identically everywhere; verifying only one family risks shipping a fix that
      still reverts mid-keystroke on the other. (Edge-case-hunter finding, resolved in this pass.)
- [x] A backspace/deletion sequence is exercised too, not only forward typing: deleting a digit down to a
      shorter, momentarily out-of-range value (e.g. clearing a two-digit `Stack`-row value back to a single
      below-minimum digit) must behave the same way as typing forward — the in-progress text survives until
      departure, rather than reverting the instant the interim value is out of range. (Edge-case-hunter
      finding, resolved in this pass.)
- [x] `SPEC-4-04`'s three existing real-keystroke tests
      (`a_real_keystroke_sequence_commits_a_typed_percentage`,
      `a_real_keystroke_sequence_out_of_range_is_refused`,
      `a_real_keystroke_sequence_commits_a_typed_stack_percentage`) stay green and unweakened — they cover
      the fully-typed-then-departed path, which this ticket does not change.
- [x] The out-of-range-refused behaviour on departure is unaffected: a value that is still out of range once
      the field is actually departed (not mid-typing) is refused with the existing actionable message, never
      silently accepted or clamped just because this fix lets an in-progress digit sequence survive.
- [x] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- `DEF-14`/`DEF-15` (tooltip size/placement, row height across toggle state) — `SPEC-5-01`/`SPEC-5-02`,
  landed before this ticket. This ticket touches only the percentage `TextInput`'s own keystroke handling.
- Any change to the accessible-value (`set_accessible_value`) commit path `SPEC-2-01` already proved — this
  defect is specific to a real keystroke sequence, the same distinction `DEF-5` itself drew.
