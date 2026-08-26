---
type: decision
id: DEC-004
status: applied
touches:
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/04-usecases/UC-4-change-shortcut.md
  - .what/settings/05-scenarios/SCN-01-invalid-shortcut-rejected.md
  - .how/settings/SDD-settings.md
  - .how/window-management/SDD-window-management.md
  - .how/_platform/ARCHITECTURE-SPINE.md
supersedes: null
superseded_by: null
created: '2026-08-26'
---

# DEC-004 — The hook reports the chord it observed, and swallows it only while a field is recording

## Decision

The low-level keyboard hook reports a chord it observed to the Settings window as a virtual-key code
plus a modifier set, and Settings stops deriving a recorded chord from window-system text.

The hook makes **three independent decisions** about a keystroke, and a lease is a named combination
of them rather than a single switch:

- **report** — post the chord to Settings
- **suppress** — do not run Wira Desk's own action for it
- **swallow** — do not pass it on to Windows

| Lease | Armed while | report | suppress | swallow |
|---|---|---|---|---|
| none | outside both leases | no | no | a matched chord only, as today |
| **Observe** | The Shortcuts pane is visible and Settings holds the foreground window | yes | yes | no |
| **Record** | A shortcut field is listening and Settings holds the foreground window | yes | yes | yes |

Record implies observe; observe never implies record. Both report only a non-modifier key-down
carrying at least one modifier, and neither reports a modifier-only press or any key release.
Swallowing implies suppressing: a chord that reaches nothing does not reach Wira Desk either.
Outside both leases nothing is reported and nothing is suppressed.

**Observe is the lease that already exists, given a voice.** The capture lease in the code today is
exactly `report=no, suppress=yes, swallow=no`. Observe adds the report and narrows the arming
condition from the whole Settings window to one pane; its disposition is not new.

The lease is addressed by **process id**, and its level travels in the same message rather than in a
second one: `wParam` carries `0` (none), `1` (observe) or `2` (record), and `lParam` carries the
Settings process id. The hook compares that id against the foreground process id, so a process id is
the value the comparison actually needs; a window handle would have to be converted into one on the
daemon side, and that conversion is a step that can only fail. Two independent booleans are refused
for the same reason a second message is: one level that is last-write-wins cannot contradict itself.

The swallow exists only to record. No chord is ever claimed for a Wira Desk action on the strength
of this decision, and outside the record lease every chord belongs to Windows entirely.

## Why

`DEC-003` refuses shell chords and explains the refusal. Without this decision that explanation
cannot reach the screen. The current sequence, when `Win+1` is pressed while a field is listening:

1. The hook sees the chord, `match_shortcut` returns `None`, and the callback exits `PassToNext`.
2. Explorer receives the chord and runs its shell action.
3. Settings loses the foreground window.
4. The UI toolkit in Settings never receives the key event.
5. `accept_capture` is never called, so `validate_shortcut` is never called, so nothing is refused
   and nothing is said.

The existing capture lease does not close this. It is checked *after* the `match_shortcut` early
return, so it engages only for a chord that is already one of the six configured shortcuts — its job
is to stop Wira Desk eating its own chord while Settings re-records it, and `Win+1` never reaches it.
Its disposition is also the opposite of what is needed: it passes the chord through precisely so the
toolkit can see it, and passing through is what lets Explorer act.

Reporting from the raw virtual-key code also fixes a second reported symptom rather than working
around it. `Alt+Backtick` produces no character text, because Windows routes it as a system key and
the dead-key path yields nothing, which is why the recorder feels dead for the product's own default
fallback chord. A virtual-key code is `0xC0` whether the OS produced a character, a dead key, or
nothing at all. Three heuristics in Settings become unnecessary as a result: the `GetAsyncKeyState`
patch for an unreliable meta flag, the `GetAsyncKeyState` backtick rescue, and the text-to-key-name
ladder. This decision removes code from the crate that carries the most guesswork.

It opens the channel `DEC-002` recorded as missing — *"the daemon has no channel back to the settings
process to deliver it — the only IPC that exists runs the other way"* — without contradicting it.
Nothing is predicted here. The hook reports a keystroke it actually observed, which is the principle
`DEC-002` states.

The observe lease exists for a second reader of the same channel. `DEC-005` needs to know whether the
hook saw a keystroke that the Settings window did not, because that difference is the only honest way
to tell a chord another application merely claimed from a chord another application actually killed.
That reader is not recording anything, so it MUST NOT get the swallow: a lease that swallowed
whenever the Shortcuts pane was open would take the keyboard for as long as a pane stays open, which
is the third alternative below and it stays rejected. Splitting the two capabilities is what keeps it
rejected while still feeding the reader.

Observe suppresses Wira Desk's own action as well, and that is not a detail. A user pressing a
configured chord to see whether the key check registers it would otherwise have the window switch out
from under the Settings window they are reading — the same focus theft this decision exists to stop,
arriving from Wira Desk itself instead of from Explorer. Suppressing costs nothing here because the
chord is being examined rather than used, and the arming condition already requires Settings to hold
the foreground, so no ordinary use of a shortcut falls inside this window.

