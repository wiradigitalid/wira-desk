---
type: mandate
id: DEC-021
status: accepted
touches:
  - .control/decisions/DEC-021-autopilot-mandate-for-spec-7-delivery.md
  - .control/memlog/autopilot-DEC-021.md
  - .control/registry/decisions.yaml
  - .control/registry/defects.yaml
  - .control/registry/specs.yaml
  - .scratch/spec-7-shortcuts-pane-third-follow-up/issues/01-defect-def-18-percentage-not-reverted-on-enter-or-blur.md
  - crates/settings/ui/components/shortcut_row.slint
supersedes: null
superseded_by: null
created: '2026-09-08'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-08
---

# DEC-021 — Autopilot mandate for SPEC-7 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-6` closed, `SPEC-7` open at 0/1 tickets with its tickets already reviewed (`05602b6`, edge-case-hunter) —
through every runnable `FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish
or a parked row. Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

In scope: `SPEC-7`'s one ticket (`SPEC-7-01`), delivering follow-up defect fix `DEF-18`
(an out-of-range typed percentage is not reverted at Enter/blur, only at save).

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing
by agent, coding/document review by Claude (claude-byok), and single/safe build location preventing race
conditions.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
