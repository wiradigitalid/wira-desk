---
spec: SPEC-4
release: "0.5.0"
prd: wira-desk
fr: [FR-15, FR-26, FR-28]
status: ready-for-agent
---

# SPEC-4 — Shortcuts pane consolidation & redesign

## Problem Statement

The owner's live manual test of the `DEC-016` build (`.scratch/live-test-dec-016.md`, 2026-09-07, pre-merge
of PR #17) found the Shortcuts pane's current shape — three groups that fell out of insertion order rather
than design (recorded at the time in `.control/memlog/settings.md`, and named explicitly in `DEC-014`, which
sat `draft` until the owner's test confirmed it) — does not read the way the owner intended, and surfaced
three further defects the taxonomy question never covered: a `Layout` pane surviving to hold one lone
percentage control, a row's description truncating to an ellipsis nobody can read in full, a toggle sitting
visually higher than its row's keycap, and the pane growing a horizontal scrollbar it should never need. A
fifth finding — typed digits not reaching a percentage field's value by hand, though the stepper buttons and
every existing automated test's accessible-value path both work — is a defect (`DEF-5`) rather than a design
gap, and is `SPEC-4-04`.

## Solution

`DEC-014` (accepted 2026-09-07, same day as this spec) is applied in full: the Shortcuts pane's sixteen
rows re-cut from three groups to five — *Switching* · *Snap half* · *Snap third* · *Snap custom* · *Resize,
move & arrange* — and the `Layout` pane, down to one control since `SPEC-3-01` moved Overlapping Stack's own
toggle out of it, is retired: its `stack_width_percent` control folds inline onto the Overlapping Stack row,
the same `has_percent` shape a `Snap custom` row already uses. Settings ships four panes, not five. Row-level
polish — a hover tooltip instead of a truncated description, one shared vertical centre for a row's control
cluster and its toggle, and a pane that never needs to scroll sideways — closes the remaining live-test
findings. No `FR` or `UC` is added: this reorganises how three already-promised capabilities (`FR-15`
Overlapping Stack, `FR-26` custom-percentage snap, `FR-28` per-action enable/disable) are presented, which is
`.how/` territory, not a new `.what/` promise.

## User Stories

1. As a user scanning the Shortcuts pane, I want actions grouped by what they snap to (half / third /
   custom) rather than by the order they were added in, so that the pane reads as a designed taxonomy
   instead of an implementation accident.
2. As a user looking for the Overlapping Stack width, I want it on the same row as the Overlapping Stack
   on/off toggle, so that I do not have to visit a separate `Layout` pane for one setting connected to a
   control I already found in Shortcuts.
3. As a user whose row description is cut off with `…`, I want to see the full text by hovering or
   focusing the row, so that a long description is never permanently unreadable.
4. As a user comparing a row's toggle and its keycap, I want them visually aligned on the same centre line,
   so that the row reads as one coherent control rather than misaligned pieces.
5. As a user resizing or using the default-sized Settings window, I want the Shortcuts pane to never grow a
   horizontal scrollbar, so that every control stays reachable without sideways scrolling.

## Implementation Decisions

- **The declared sequence changes, and that is a precedence change, not cosmetic.** `ShortcutField::ALL`'s
  order is load-bearing three times over (`LBR-ST-14`): pane draw order, keyboard focus order, and
  chord-collision precedence. `DEC-014` prices this explicitly — `SnapMaximize` moves from position 5 to
  second-to-last, which changes who wins an unlikely future collision. `SPEC-4-01` is scoped to change
  exactly what `DEC-014` names, nothing more.
- **`stack_width_percent` is not a `ShortcutField`.** It has no chord, no enable flag, no collision
  precedence — folding it into `ShortcutRow`'s existing `has_percent` control (rather than inventing a new
  one) is the whole point of `SPEC-4-02`, and every place keyed on `Pane::Layout` (focus order, accessible
  names, sidebar data) must be found and updated in the same ticket, not left dangling.
