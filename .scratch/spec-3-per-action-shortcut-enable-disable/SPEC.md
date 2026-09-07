---
spec: SPEC-3
release: "0.3.0"
prd: wira-desk
fr: [FR-28, FR-29]
status: ready-for-agent
---

# SPEC-3 — Per-action shortcut enable/disable

## Problem Statement

Overlapping Stack is the only one of sixteen editable shortcut actions with an on/off control
(`layout.enable_overlapping_stack`), and no `FR`, `UC`, or `DEC` explains why it alone has one. Every other
action's chord is permanently claimed from Windows and every other application, whether or not the user
wants that feature — there is no way to free `Ctrl+Alt+1` for another application if the user never uses
snap-to-thirds. Worse, the one existing toggle does not actually free anything: `layout.enable_overlapping_stack`
is checked inside the arrangement planner (`crates/daemon/src/arrangement/stack.rs:15`), after the
low-level keyboard hook has already consumed the keystroke — "disabled" still steals the chord, it just
does nothing with it.

## Solution

Every one of the sixteen editable shortcut actions gains an explicit, persisted on/off control
(`FR-28`, `UC-11`), and turning one off excludes its chord from keyboard-hook registration entirely rather
than registering it and discarding the match (`FR-29`, `UC-12`). Overlapping Stack's existing toggle is
superseded by this mechanism, not kept alongside it — one on/off concept, not two.

A disabled action is a different concept from an `unbound` one, and the two must never be presented as the
same state to the user, even though both currently resolve to the identical "chord absent from hook
registration" wire representation at the daemon boundary (`Option<Shortcut> == None`, confirmed by reading
`crates/daemon/src/hook.rs`'s `Chords`/`match_shortcut`/`unbind_duplicates`). `disabled` is a preference the
user set on purpose, owned by `settings`. `unbound` is a runtime derivation `window-management` computes
when a chord collides with an earlier action (`BR-6`, `DEC-009`), owned by `window-management`. `BR-9`
records the split.

## User Stories

1. As a user who never uses snap-to-thirds, I want to turn those three actions off, so that `Ctrl+Alt+1/2/3`
   is free for another application to bind instead of being permanently claimed and unused.
2. As a user turning an action back on, I want it to remember the chord it held before I disabled it, so
   that I do not have to re-enter a shortcut I already chose once.
3. As a user looking at a row that shows no chord taking effect, I want to tell at a glance whether I turned
   it off myself or whether it lost a chord collision, so that a real misconfiguration is never mistaken for
   a setting I intentionally made.
4. As a user upgrading from a version before this feature existed, I want every action to load as enabled
   exactly as it already behaved, so that upgrading never silently turns my shortcuts off.
5. As a user who disables an action that shares a chord with another action, I want that chord fully
   available to the other action with no collision warning, so that disabling one action can free its chord
   for another I configure afterward.

## Implementation Decisions

- **A persisted flag per action, independent of the chord.** `shared::Config` gains sixteen new boolean
  fields, one per `ShortcutField`, alongside the existing chord-string fields. Disabling an action never
  clears or invalidates its stored chord; `validate_shortcut`'s existing empty-chord rejection is untouched
  — a separate flag, not a relaxed string check.
- **Every new field's `Default` arm sets `true`, not Rust's primitive `bool` default of `false`.**
  `shared::Config`'s structs carry container-level `#[serde(default)]`, which fills a field missing from a
  loaded TOML file from that struct's own hand-written `Default::default()` — never from the primitive
  type default. Getting this wrong on any one of the sixteen fields silently disables that action for every
  config file written before this feature existed.
- **Exclusion happens at `Chords` construction, before matching.** `crates/daemon/src/hook.rs`'s `Chords`
  struct is already `Option<Shortcut>` per field, and a chord already absent from it already takes no part
  in `match_shortcut` or `unbind_duplicates` — both concepts collapse to the same `None` representation at
  this layer today. The only change is that building `Chords` from a config snapshot skips a disabled
  field's chord; no change to matching or collision-resolution logic itself.
- **A disabled action is never a collision candidate.** `crates/settings/src/persistence.rs`'s
  `validate_config`/`find_conflict` exclude a disabled field's chord from the duplicate-chord scan entirely
  — it can neither win nor lose a collision it no longer contends for.
