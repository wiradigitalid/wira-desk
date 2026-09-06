---
spec: SPEC-1
release: "0.2.0"
prd: wira-desk
fr: [FR-26, FR-27]
status: ready-for-agent
---

# SPEC-1 — Custom-percentage edge snap and snap to thirds

## Problem Statement

Wira Desk already snaps the active window to a fixed half of the screen (left, right, top, or bottom,
`Ctrl+Alt+Arrow`), or maximizes it. That covers exactly one ratio: 50/50. A user who habitually works with
an uneven split — a 70/30 reading layout, a narrow terminal down one edge — has no way to make that their
default; every press of the half-snap shortcut always returns to 50/50. Separately, a user splitting the
screen into three side-by-side panes (browser / editor / terminal) has no shortcut for a third at all —
they resize by hand, every time, on every monitor they move to.

## Solution

Two new keyboard-driven arrangement commands, both DPI-aware per monitor like every existing snap:

1. **Custom-percentage edge snap** (`FR-26`, `UC-9`) — `Ctrl+Alt+Shift+Left/Right/Up/Down` snaps the
   active window against the named edge at a percentage of the work area the user sets independently for
   that edge in Settings (default 50%, so an untouched install behaves like the existing half-snap until
   the user changes it).
2. **Snap to thirds** (`FR-27`, `UC-10`) — `Ctrl+Alt+1/2/3` snaps the active window to the left, middle,
   or right third of the current monitor's work area.

Both reuse the existing arrangement pipeline (hook → ring buffer → worker → planner → `SetWindowPos`) that
`snap_half_left/right/top/bottom` already runs through; neither introduces a new pipeline.

## User Stories

1. As a user who prefers a 70/30 split with my reference material on the right, I want to set the right
   edge's snap percentage to 70% in Settings, so that pressing the percentage-snap shortcut for the right
   edge always gives me that layout without resizing by hand.
2. As a user who has never opened Settings, I want the percentage-snap shortcuts to behave like the
   existing half-snap by default, so that the feature does not change anything for me until I choose to
   configure it.
3. As a user who splits my screen into three columns for browser / editor / terminal, I want one shortcut
   per column, so that I can arrange a three-pane layout as fast as I already arrange a two-pane one.
4. As a user with an ultrawide monitor, I want the percentage and thirds snaps to be computed from that
   monitor's actual work area and DPI, so that the split is exact regardless of monitor size or scaling.
5. As a user who has customized the Overlapping Stack shortcut away from its default, I want my
   customization left untouched by this release, so that a shortcut I deliberately chose does not change
   out from under me.
6. As a user who never touched the Overlapping Stack shortcut, I want to be told (via the existing
   Tier-2 log warning) if my install's chord now resolves to the new percentage-snap feature instead, so
   that a shortcut that stops doing what it used to is not a silent mystery.
7. As a user configuring a percentage in Settings, I want an out-of-range value rejected the same way an
   invalid shortcut is rejected today, so that I cannot save a configuration that would produce a
   degenerate window placement.
8. As a user, I want each edge's percentage to be independent of the others, so that setting the left
   edge to 70% does not imply or constrain what the right edge does.
9. As a user pressing a thirds shortcut on a very narrow monitor, I want the command to refuse rather than
   place a zero-width window, so that I never end up with an invisible or unusable window.
10. As a user whose active window enforces a minimum size larger than the requested percentage or third,
    I want the window positioned flush to the target edge/column at its enforced minimum size, so that the
    window never disappears or gets crushed below what the application allows.
11. As a screen-reader user, I want the four new percentage-snap shortcut fields and three new thirds
    shortcut fields to appear in the Shortcuts pane with the same accessibility behavior as every existing
    shortcut field, so that configuring them is no different from configuring any other shortcut.
12. As a user who presses a percentage-snap or thirds-snap chord while the Settings window is focused, I
    want nothing to happen and the chord to be silently consumed, exactly like every other arrangement
    shortcut does today, so that Wira Desk's own window is never an accidental arrangement target.