- **Tooltip, centring, and no-scroll are independent of each other and of the first two tickets**, but
  `SPEC-4-03` is sequenced after `SPEC-4-01`/`SPEC-4-02` so it polishes each row's *final* shape (including
  the `Stack` row's new percent control) exactly once, rather than being redone after the structural tickets
  land.
- **`DEF-5` is a defect, kept out of the three design tickets on purpose.** Every existing automated test
  for percentage-field commit drives the control through `set_accessible_value` (the UI Automation
  `RangeValuePattern` path); none exercises a real keystroke reaching Slint's native `TextInput` entry.
  `SPEC-4-04` root-causes this with `wdi-systematic-debugging` before any fix, per this repo's standing rule
  for a bug with an unknown cause.

## Testing Decisions

A good test here verifies the same three things `SPEC-2-01`/`SPEC-3-01` already established as this
component's style: observable model state (the declared sequence, group membership, label text), the
rendered Slint tree (which control a row shows, its accessible name, whether an element still exists),
and — for `DEF-5` specifically — a real simulated keystroke sequence, not the accessible-value shortcut
every existing test already uses.

Modules to test, and the prior art each follows:

- **Five groups, correct membership, `SnapMaximize`'s new position** — prior art: `app.rs`'s existing
  `grouping_never_reorders_the_declared_sequence` and the declared-order index assertions
  (`ShortcutField::SnapThirdLeft as usize == 7`, etc.). Extended for five groups; the `SnapMaximize`
  assertion inverts rather than being deleted.
- **`Snap custom` labels drop `(custom %)`** — prior art: `ShortcutField::label`/`from_label` round-trip
  test. A new assertion that no `Snap custom` label contains `"(custom"`.
- **`Stack` row gains a percent control, `Layout` pane is gone** — prior art: `SPEC-2-01`'s
  `layout_pane_slint_snapshot.rs` tests for stack-width commit-on-departure and out-of-range refusal; these
  migrate to cover the relocated control rather than being left asserting against a removed pane.
- **Tooltip, centring, no horizontal scroll** — new UI-tree assertions in
  `shortcut_row_slint_snapshot.rs`/a new `shortcuts_pane_slint_snapshot.rs`: the description text is absent
  from the always-rendered tree and present as a tooltip/accessible-description; the control cluster and
  toggle report the same vertical centre; the pane's rendered width at the shell's default window size does
  not exceed it.
- **A real keystroke commits a typed percentage** — new behavior, `SPEC-4-04`. Prior art for the *existing*
  accessible-value path is `SPEC-2-01`'s five commit-on-departure tests, which stay unchanged; the new test
  drives the same field through a simulated key-press sequence instead and must pass independently.

## Out of Scope

- Any change to which chord an action defaults to, or to collision-resolution logic itself — only the
  declared order's effect on precedence, per `DEC-014`.
- Renaming or migrating any `config.toml` key. `stack_width_percent` keeps its name and section; only its
  UI location moves.
- A second taxonomy revision. If five groups reads wrong in practice, that is another `DEC-`, not a
  same-spec adjustment (`DEC-014`'s own Cost section says so).
- Any fix folded into `SPEC-4-01`/`02`/`03` for `DEF-5` "as a side effect" — it needs `SPEC-4-04`'s own
  dedicated test regardless of which ticket happens to touch the same file first.

## Further Notes

`DEC-014` (`.control/decisions/DEC-014-the-shortcuts-pane-regroups-into-five-taxonomic-groups.md`) is the
accepted decision this spec applies in full, including its accepted extension covering the `Layout` pane's
retirement. `.how/settings/01-ux/DESIGN.md`'s "Layout & Navigation Hierarchy" section and
`.how/settings/SDD-settings.md`/`.how/settings/04-components/LC-settings-shell.md`'s pane-routing lines are
already updated to the four-pane, five-group target state this spec builds toward — nothing in `.what/` or
`.how/` remains to be written before a ticket picks this up. `.control/registry/defects.yaml`'s `DEF-5` is
the one finding from the live test that is a defect rather than a design gap.
