---
id: SPEC-3-02
component: window-management
satisfies: [UC-12, FR-29]
blocked_by: [SPEC-3-01]
status: done
tests:
  - hook::tests::a_disabled_action_is_absent_from_chords
  - hook::tests::a_disabled_actions_chord_falls_through_to_call_next_hook_ex
  - hook::tests::a_disabled_field_shares_a_chord_with_an_enabled_field_without_collision
  - arrangement::stack::tests::stack_planner_no_longer_reads_a_layout_level_toggle
---

# 02: Hook registration exclusion (window-management)

**What to build:** A shortcut action `settings` recorded as disabled (`SPEC-3-01`) is excluded from
keyboard-hook registration entirely — its chord takes no part in matching, so the physical key combination
reaches the foreground application or Windows exactly as it would if Wira Desk were not running. This
replaces `layout.enable_overlapping_stack`'s existing check inside the arrangement planner
(`crates/daemon/src/arrangement/stack.rs:15`), which ran *after* the hook had already consumed the
keystroke — the defect this ticket exists to close.

**Blocked by:** `SPEC-3-01` — needs the sixteen per-action enabled fields to exist in `shared::Config`.

- [x] `crates/daemon/src/hook.rs`'s `Chords` construction (wherever it builds from a config snapshot) skips
      a disabled field's chord entirely, so it is absent from the struct the same way an unconfigured field
      already is. No change to `match_shortcut` or `unbind_duplicates` themselves — both already operate
      over `Option<Shortcut>`, and a disabled action's chord is simply never placed into the structure they
      read.
- [x] A keystroke matching a disabled action's stored chord is not recognized as any Wira Desk action and is
      passed to `CallNextHookEx` unmodified — verified by a test asserting the specific chord produces no
      match against a `Chords` built from a config with that action disabled.
- [x] A disabled action sharing a chord with an enabled action produces no collision diagnostic and no
      unbinding — the enabled action registers exactly as if the disabled one were not configured at all.
- [x] `crates/daemon/src/arrangement/stack.rs`'s planner no longer reads `layout.enable_overlapping_stack`
      (field removed in `SPEC-3-01`); Overlapping Stack's arrangement behavior when its action is *enabled*
      is otherwise unchanged — this ticket only removes the now-redundant planner-level check, matching
      `SPEC-3-01`'s Overlapping Stack field removal so no reference to the retired field survives in either
      crate.
- [x] Full test suite green once, not only this ticket's own tests.

## Panel follow-ups, not return-trip triggers

Both review axes clean. Two non-blocking notes carried to the final report rather than fixed here: `3p.md`
states 534 tests where the coordinator's independent run on this exact commit shows 533 (progress-tracker
accuracy, not a code defect); `Chords::from_config` and `load_shortcuts_from_config` hand-encode the same
16-field mapping in two places — a maintenance note for whoever adds a 17th field, not a ticket defect.