## Cost

Settings gains the ability to swallow keyboard input, which it did not have before, and a lease left
armed is a keyboard that has stopped responding. Four bounds are load-bearing and none of them is
optional: one owning place per lease for arming and disarming — the record lease tied to the state
that owns listening, the observe lease to the state that owns the visible pane, and neither to
callback wiring; failing closed when Settings is not the foreground window; reaping a dead lease
holder on the heartbeat that already exists, never inside the callback; and never swallowing a chord
Windows keeps regardless, where swallowing cannot help and would misrepresent what happened.

Two leases mean two things that can be left armed, and only one of them can freeze a keyboard. That
asymmetry MUST NOT be read as permission to be casual with the observe lease: an observe lease left
armed after Settings closes keeps the callback posting to a window that no longer exists, which the
heartbeat reaping above is what catches.

The lease comparison moves above `match_shortcut`, so the callback runs it on every non-modifier
key-down rather than only on a matched chord. It stays one comparison and one posted message, with no
allocation, but the budgeted path is no longer entered only on a rare event.

The observe lease widens that again: the callback now posts for ordinary chords typed while the
Shortcuts pane happens to be open, not only for a chord someone is deliberately recording. The bound
is what keeps this acceptable — a non-modifier key-down carrying at least one modifier, only while
Settings holds the foreground window — and that bound is load-bearing rather than a tuning choice.

Passing the chord through under the observe lease means checking a chord another application owns will
fire that application's action. A user finding out why `Alt+1` does nothing may trigger the overlay
that was taking it. That is the price of not swallowing, it is the right side of the trade, and it
MUST be expected rather than discovered.

The hook can report a chord the shared vocabulary cannot name — `Win+Semicolon` yields a virtual-key
code with no canonical form. That has to be an explicit refusal with its own message; leaving it
silent reproduces the unresponsive recorder this decision exists to fix, in a narrower place.

Recording is degraded when the daemon is not running, because there is no hook to report. The
text-derived path stays as a confined fallback and is not deleted, which means two recording paths
exist and the weaker one must be visibly marked rather than silently used.

The foreground process lookup is called directly in the callback today, so the lease branch cannot be
reached by a test at all: it asks the desktop through the identity collector, a harness with no
foreground window gets `0`, and the guard demands non-zero. `DEF-1` already fixed this exact shape one
branch lower by giving the bypass decision a seam, and the lease check sits above that seam and never
benefited from it. That seam is a prerequisite of this change rather than a follow-up to it, and
`DEF-3` records what an untestable lease branch has already cost.

Addressing the lease by process id means Windows may recycle the number. A lease left armed against a
dead process points at an id an unrelated process can later inherit, and the foreground comparison
would then match a process that asked for nothing. Disarming on every exit path and reaping on the
heartbeat shrink that window; neither closes it, and it MUST be carried as a known residual rather
than described as handled.

## Alternatives

**Keep the lease as passthrough and detect the chord from toolkit text.** Lost outright: for `Win+E`
and `Win+D` the event never arrives, so the refusal is undeliverable. That is the reported symptom,
not a residual risk.

**A pre-emptive hint only, with no hook change.** Kept, but as a layer rather than the answer. It
prevents attempts; it cannot explain one that has already happened, and it does nothing for a chord
the hint does not list.

**Swallow for the whole lifetime of the Settings window rather than only while listening.** Refused:
it takes the keyboard for as long as a window the user may leave open stays open, and it buys
nothing — no chord needs recording when no field is listening. The observe lease is not this
alternative arriving through a side door: it reports without swallowing, so the keyboard is never
taken. Any later proposal to let observe swallow is this rejected alternative again.

**One lease that both reports and swallows, arming it wherever a reader needs the report.** Refused,
and it is the shape this decision was first written in. It reads as simpler until the diagnostic
reader of `DEC-005` needs the report while the pane is merely open — at which point one lease means
the keyboard is swallowed for as long as the pane is open. Two capabilities on one lease is what made
that outcome look like a small extension instead of a rejected alternative.

**Address the lease by window handle rather than process id.** It has one real advantage: the daemon
could reap a dead lease with `IsWindow` and would not lean on a heartbeat. Refused because the
comparison the hook actually performs is against a process id, so a handle has to be converted on the
daemon side, and that conversion is where `DEF-3` happened. The reaping advantage is bought back by
the heartbeat that already exists.

## Reversal trigger

Revisit if any of the four bounds cannot be closed without work inside the callback. If the swallow
window can leave the keyboard unresponsive in a way only unbounded callback work would prevent, then
recording shell chords is abandoned and the pre-emptive hint of `DEC-003` becomes the whole answer.

## Trace

Came from the same user testing on 2026-08-26 as `DEC-003`, and from reading the capture lease
against it: the lease was proposed as the place this feature would go, and turned out to be the thing
that has to be inverted for the feature to exist. Widened on the same day, after a second reading
found that the lease had never armed in the first place — `DEF-3`, which is why the process-id
addressing above is part of this decision and not left to the implementation. Depends on `DEC-003`
for its purpose; `DEC-003` depends on it for delivery.
