---
type: decision
id: DEC-005
status: applied
touches:
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/05-scenarios/SCN-01-invalid-shortcut-rejected.md
  - .how/settings/SDD-settings.md
supersedes: null
superseded_by: null
created: '2026-08-26'
---

# DEC-005 — The key check reports what was observed, and never predicts what will happen

## Decision

The key check in the Shortcuts pane reports keystrokes that were actually observed, by correlating
what the daemon's hook saw against what the Settings window received. It renders no verdict about a
chord nobody has pressed, and when the daemon is not running it says so and stops diagnosing rather
than inferring an answer from one signal.

The four correlations it can report, and the only four it may claim:

| Hook saw | Window saw | What it means | Chord usable |
|---|---|---|---|
| yes | yes | Nothing intercepts the chord | yes |
| yes | no | Another application claimed it, but Wira Desk's hook receives it first | **yes** |
| no | no | An earlier low-level hook swallowed it, or Windows consumed it | no |
| no | yes | The daemon is not running, or its hook is dead | not for now |

## Why

A user pressed `Alt+1` and nothing happened, while `Alt+2` worked. The cause was a third-party
overlay taking the chord first, and nothing in the product could say so. `DEC-002` already names this
as the state it knowingly leaves in place: *"a user learns about an external clash by pressing the key
and watching nothing happen"*, and its Cost says the honest answer *"only exists at the moment the key
is pressed"*. This is that moment, and `DEC-004`'s observe lease is the channel that carries it.

The reason it takes two signals rather than one is that a single signal is not merely incomplete — it
is wrong in a way that costs the user a working shortcut. Two different mechanisms produce the same
silence in the Settings window:

- An application holding the chord through `RegisterHotKey` takes it out of every other
  application's input queue, so the window receives nothing. But a low-level hook runs *before*
  hotkey dispatch — the asymmetry `DEC-002` is built on — so Wira Desk's hook still sees it and the
  chord still works.
- An application holding the chord through its own low-level hook installed later in the chain is
  called first and can swallow it outright, and then the chord is genuinely dead.

From the window alone the two are indistinguishable. A check reading only window events would tell a
user to abandon `Alt+1` in the first case, where `Alt+1` functions perfectly — the same
wrong-in-both-directions failure `DEC-002` refused a `RegisterHotKey` probe for. The second row of the
table above is the whole reason this decision exists.

Correlation is deterministic rather than a race. The hook sits earlier in the input path than window
message delivery, so its report always precedes the corresponding window event; nothing has to be
timed or debounced to pair them.

Predicting is refused for the same reason probing was. A green or red badge on each shortcut row,
stating whether a chord is available before anyone presses it, is a probe wearing different clothes: it
would have to guess at the two mechanisms above and would be wrong in both directions exactly as
`DEC-002` records. The line this decision draws is that the product reports observations and never
forecasts.

## Cost

The check is only as truthful as the daemon is present. Without it, three of the four rows collapse
into one and the check must decline to answer — so it goes quiet in one of the states a user might be
trying to diagnose, which is a real gap and not a technicality.

It cannot name the application that took the chord. Windows exposes no supported way to ask who holds
a chord, and `DEC-002` forbids manufacturing an answer, so the copy names categories — GPU software, a
chat client, a game overlay — and never a process. A user who wants the culprit's name does not get
it here.

Correlation is per keystroke and keeps no history. A chord an overlay takes only while a game is
running reads as healthy on every press outside the game, and the check offers nothing that would make
that intermittency visible.

The second row tells a user a chord works while another application also believes it owns that chord.
That is true and it is what they need to know, but the check does not warn that the other
application's own shortcut has stopped working as a result. Wira Desk winning a chord means something
else lost it.

## Alternatives

**A green or red availability badge per shortcut row.** Refused above: prediction, and wrong in both
directions.

**Polling `GetAsyncKeyState` to detect a key that was physically pressed but never delivered, so the
check works without the daemon.** Refused. It reads the driver's key state before delivery, so it
proves a key went down and not that it arrived anywhere; it cannot tell the second row of the table
from the third, which is the distinction the whole check exists to make; and it costs a continuous
poll for an answer the hook already supplies for free. It answers a less useful question at a higher
price.

**A perpetually pulsing liveness indicator.** Refused twice over. It animates whether or not input
arrives, so it proves the renderer is alive rather than that the keyboard reaches the application —
which is the one thing the user is asking. And a perpetual animation is the only element that would
make this pane consume CPU while the keyboard is idle. An indicator that beats once per observed
keystroke costs nothing at rest and proves the right thing.

## Reversal trigger

Revisit if Windows exposes a supported way to ask which process holds a chord: naming the culprit
becomes possible, and the category-based copy turns from vague into wrong. Revisit also if
pass-through under the observe lease means checking a chord fires another application's action often
enough to be a hazard — then diagnosis moves behind a deliberate action instead of being always live.

## Trace

Came from a user report on 2026-08-26 — `Alt+1` dead, `Alt+2` fine, cause invisible — and from the
consultation that followed it. Depends on `DEC-004`'s observe lease for its second signal. Closes the
observational half of `DEC-002`'s Cost, the half `DEC-003` cannot reach: a chord swallowed by an
earlier third-party low-level hook is exactly what `DEC-003`'s curated catalogue never enumerates.
