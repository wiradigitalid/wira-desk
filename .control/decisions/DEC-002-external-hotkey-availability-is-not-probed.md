---
type: decision
id: DEC-002
status: applied
touches:
  - .how/settings/SDD-settings.md
supersedes: null
superseded_by: null
created: '2026-08-25'
---

# DEC-002 — Whether another application already holds a chord is never predicted by probing the operating system

## Decision

Wira Desk does not test a chord against the operating system to predict whether it is available;
a clash with another application is surfaced from what the running hook actually observes, never
from a trial registration.

## Why

Probing means calling `RegisterHotKey` for the captured chord, seeing whether it is refused, and
releasing it again. It is the obvious move, it was proposed, and it is wrong here — because this
product deliberately does not dispatch through `RegisterHotKey`. The addendum records that choice
and its reason: registration is first-come-first-served, so a third-party application that got
there first would take the chord away. `WH_KEYBOARD_LL` was chosen precisely to escape that.

The same asymmetry destroys the probe. A low-level keyboard hook is called before a registered
hotkey is dispatched, so a chord some other application holds through `RegisterHotKey` still
reaches Wira Desk first and still works — the probe would refuse a chord that functions perfectly.
In the other direction, an application that intercepts through its own low-level hook installed
earlier in the chain registers nothing at all, so the probe sees a free chord and says yes to one
that will never fire.

Wrong in both directions, and both directions produce the symptom the probe was proposed to cure:
a combination that cannot be used for a reason the user is never told. A check that manufactures
that experience cannot be the fix for it.

## Cost

There is no answer at the moment of editing to *is anything else already using this*. The honest
answer only exists at the moment the key is pressed, and the daemon has no channel back to the
settings process to deliver it — the only IPC that exists runs the other way. Until that gap is
closed by its own decision, a user learns about an external clash by pressing the key and watching
nothing happen, which is the state this decision knowingly leaves in place rather than papering
over.
