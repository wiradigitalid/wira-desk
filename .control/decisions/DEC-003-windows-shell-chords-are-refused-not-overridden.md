---
type: decision
id: DEC-003
status: applied
touches:
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/04-usecases/UC-4-change-shortcut.md
  - .what/settings/05-scenarios/SCN-01-invalid-shortcut-rejected.md
  - .how/settings/SDD-settings.md
supersedes: null
superseded_by: null
created: '2026-08-26'
---

# DEC-003 — A chord the Windows shell owns is refused and explained, never overridden

## Decision

A chord the Windows shell already owns is refused, and the refusal names the Windows function the
user would have lost; it is never claimed for a Wira Desk action, whatever the user asks. The set of
such chords is a curated catalogue carried as data in `shared`, and each entry says which of two
kinds it is: one Windows keeps regardless of any hook, and one Wira Desk could take but will not.

## Why

Overriding a shell chord is technically available. `RegisterHotKey` dispatch happens after the
low-level hook chain, which is the same asymmetry `DEC-002` already records, so `Win+1` could be
swallowed and claimed. It is refused anyway, and one asymmetry decides it:

**An override lives only as long as the daemon does.** Wira Desk is a tray utility that runs all day
and can be stopped, crash, or not yet have auto-started. With `Win+1` bound to a Wira Desk action,
the same keypress snaps a window when the daemon is alive and opens the first taskbar app when it is
not. With `Ctrl+Win+1` bound instead, the failure is that nothing happens — which reads correctly as
*Wira Desk is not running*. The first failure reads as *this application is broken and now does
random things to my computer*, and it teaches the user that Wira Desk sometimes hijacks Windows.
That is a trust cost out of all proportion to a few extra chords in the selection space.

Three supporting reasons:

- The remaining space is already wide. All six shipped defaults use `Ctrl+Win+*`, which is close to
  empty. Refusing shell chords barely narrows real choice.
- A curated refusal ages better than an override. Windows adds shell hotkeys with each release, and
  an override that collides with a *future* one wins silently, taking an OS function the user has
  not learned exists yet.
- The defaults already had this instinct. `snap_half_left` is `ctrl+win+left`, deliberately not
  `win+left`. This decision only raises that instinct into an enforced rule.

Two kinds, not one, because a single policy is dishonest at one end or the other. `Ctrl+Alt+Del`
never reaches the hook chain and `Win+L` is not stopped by swallowing it, so a message that offers
an alternative for those would be a promise nothing can keep. Every other entry can be taken and is
refused as policy, so its message can honestly offer a way through.

Nothing here is probed. The catalogue is written, reviewed, and versioned knowledge about Windows,
not a trial registration, so this decision sits inside `DEC-002` rather than against it. It also
answers a hole the corpus already named: `SDD-settings.md` records that `is_reserved_system_shortcut`
refuses five combinations, so `Win+Shift+S` or `Win+V` is accepted and then permanently unreachable —
and `OQ-9` records that nothing tracked the reserved set drifting at all.

## Cost

The catalogue is a maintenance surface, and an entry never added is a chord accepted and then dead
on arrival — the exact silence `DEC-002` records as its own cost. This decision narrows that silence
to what has been enumerated; it does not end it, and it MUST NOT be described as ending it. What
lies outside the catalogue — a chord a third-party low-level hook swallows before the daemon sees
it — is reachable only by observing a real keypress, which is `DEC-005`, not by any list.

A user who genuinely wants `Win+1` has no route to it. There is no override affordance, by
construction, and adding one later contradicts this decision rather than extending it.

The refusal cannot reach the user at the moment of capture without `DEC-004`. Until that lands, a
shell chord pressed while a field is listening still steals focus and produces no message, and the
refusal only surfaces on the save path and through a pre-emptive hint. Shipping this decision alone
therefore fixes the configuration file and not the reported symptom.

`Alt+Space`, `Alt+Esc`, and `Alt+Backtick` stay allowed on purpose. `Alt+Backtick` is this product's
own `switcher.fallback_shortcut` default, so the catalogue can never grow to include it without the
shipped default failing its own validation.

## Alternatives

**A soft warning with an explicit override.** The first shape considered, and the one that loses to
the daemon-liveness asymmetry above: it makes one chord do two different things depending on whether
a background process is alive, and neither state is visible to the user at the moment they press it.

**Discovering the set dynamically by trial `RegisterHotKey`.** Refused by `DEC-002`, which already
records that the probe is wrong in both directions.

**One uniform policy for every reserved chord.** Refused: it forces either an alternative offered
for `Win+L` that cannot work, or a flat refusal for `Win+1` that misrepresents a chord which
functions perfectly. The two kinds exist to avoid both lies.

## Reversal trigger

Revisit if Windows exposes a supported way to hold a chord ahead of the shell — that removes the
liveness asymmetry this decision rests on. Revisit also if the catalogue starts needing per-version
branching to stay correct, because that makes a single curated list the wrong shape rather than
merely a more expensive one.

## Trace

Came from user testing on 2026-08-26: `Win+1` and `Win+E` pressed while a shortcut field was
listening caused Explorer to act and steal focus, with no message anywhere. Anticipated in the
corpus by `OQ-5` (the rejection's UX has no written authority) and `OQ-9` (the reserved set can
drift and nothing tracks it); both close when this decision reaches `applied`.