- **Overlapping Stack's existing toggle is superseded, not kept alongside this mechanism.**
  `layout.enable_overlapping_stack` and its check inside `crates/daemon/src/arrangement/stack.rs`'s planner
  are removed once Overlapping Stack's own row carries the new general-purpose enabled flag; a window-manager
  reader should find one on/off concept, not two overlapping ones.
- **A config that explicitly turned Overlapping Stack off migrates that choice to the new flag.** Removing
  `layout.enable_overlapping_stack` outright would silently re-enable it for anyone who had set it to
  `false` — the field simply disappears and the new one defaults `true` like every other action. On first
  load of a config still carrying the old key, its value seeds the new field's default for that one load
  (`false` stays off, `true` or absent behaves exactly as the universal `true` default already would); the
  old key is dropped from the file on next save, the same way any retired field already leaves a saved
  config today. This is a one-time seed, not an ongoing dual-field reconciliation.
- **The UI toggle is a new control on each Shortcuts-pane row**, not a design decided here — `wdi-ux`'s
  domain per the existing "no UI layout decision in a spec" boundary (`SPEC-1`'s own Out of Scope set the
  same precedent). This spec only requires that a disabled row is visibly distinct from an unbound one; the
  exact visual treatment is the builder's to choose within that constraint, consistent with how every other
  Slint control in this pane was built without a spec dictating pixels.

## Testing Decisions

A good test here verifies observable state — what `Chords` contains after excluding a disabled field, what
`find_conflict` reports for a disabled row, what a config round-trip preserves — never the internal call
sequence. This is the same style the existing collision/precedence tests already use.

Modules to test, and the prior art each follows:

- **Chords construction excludes a disabled field** — prior art: the existing test suite around
  `Chords`/`in_declared_order`/`match_shortcut`. A disabled field's chord must be absent from the
  constructed `Chords`, and a keystroke matching that chord must fall through to `CallNextHookEx` exactly
  like an unconfigured one.
- **A disabled field is excluded from collision detection** — prior art: the existing
  `find_conflict`/`validate_config` duplicate-chord tests. A disabled field sharing a chord with an enabled
  field must produce no collision; the enabled field must register normally.
- **Default is `true` for a config predating this feature** — prior art: the existing test loading a
  config file that omits later-added fields and asserting they take their `Default` value (the same pattern
  already covering, e.g., `snap_top`/`snap_bottom` added after the original freeze). Every one of the
  sixteen new fields needs this coverage, not a representative sample.
- **Configuration round-trip** — prior art: the existing test verifying every configuration field survives
  a save-and-reload unchanged. The sixteen new flags need the same coverage.
- **Migrating an explicit old toggle value** — new behavior. Loading a config with
  `layout.enable_overlapping_stack = false` and no new flag present must seed Overlapping Stack's row as
  disabled on that load; loading one with the old key `true` or absent must load it enabled, identically to
  every other action's default.
- **Re-enabling restores the prior chord** — prior art: none directly; this is new behavior. Test that
  toggling a field off then on, with no other edit, leaves its chord string exactly as it was.
- **Frozen defaults** — prior art: the existing test pinning every shipped shortcut default. Extended to
  assert every action's default enabled state is `true`.

## Out of Scope

- The exact visual design of the on/off control, and whether it reads as a switch, checkbox, or otherwise —
  `wdi-ux`'s to decide, not settled here.
- Any change to which chord an action defaults to. This spec only adds an on/off dimension; no default
  chord changes.
- The Shortcuts-pane taxonomy/grouping question (`DEC-014`, still `draft`) — explicitly separate, gated on
  the owner's own accept.
- Bulk enable/disable (an "enable all" / "disable all" action). One control per row only.

## Further Notes

`BR-9`, `LBR-ST-17` in `.what/settings/02-rules/rules-settings.md`, `UC-11`
(`.what/settings/04-usecases/UC-11-turn-a-shortcut-action-on-or-off.md`), and `UC-12`
(`.what/window-management/04-usecases/UC-12-disabled-action-chord-not-claimed.md`) are already landed, along
with `[MISSING]`-marked design amendments in both components' SDDs, `LC-config-writer.md`, and
`LC-hook-thread.md`. Nothing in `.what/` or `.how/` remains to be written before a ticket picks this up.