## Implementation Decisions

- **New wire commands.** Seven new command codes are added to the existing `u8` command enum, extending
  it rather than renumbering anything: four for the percentage-snap edges (left, right, top, bottom) and
  three for the thirds columns (left, middle, right). The hook thread's existing chord-to-command
  translation gains seven new arms; nothing about the existing translation for other chords changes.
- **Percentage is read at plan time, not carried on the wire.** The wire command names only which edge was
  pressed. The planner reads that edge's configured percentage from the persisted configuration at the
  moment it plans, the same way every other planning input is read fresh rather than cached.
- **Planning is pure and deterministic.** Both new planners are added alongside the existing half-snap and
  maximize planners: given a work area and (for the percentage snap) a percentage, or (for thirds) a
  column selector, they return a target rectangle with no side effects and no dependency on anything but
  their inputs — matching how every existing arrangement planner already works.
  - Percentage snap: the target rectangle is the named percentage of the work area's width (left/right) or
    height (top/bottom), measured from that edge inward. Each edge's percentage is independent; nothing
    couples a left-edge press to what the right edge is configured to.
  - Thirds: the work area's width is divided into three columns computed fresh on every press. Any
    remainder pixel width from a division not exact by three goes to the **middle** column — unlike the
    existing half-snap, which gives a remainder to the first half — so the layout stays visually symmetric
    (left, center, right) rather than putting a stray pixel against only one outer edge.
- **Degenerate placements are refused, not clamped.** A configured percentage, or a work area too narrow
  to produce three non-zero columns, that would produce a zero or negative extent is refused: the planner
  returns no plan, the same "nothing moves" outcome the existing half-snap already gives when its own
  degenerate case is hit.
- **A percentage is valid from 1 to 99 inclusive.** 0 and 100 are excluded on purpose — they duplicate the
  existing maximize and (for one edge alone) near-zero-extent cases rather than expressing a genuinely new
  layout, so a value outside 1-99 is refused at save the same way an invalid shortcut chord is. The control
  that collects it is a bounded numeric input (a slider or spinner), not free text, so a non-numeric entry
  is not a reachable state — the same reasoning that already governs `stack_width_percent`'s existing
  control.
- **Minimum-size enforcement and multi-monitor targeting are unchanged.** Both new commands go through the
  existing post-plan enforcement that already respects an application's declared minimum size and already
  resolves the containing monitor by center-point when a window spans a boundary. Neither is reimplemented.
- **Wira Desk's own windows stay ineligible targets.** Both new commands are refused before any geometry is
  planned when the foreground window belongs to Wira Desk itself, the same guard every existing
  arrangement command already goes through.
- **Overlapping Stack's default shortcut moves.** Its shipped default moves off the arrow tier
  entirely, to a letter chord, so the arrow keys under that modifier combination are free for the new
  percentage-snap feature. This is a change to an existing shipped default, not new behavior of its own.
- **No migration of existing configuration.** An installation whose configuration file already has an
  explicit value for the Overlapping Stack shortcut keeps that value exactly as written — nothing rewrites
  a user's file. The consequence: an installation that still has the old default value will find that value
  now also matches the new percentage-snap-downward chord. The existing chord-collision handling already in
  place resolves this the same way it resolves any other two-fields-one-chord collision: whichever field is
  declared first wins the chord at startup, the other is left unbound, and one diagnostic warning names
  both fields and the chord — no crash, no reject, nothing silent. The new percentage-snap fields are
  declared ahead of the Overlapping Stack field in that declared order, so on such an installation the new
  feature's chord wins and the old stack binding on that one direction goes dark until the user notices the
  warning or opens Settings and rebinds.
- **New configuration schema.** Four independent per-edge percentage values (default matching the existing
  half-snap's 50/50), and seven new shortcut-binding fields — one per new command — added to the existing
  configuration schema and to the existing Shortcuts pane's one declared list of editable actions (the same
  list that drives that pane's draw order, its keyboard focus order, and the chord-collision precedence
  order described above). No second, separately-maintained list of these actions is created.
