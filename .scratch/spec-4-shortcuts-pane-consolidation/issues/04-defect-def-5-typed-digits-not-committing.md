---
id: SPEC-4-04
component: settings
satisfies: [UC-4]
blocked_by: [SPEC-4-03, SPEC-4-05]
status: done
tests:
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_commits_a_typed_percentage
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_out_of_range_is_refused
  - shortcut_row_slint_snapshot::tests::a_real_keystroke_sequence_commits_a_typed_stack_percentage
---

# 04: Defect DEF-5 — typed digits do not reach a percentage field's value

**What to build:** Fix the gap `.control/registry/defects.yaml`'s `DEF-5` describes: on the live standalone
build, typing a digit by hand into a Shortcuts-pane percentage field has no visible effect — only the `+`/
`-` stepper buttons work. Every existing automated test for this control drives it through
`ElementHandle::set_accessible_value` (the UI Automation `RangeValuePattern` path), which is a different
code path from a real keystroke reaching Slint's native `TextInput` character entry. Read `DEF-5` in full
before starting.

**Blocked by:** `SPEC-4-03` — both tickets touch `shortcut_row.slint`, and `SPEC-4-03` restructures the row
(tooltip, centred cluster) more broadly than this ticket's narrower change to `pct_input`'s key handling.
Landing `03` first and rebasing this fix onto its shape is a smaller conflict surface than the reverse; this
is an enforced ordering, not an optional suggestion, so the two are never picked up in parallel.

- [x] **Root-cause with `wdi-systematic-debugging` before writing any fix.** This repo's own standing rule
      for a bug with an unknown cause applies here without exception — do not guess at `input-type: number`,
      focus/z-order between `pct_field_touch`'s `TouchArea` and the `TextInput` beneath it, or anything else,
      and patch blind. Candidates worth ruling in or out, not a prescription of which is right:
      - Whether `input-type: number` on Slint 1.17's `TextInput` accepts direct character entry at all, or
        narrows it to something the stepper buttons happen to satisfy and typed keystrokes do not.
      - Whether the click actually lands keyboard focus on `pct_input` itself, given `pct_field_touch`'s
        `TouchArea` and the `TextInput`-holding `HorizontalLayout` are siblings in the same `Rectangle`.
      - Whether anything in this process or the elevated daemon's global low-level keyboard hook could be
        involved — check this only if the first two are ruled out, since a global hook affecting a
        same-process, non-chord keystroke would itself be a significant, differently-owned finding.
      - Whether the underlying Slint version/feature set installed actually matches what the `.slint` source
        assumes (`Cargo.lock` pins `slint = "1.17.1"`).
      - Whether standalone launch (`WIRADESK_SETTINGS_ALLOW_NO_DAEMON=1`, used by this session's own
        automated passes) and a real daemon-attached launch behave differently here — this session's two
        live-test attempts produced different outcomes across those two modes for unrelated reasons
        (foreground-lock/UIA access), so ruling this in or out explicitly, rather than assuming one mode's
        root cause generalizes to the other, is part of this ticket's own root-cause pass.
- [x] Once root-caused, the fix makes a **real simulated key-press sequence** (not
      `set_accessible_value`) — typing individual digit characters into a focused, empty-or-partially-typed
      field — result in the typed digits appearing in `typed_text` and, on departure, committing exactly as
      `SPEC-2-01` already proved for the accessible-value path. This explicitly includes the refusal half of
      that behaviour, not only the happy path: a real keystroke sequence that types an out-of-range value
      (e.g. outside 1–99, or outside 10–100 on the `Stack` row once `SPEC-4-02` lands) and departs the field
      must be refused with the same actionable message `SPEC-2-01` already proved for the accessible-value
      path, never silently clamped or silently accepted just because this defect's fix made the digits land.
- [x] A new test asserts the out-of-range-refused behaviour specifically for a real-keystroke-typed value —
      `persistence::tests::an_out_of_range_percentage_typed_then_saved_is_refused` is prior art for the
      accessible-value path; this ticket needs its own version driven by the real keystroke path, not a reuse
      of that test's existing assertion under a new name.
- [x] The existing accessible-value-driven tests (`typed_percentage_commits_on_save_click` and its four
      siblings) are unchanged and still green — they cover the automation/assistive-technology path, which
      this ticket does not touch, and MUST NOT be weakened or deleted to make room for the new test.
- [x] The new test drives the field through whatever this repo's test harness uses for simulated character
      input to a Slint `TextInput` (a keyboard event dispatched through `slint::platform::WindowEvent`, or
      the equivalent this harness already has elsewhere) — not through the accessible-value setter, since
      that is exactly the path this defect exists in the gap between.
- [x] If the fix touches `SPEC-4-02`'s relocated `Stack`-row percent control (once that ticket has landed),
      the same real-keystroke test is added there too — this defect is a property of the shared
      `ShortcutRow` percent control, not of any one row.
- [x] The fix and its new test run under standalone launch
      (`WIRADESK_SETTINGS_ALLOW_NO_DAEMON=1`, this repo's normal automated-test seam). Verifying it live under
      a real daemon-attached, elevated launch is `smoke_test`/owner territory per this repo's own build-safety
      rules, not something this ticket's own automated test can reach — say plainly in the closing report
      that only the standalone mode was verified automatically, rather than implying both were.
- [x] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- Everything in `SPEC-4-01`/`02`/`03` (taxonomy, pane retirement, tooltip/centring/scroll polish). This
  ticket is the one place in `SPEC-4` that is a defect fix rather than a design change, and it must not grow
  into either.
