---
id: SPEC-3-01
component: settings
satisfies: [UC-11, FR-28]
blocked_by: []
status: done
tests:
  - config::tests::action_enabled_fields_roundtrip_through_toml
  - config::tests::frozen_snapping_defaults
  - persistence::tests::a_disabled_action_is_excluded_from_collision_detection
  - persistence::tests::a_config_predating_the_enabled_field_loads_every_action_enabled
  - app::tests::disabling_an_action_retains_its_stored_chord
  - app::tests::re_enabling_an_action_restores_its_prior_chord
---

# 01: Per-action enable flag (settings)

**What to build:** Every one of the sixteen editable shortcut actions (`ShortcutField::ALL`) gains a
persisted on/off flag, independent of its chord string. A UI control on each Shortcuts-pane row lets the
user toggle it; Save persists the flag and, on the daemon side (`SPEC-3-02`, blocked by this ticket), it
determines whether the chord is registered at all. Overlapping Stack's existing
`layout.enable_overlapping_stack` field is retired in favor of this general mechanism — one row, one on/off
concept.

**Blocked by:** None (can start immediately)

- [x] `shared::Config` gains sixteen new boolean fields, one per `ShortcutField`, alongside the existing
      chord-string fields (e.g. `snapping.snap_third_left_enabled` following the existing per-field naming
      convention for that section).
- [x] Every new field's `Default` arm sets `true` explicitly — not derived from the primitive `bool`
      default. A test loads a config file that predates this feature (omits all sixteen fields) and asserts
      every action loads enabled, matching its behavior before this feature existed.
- [x] `layout.enable_overlapping_stack` is removed from the schema; Overlapping Stack's row uses the new
      general-purpose flag instead. `crates/daemon/src/arrangement/stack.rs:15`'s existing check against
      the old field is removed as part of this (the daemon-side exclusion mechanism itself is `SPEC-3-02`'s
      ticket, but this field's removal and the stack planner's now-unconditional behavior — since exclusion
      moves to the hook layer — land together so the schema has no orphaned field for even one commit).
- [x] Loading a config that still has `layout.enable_overlapping_stack = false` seeds Overlapping Stack's
      new enabled flag as `false` on that load, rather than silently re-enabling it via the universal `true`
      default; the old key is absent from the file after the next save, same as any other retired field.
      Loading one with the old key `true`, or absent entirely, loads it enabled.
- [x] `crates/settings/src/persistence.rs`'s `validate_config`/`find_conflict` exclude a disabled field's
      chord from the duplicate-chord scan entirely. A disabled field sharing a chord with an enabled field
      produces no collision; the enabled field registers normally, unaffected.
- [x] Disabling an action never touches its stored chord string; toggling it back on (with no other edit)
      leaves the chord exactly as it was before disabling. `validate_shortcut`'s existing empty-chord
      rejection is unchanged by this ticket — no path in this feature ever calls it with an empty string.
- [x] A UI control appears on every Shortcuts-pane row to toggle the flag, wired to the model the same way
      the existing percentage stepper controls are. Exact visual treatment is this ticket's to choose
      (`SPEC.md`'s Out of Scope) subject to one constraint: a disabled row must read visibly differently
      from a row `DEC-009`'s collision resolution left unbound (`BR-9`) — reuse or extend whatever
      conflict-name/unbound indicator the row already renders for a collision, rather than inventing a
      second unrelated visual language.
- [x] Every new field round-trips through save/reload unchanged (existing round-trip test pattern extended).
- [x] The frozen shipped-defaults test is updated: every action's default enabled state is `true`, and the
      removed `enable_overlapping_stack` field's old frozen-default row is removed.
- [x] Full test suite green once, not only this ticket's own tests.

## Panel follow-up, not a return-trip trigger

Both review axes independently found the same dead code: `crates/settings/src/theme.rs`'s
`TOGGLE_OVERLAPPING_STACK` accessible-name constant is orphaned now that the corresponding LayoutPane
control is retired — no Slint control renders it, only `theme.rs`'s own inventory tests still reference
it. Not a checkbox failure (no behaviour delta), carried to `SPEC-3-02`'s builder brief as a bundled
cleanup rather than triggering its own return trip.