- **Percentage values are validated the same way an invalid shortcut chord is validated today** — refused
  before save, with a message the user can act on, rather than silently clamped or accepted and only
  refused later at the point of use.
- **A percentage change in Settings does not reach an already-snapped window.** It takes effect on the
  next press of that edge's shortcut, the same way every other configuration change in this product
  already works — nothing here re-applies geometry to a window on save.

## Testing Decisions

A good test here verifies the observable planning behavior — the rectangle a given work area and input
produces, or the refusal when the input is degenerate — never the internal call sequence used to get
there. This is exactly the style the existing half-snap and maximize planners are already tested with:
given a synthetic work area, assert the exact returned geometry, with no window handle, no monitor
enumeration, and no other Win32 dependency involved.

Modules to test, and the prior art each follows:

- **The percentage-snap planner** — prior art: the existing half-snap planner's own test suite, which
  covers exact-region correctness, the two-region tiling guarantee, deterministic division of an odd
  extent, behavior at negative-origin and inverted or empty work areas, and single-pixel-wide work areas.
  The percentage-snap planner needs the equivalent coverage for a configurable ratio instead of a fixed
  50/50: the returned rectangle at a representative set of percentages, refusal at a percentage that would
  produce a zero or negative extent, and that one edge's configured percentage never affects another
  edge's plan.
- **The thirds planner** — prior art: the same half-snap test suite, adapted for three regions instead of
  two: the three columns exactly tile the work area with no gap and no overlap, a width not evenly
  divisible by three gives its remainder to the middle column specifically (not the first, unlike the
  half-snap), and refusal when the work area is too narrow to produce three non-zero columns.
- **The frozen-defaults style of test already covering every other shipped shortcut default** — prior art:
  the existing test that pins every shipped shortcut's default value so a change to any of them is loud
  rather than silent. The seven new shortcut fields' defaults, and the changed Overlapping Stack default,
  both need a row in that same test.
- **Configuration round-trip** — prior art: the existing test verifying every configuration field survives
  a save-and-reload unchanged. The four new percentage values and seven new shortcut fields need the same
  coverage.
- **The declared-sequence / chord-collision-precedence test already covering every existing shortcut
  field** — prior art: the existing test asserting that the field declaration order is the precedence
  order. The seven new fields need rows confirming their position in that order, in particular that every
  percentage-snap field is declared ahead of the Overlapping Stack field.
- **Percentage validation** — prior art: the existing test suite for shortcut-chord validation refusing an
  invalid entry before save. An out-of-range percentage needs the equivalent: refused before save, with the
  existing draft left unchanged, matching how an invalid shortcut is handled today.

## Out of Scope

- Migrating or rewriting any existing installation's configuration file. Explicitly refused — see
  Implementation Decisions above.
- Any UI layout or grouping decision for where the seven new shortcut rows and four new percentage
  controls appear in the Shortcuts/Layout panes. That is a design decision for whoever owns the Settings
  UI, not something this spec settles.
- Any change to the existing fixed half-snap, maximize, monitor-move, or Overlapping Stack *behavior* —
  only the Overlapping Stack shortcut's default *value* changes; none of its arrangement logic does.
- A percentage-snap or thirds-snap variant for the vertical axis of thirds (top/middle/bottom row split).
  Only horizontal thirds are in scope.
- Per-virtual-desktop independent configuration of any of this — same boundary the product already draws
  for every other snap shortcut.

## Further Notes

Both new commands are additive to the existing arrangement pipeline: same hook-to-ring-buffer-to-worker
path, same DPI handling, same eligibility and multi-monitor targeting rules, same non-blocking Win32
application. Nothing about the pipeline itself changes shape; only two new planners and a small
configuration surface are added to it, plus one existing default value that moves.
