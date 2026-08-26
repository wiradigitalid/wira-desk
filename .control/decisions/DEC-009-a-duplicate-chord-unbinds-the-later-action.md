---
type: decision
id: DEC-009
status: applied
touches:
  - .what/business-rules.md
  - .control/product-glossary.md
  - .what/window-management/03-domain/domain-model.md
  - .what/window-management/05-scenarios/SCN-03-duplicate-chord-unbinds-later-action.md
  - .what/window-management/SRS-window-management.md
  - .how/window-management/SDD-window-management.md
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/SRS-settings.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/settings/05-model/data-model.md
supersedes: null
superseded_by: null
created: '2026-08-26'
accepted: '2026-08-26'
accepted_by: Product Owner (in session)
applied: '2026-08-26'
---

# DEC-009 — A duplicate chord unbinds the later action at startup, and refuses the whole reload

## Decision

**Two actions configured to the same chord is a defined condition with two different answers,
chosen by whether a last-known-good configuration exists.**

**At startup** — `hook::load_shortcuts`, where there is no previous configuration to fall back to —
the chord is kept by the field that comes **first in a fixed precedence order**, the later field is
**unbound**, and exactly one Tier-2 warning names both fields and the chord. An unbound field matches
nothing: its action is unreachable until the user changes one of the two, and no other action fires in
its place.

The precedence order is fixed, and it is the declaration order of the fields:

```
switcher.shortcut
switcher.fallback_shortcut
snapping.snap_half_left
snapping.snap_half_right
snapping.snap_half_top
snapping.snap_half_bottom
snapping.snap_maximize
layout.move_next_monitor_shortcut
layout.stack_shortcut
```

**At reload** — `config::validate`, reached only from `WM_APP_RELOAD_CONFIG` — a duplicate is a
rejection reason, `RejectReason::DuplicateShortcut`. The whole candidate configuration is refused,
every actor stays on its last-known-good snapshot, and one Tier-2 warning is emitted. This is the
existing all-or-nothing reject contract, extended by one reason rather than amended.

**The precedence order preserves what `decode_command` already does, and extends it.** Its
`if / else if` chain resolves a chord to the first matching field, so on a duplicate the earlier field
already wins — and the relative order of the fields that exist today (left, right, maximize, stack) is
unchanged above. What this decision adds is the *placement of the three new fields* inside that order,
and the warning that was missing. It makes the existing behaviour **specified** rather than incidental;
it does not claim the new placements were already settled somewhere.

## Why

The condition is not hypothetical, and it is being created by this very batch of work. Duplicate
detection exists in exactly one place — `settings::persistence`, on the Save path — so Settings cannot
write a colliding pair. `daemon::config::validate` does not check for duplicates at all, and
`load_shortcuts` does not either. Until now that gap was harmless, because every shipped default was
distinct and Settings guarded every user-authored change.

`DEC-008` breaks that. `snapping.snap_half_bottom` arrives with the default `ctrl+alt+down`, and any
existing `config.toml` that already sets `layout.stack_shortcut = "ctrl+alt+down"` — the owner's own
file does — becomes a colliding configuration on first launch after upgrade, with no user action
involved. Without this decision, the daemon loads both, `decode_command`'s chain silently picks snap
bottom, and the stack chord stops working with nothing anywhere saying why. "It silently stopped
working after an update" is the worst available outcome and it was on the default path.

**Why unbind rather than reject at startup.** Rejecting means one of two things, and both are worse
than losing one action. Either the daemon refuses to start, which turns a chord clash into an outage;
or it falls back to full defaults, which discards every customisation the user has — their switcher
chord, their VM bypass list, their auto-start — as the price of one ambiguous pair. Unbinding is
proportionate: the user loses exactly the thing that was ambiguous and keeps everything that was not.

