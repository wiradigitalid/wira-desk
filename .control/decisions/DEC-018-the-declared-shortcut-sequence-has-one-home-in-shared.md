---
type: decision
id: DEC-018
status: accepted
touches: []
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: DEC-017
---

# DEC-018 — The declared shortcut sequence has one home in `shared`, and both crates assert against it

## Decision

The declared order of the sixteen editable chord actions becomes one constant in
`crates/shared/src/constants.rs` — the config key of each action, in sequence. `settings` and `daemon`
each keep their own typed list, and each **asserts its list against that constant** rather than against a
literal of its own.

`SPEC-4-01` is amended to carry this: the `DEC-014` reorder is applied to `ShortcutField::ALL`
(`settings`) **and** to `Chords::in_declared_order` plus `resolve_chords`'s row table (`daemon`) in the
same ticket, with the guard test that holds them together.

## Why

`LBR-ST-14` already requires it, in words that leave no room: *one* declared sequence is the single source
of the pane's draw order, its keyboard focus order, **and the precedence order that resolves a chord
collision**, and *a second, independently maintained list of the same actions must not exist*. Its scope
line names both `settings` and `window-management`.

Two such lists exist today. `ShortcutField::ALL` orders the pane and the save-time duplicate rejection;
`Chords::in_declared_order` and the row table in `resolve_chords` order the collision unbinding that
actually runs. They agree only by coincidence — nothing reads one from the other, no test compares them,
and `daemon` cannot even see `ShortcutField`. The agreement has survived on discipline.

`SPEC-4-01` is where the discipline would have run out. `DEC-014` moves Maximize behind every snap
variant and records the consequence it accepts: *the day a user's own edit puts two of these chords in
contention, Maximize now loses precedence it used to hold.* That precedence is the daemon's. Changing
only `ShortcutField::ALL` would leave the consequence undelivered while making it worse than untouched:
the pane's own Tier-2 collision warning (`DEC-009`) would name one winner and the daemon would unbind the
other. A settings-only slice of this ticket is not a thin version of the change — it is a wrong one, and
it is the horizontal slice `wdi-build` refuses.

## Cost

One constant and two assertions, and the reorder becomes a four-file change instead of two. Nothing
stored moves: this is in-memory declared order, so no `config.toml` migration and no chord rename —
the same price `DEC-014` already accepted.

If the constant turns out to want richer contents than config keys — the command byte, the enable flag —
that is another edit to one place, which is the whole point of moving it there. The failure this forecloses
is the other direction: the next taxonomy change touching one list and shipping a pane that disagrees with
the daemon about who keeps a chord.
