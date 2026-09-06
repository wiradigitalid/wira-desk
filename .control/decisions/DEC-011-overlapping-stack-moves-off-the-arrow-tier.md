---
type: decision
id: DEC-011
status: applied
touches:
  - .how/_platform/cross-cutting.md
supersedes: null
superseded_by: null
created: '2026-09-06'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-06
---

# DEC-011 — Overlapping Stack's default shortcut moves from Ctrl+Alt+Shift+Down to Ctrl+Alt+Shift+S

## Decision

`layout.stack_shortcut`'s shipped default moves from `ctrl+alt+shift+down` to `ctrl+alt+shift+s`, freeing
every arrow key under `Ctrl+Alt+Shift` for a new arrangement tier.

## Why

Two new window-management features — a user-configurable-percentage snap to each screen edge, and a
snap to left/middle/right thirds — need shipped default chords of their own. `Ctrl+Alt+Arrow` is already
committed to the fixed halves (`DEC-008`), and `Ctrl+Alt+Shift+Down` is already the shipped default for
`layout.stack_shortcut`, also set by `DEC-008`. Freeing `Down` is cheaper than inventing a fourth modifier
family: no letter or digit chord is claimed by any current default (`SwitcherConfig`, `SnappingConfig`,
`LayoutConfig` in `crates/shared/src/config.rs`), and `S` reads as a mnemonic for the feature it triggers.

## Cost

An install whose `config.toml` still carries the retired default is not migrated. Once the new
percentage-snap field is declared ahead of `layout.stack_shortcut` in the one declared sequence
(`LBR-ST-14`), that install's chord resolves to the new field at startup, `stack_shortcut` is left
unbound, and one Tier-2 warning fires — the exact mechanism `DEC-009` already built and already priced
once for this same field. No value-matching rewrite is added to spare that install: `DEC-009`'s own
Alternatives already refused that idea, because a stored chord cannot be told apart from a chord the user
chose on purpose.

`DEC-008`'s stated modifier semantics — "adding Shift means the variant that reaches wider than one
window or one screen" — stop describing every field under that tier once a same-window, same-screen
percentage snap also claims it. `DEC-008` is `applied` and is not edited to say so; this file is where
that drift is recorded instead, the same way `DEC-008` itself recorded a premise it moved in `DEC-006`
without editing `DEC-006`.

A fresh install now learns `S` for stacking rather than the more mnemonic arrow-down; any onboarding or
tutorial copy naming the old chord needs its own update, which is `wdi-ux`'s and `wdi-component`'s, not
this file's.
