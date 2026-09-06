---
id: SPEC-1-01
component: window-management
satisfies: [UC-9, FR-26]
blocked_by: []
status: ready-for-agent
tests:
  - arrangement::snap::tests::snap_percent_returns_configured_width_from_the_named_edge
  - arrangement::snap::tests::snap_percent_refuses_a_zero_or_negative_extent
  - arrangement::snap::tests::percent_edges_are_independent_of_each_other
  - config::tests::percent_snap_fields_roundtrip_through_toml
  - config::tests::frozen_snapping_defaults
  - persistence::tests::an_out_of_range_percentage_is_rejected_before_save
  - app::tests::field_declaration_order_places_percent_snap_ahead_of_stack
  - commands::tests::roundtrip_all_commands
---

# 01: Custom-percentage edge snap

**What to build:** Pressing `Ctrl+Alt+Shift+Left/Right/Up/Down` snaps the active window against that edge
at a percentage the user has configured for it in Settings (default 50%, so an untouched install behaves
like the existing half-snap). Each edge's percentage is independent of the others. As part of freeing this
shortcut tier, Overlapping Stack's default shortcut moves from `Ctrl+Alt+Shift+Down` to `Ctrl+Alt+Shift+S`
(`DEC-011`) — no existing configuration file is rewritten; an install still carrying the old default
resolves the chord collision through the existing precedence/unbind mechanism, in this ticket's favor since
the new fields are declared first.

**Blocked by:** None (can start immediately)

- [ ] Four new wire commands exist for the percentage-snap edges (left, right, top, bottom), extending the
      existing command enum without renumbering anything already assigned.
- [ ] Four independent per-edge percentage values and four new shortcut-binding fields exist in the
      persisted configuration schema, each round-tripping through save/reload unchanged, with a default of
      50% / matching the existing frozen half-snap chords' modifier family.
- [ ] Overlapping Stack's shortcut-binding field's default value changes to `Ctrl+Alt+Shift+S`; the frozen
      shipped-defaults test is updated to assert the new value rather than the old one.
- [ ] A percentage-snap planner: given a work area, an edge, and a percentage, returns the target rectangle
      — the named percentage of the work area's width (left/right) or height (top/bottom), measured from
      that edge inward — with no side effects and no monitor/Win32 dependency. Covered at minimum by: exact
      geometry at a representative set of percentages; refusal (no plan) at a percentage that would produce
      a zero or negative extent; one edge's plan is unaffected by another edge's configured percentage;
      correct geometry at a negative-origin work area and at a one-pixel-wide/tall work area, matching the
      boundary coverage the existing half-snap planner already has.
    A test that passes before this planner exists asserts nothing, so seed a stub returning a fixed
      degenerate result before writing it.
- [ ] A configured percentage outside 1-99 is refused at save, with the existing draft left unchanged,
      the same way an invalid shortcut chord is refused today. The percentage is collected via a bounded
      numeric control (matching the existing `stack_width_percent` slider), not free text.
- [ ] Changing a percentage in Settings does not move a window already snapped at the old value; the new
      percentage applies starting from the next press of that edge's shortcut.
- [ ] The chord-to-command translation gains four new arms for these commands, alongside the existing ones,
      with no change to how any existing chord is translated.
- [ ] The four new shortcut fields and four percentage controls appear in the Settings pane's one declared
      list of editable actions — the same list already driving that pane's draw order, focus order, and
      chord-collision precedence — with the new fields declared ahead of the Overlapping Stack field in
      that list.
- [ ] An out-of-range percentage entered in Settings is refused before save, with a message the user can
      act on, the same way an invalid shortcut chord is refused today — not silently clamped, not accepted
      and refused later.
- [ ] The existing declared-sequence/precedence-order test gains rows for the four new fields, including a
      row confirming each is declared ahead of the Overlapping Stack field.
- [ ] A window with an enforced minimum size larger than the requested percentage is positioned flush to
      the named edge at its enforced minimum, via the existing minimum-size enforcement path (not
      reimplemented).
- [ ] Pressing any of the four new chords while Wira Desk's own window is foreground resolves no target and
      arranges nothing, via the existing arrangement-target eligibility guard (not reimplemented).
- [ ] Full test suite green once, not only this ticket's own tests.
