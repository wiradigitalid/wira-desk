---
type: decision
id: DEC-014
status: draft
touches: []
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: null
---

# DEC-014 — The Shortcuts pane regroups into five taxonomic groups, and Maximize moves out of Snap & Resize

## Decision

`ShortcutField::ALL`'s declared sequence (`LBR-ST-14`) is re-cut from three groups to five: **Switching**
(unchanged) · **Snap half** (left/right/top/bottom half) · **Snap third** (left/middle/right third) ·
**Snap custom** (left/right/top/bottom percentage edge, percentage inline on its own row) · **Resize, move
& arrange** (Maximize, Move to next monitor, Overlapping Stack). The percentage value stays inline on its
own action row rather than moving to the Layout pane, because `LBR-ST-14` already forbids a second,
independently maintained list of the same actions, and a Layout-pane control would be exactly that.

## Why

The three-group taxonomy was never designed; it fell out of declaration order. `.control/memlog/settings.md`
already recorded the gap at the time percentage/thirds snap were added: *"deciding which card group the 7
new rows join... is a real UX design decision (`wdi-ux`'s), not a mechanical count substitution."* The owner
asked, in session, why the four percentage-snap rows read as "Move & arrange" rather than "Snap & resize"
alongside the halves and thirds they conceptually belong with — and confirmed grouping by literal shape
(what the action snaps *to*) reads better than the historical accident of insertion order.

`LBR-ST-14` makes the declared sequence load-bearing three times over — pane draw order, keyboard focus
order, and chord-collision precedence — so a taxonomy change here is not cosmetic. The consequence worth
recording on purpose: **Maximize moves from position 5 to the second-to-last position**, behind every snap
variant instead of ahead of them. On the shipped defaults nothing collides, so nothing is currently
observable — but the day a user's own edit puts two of these chords in contention, Maximize now loses
precedence it used to hold. Nobody chose that as a goal; it is the honest side effect of a taxonomy chosen
for a different reason, and it is recorded here rather than discovered later by someone debugging a
surprising collision resolution.

## Cost

Every install's existing `config.toml` is unaffected — this changes `ShortcutField::ALL`'s in-memory
declared order, not any stored value, and no chord is renamed, so nothing here is a migration. Reordering
does change which action a Tier-2 collision-unbind warning (`DEC-009`) names as the winner, for any user who
has bound two of these actions to the same chord. This is the same class of consequence `DEC-011` already
priced when it moved Overlapping Stack off the arrow tier: a declared-order change is a precedence change,
whether or not the shipped defaults ever exercise it.

If the taxonomy is wrong in practice — if five groups reads as more categories than nine rows warrants, or
if percentage-snap's inline value control turns out to want its own tab after all — the reversal is another
declared-sequence edit and another `DEC-`, not a data migration.
