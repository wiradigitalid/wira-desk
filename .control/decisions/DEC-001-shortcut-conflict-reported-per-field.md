---
type: decision
id: DEC-001
status: applied
touches:
  - .what/settings/05-scenarios/SCN-03-shortcut-collision-refused-at-save.md
  - .what/settings/04-usecases/UC-4-change-shortcut.md
  - .what/settings/02-rules/rules-settings.md
  - .how/settings/SDD-settings.md
supersedes: null
superseded_by: null
created: '2026-08-25'
---

# DEC-001 — A shortcut conflict is reported on the fields that conflict, and Save stays enabled

## Decision

Every shortcut rejection, including one action taking a chord another action already holds, is
reported on the offending field and names both sides; the Save action stays enabled and an
unusable draft is refused when it is submitted.

## Why

SCN-01 settled this for the grammar rejections at G4 and stated the reason in its closing note: a
Save button that disables itself has to explain which field disabled it, so rejection is reported
per field instead. The duplicate-chord class did not exist then — it is newer than the scenario —
and the question of whether Save gates on *it* was answered the other way while the guard was
being built.

That gate reproduced exactly the failure SCN-01 predicted. The owner, using the build, could not
tell which of the six shortcut fields was in conflict; the button was simply dead, and the pane
offered no way to find out. The amber marking and the swap affordance that were meant to carry the
explanation are only legible once the user has already found the two rows involved, which is the
thing they could not do.

The gate was also redundant. `validate_config` already refuses a duplicate before anything is
written and already carries the name of the field it collides with, and that reason already renders
as a sentence naming both actions. Disabling Save suppressed a message that was more informative
than the disabled state that replaced it.

## Cost

A user can submit a draft that will be refused, which costs a round trip that a disabled button
would have prevented. The save-time refusal is now the only thing standing between a conflicting
pair and the configuration file, so the duplicate branch of `validate_config` and the sentence that
renders its reason are load-bearing: if either stops naming both sides, the confusion this decision
exists to remove comes straight back with no gate behind it.