**Why reject rather than unbind at reload.** The reject contract in `daemon/config.rs` is explicit
that a partially applied reload is worse than a refused one, because the user has no way to tell which
half took effect. That reasoning holds here exactly. A reload also has the two things startup lacks: a
last-known-good configuration to keep, and a human who just pressed Save and is looking at the result.
And because Settings cannot produce a duplicate, one arriving on reload means the file was hand-edited
— a case where refusing and saying so is the honest answer, not a case to be quietly repaired.

**Why declaration order rather than something cleverer.** Any ordering is arbitrary; the value is that
it is *fixed and written down*. Declaration order is the one ordering a reader can verify against the
source in a few seconds, and it is the one already in force. A rule like "the more specific action
wins" would require ranking snap against stack, which nobody can do defensibly.

## Cost

**An action becomes silently unreachable.** The warning lands in `wiradesk.log`, which no user is
reading. From the keyboard, "this chord is unbound because it collides" and "this feature is broken"
are indistinguishable. This is the same silence `DEC-002` accepted for unknowable external claims and
`DEC-006` accepted for a refused target, and it is a real cost rather than a technicality — it is
being accepted for the third time, which is worth noticing.

**Two behaviours for one condition.** Startup unbinds; reload refuses. A reader who learns one will be
surprised by the other, and the justification lives in this file rather than at either call site. The
mitigation is that both call sites must name this decision in a comment; that is an obligation this
file creates and cannot itself discharge.

**Precedence makes an arbitrary field privileged.** `snapping.snap_half_bottom` outranks
`layout.stack_shortcut` for no reason a user would guess, and on the exact collision `DEC-008`
creates, the *new* feature wins and the *existing* one goes dark. That is the least defensible
consequence of this decision and it is the one that will actually happen on the owner's machine.

**Detection is one more thing `validate` can refuse for.** `RejectReason` grows to five variants, each
needing its own honest message, and the settings-side error text must stay consistent with the
daemon-side one for the same condition.

## Alternatives

**Make the new defaults collision-proof by choosing chords nothing legacy uses.** Refused, and it is
worth being clear why since it looks like it dissolves the problem. It cannot: a user may have
configured any chord for any field, so no default is provably free. It would also mean choosing
`snap_half_bottom`'s default to dodge one known file rather than to fit the keymap, which is designing
the product around one installation.

**Unbind at reload too, for one uniform behaviour.** Refused. It contradicts the reject contract's
stated reasoning, and it silently degrades a configuration the user just saved by hand — which is the
one moment they are entitled to a straight answer.

**Reject at startup too, for one uniform behaviour.** Refused for the reason under Why: it spends
every other setting the user owns to resolve one pair.

**Resolve by "last wins" instead of "first wins".** Refused. It would require changing
`decode_command`'s chain to match, and it makes the newest field in the schema quietly outrank
everything — the same privilege problem, pointed the other way, plus a code change with no
compensating benefit.

**Have the daemon rewrite the colliding field to a free chord.** Refused. The daemon does not write
`config.toml`; only Settings does, and adding a second writer introduces the half-written-file and
lost-update problems the atomic write in `Config::save` exists to prevent.

## Reversal trigger

Revisit the silence if the unbind is reported as a bug — by anyone, including its author. The honest
fix is visible: Settings showing the collision on the fields that collide, which is `DEC-001`'s shape
already built and merely not reached from a daemon-side detection. That is a `settings` change this
decision deliberately does not make.

Revisit the precedence order if the collision `DEC-008` creates turns out to be common rather than
confined to installations that had customised the stack chord.

## Trace

Came from reading the code while planning `DEC-008`, not from a report: `settings/src/persistence.rs`
has duplicate detection, `daemon/src/config.rs` does not, and `daemon/src/hook.rs`'s `decode_command`
is an `if / else if` chain whose first-match behaviour was never specified anywhere. The collision was
then confirmed against the owner's own `%APPDATA%\WiraDesk\config.toml`.

Extends the reject contract in `daemon/config.rs` by one reason without changing its all-or-nothing
shape. Sits beside `DEC-001`, which fixed the same class of problem on the Settings side and is the
named route out of the silence this decision accepts. Depended on by `DEC-008`, which creates the
first real instance of the condition.
