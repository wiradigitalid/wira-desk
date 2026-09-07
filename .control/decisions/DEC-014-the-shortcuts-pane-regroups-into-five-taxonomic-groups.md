---
type: decision
id: DEC-014
status: accepted
touches: []
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: kodesh87, 2026-09-07
---

# DEC-014 — The Shortcuts pane regroups into five taxonomic groups, and Maximize moves out of Snap & Resize

## Decision

`ShortcutField::ALL`'s declared sequence (`LBR-ST-14`) is re-cut from three groups to five: **Switching**
(unchanged) · **Snap to half** (left/right/top/bottom half) · **Snap to third** (left/middle/right third) ·
**Snap to custom** (left/right/top/bottom percentage edge, percentage inline on its own row) · **Resize, move
& arrange** (Maximize, Move to next monitor, Overlapping Stack). The percentage value stays inline on its
own action row rather than moving to the Layout pane, because `LBR-ST-14` already forbids a second,
independently maintained list of the same actions, and a Layout-pane control would be exactly that.

**Extension accepted the same day, before this decision was ever applied:** the `Layout` pane itself is
retired, not merely left unused by `Snap to custom`. Its one remaining control — `layout.stack_width_percent`
— was never a chord and was already the last thing keeping that pane alive once Overlapping Stack's on/off
toggle moved to Shortcuts (`SPEC-3-01`). It moves inline onto the Overlapping Stack row in **Resize, move &
arrange**, the same `has_percent` pattern the four `Snap to custom` rows already use, so one row carries both
its toggle and its percentage. `Layout` is removed from the sidebar; Settings ships four panes (`General`,
`Shortcuts`, `VM & Exceptions`, `About`). This is the same reasoning as the paragraph above, extended to the
one control that reasoning had not yet reached.

## Why

The three-group taxonomy was never designed; it fell out of declaration order. `.control/memlog/settings.md`
already recorded the gap at the time percentage/thirds snap were added: *"deciding which card group the 7
new rows join... is a real UX design decision (`wdi-ux`'s), not a mechanical count substitution."* The owner
asked, in session, why the four percentage-snap rows read as "Move & arrange" rather than "Snap & resize"
alongside the halves and thirds they conceptually belong with — and confirmed grouping by literal shape
(what the action snaps *to*) reads better than the historical accident of insertion order.

`LBR-ST-14` makes the declared sequence load-bearing three times over — pane draw order, keyboard focus
order, and chord-collision precedence — so a taxonomy change here is not cosmetic. The consequence worth
recording on purpose: **Maximize moves from index 6 to index 13**, behind every snap variant instead of
ahead of them — the first entry of the new last group, ahead of Move to next monitor and Overlapping
Stack, exactly as the group enumeration above lists it. On the shipped defaults nothing collides, so nothing is currently
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

Removing `Layout` costs one more thing worth naming: `stack_width_percent` is not one of the sixteen
`ShortcutField::ALL` entries — it has no chord, no enable flag, and no collision precedence — so it does
not fit `ShortcutField`'s existing per-row plumbing without that plumbing growing a percent-only variant.
Any test or focus-order declaration keyed on `Pane::Layout` existing at all is stale the moment this ships
and MUST be updated in the same ticket, not left as a dangling reference to a removed pane.
