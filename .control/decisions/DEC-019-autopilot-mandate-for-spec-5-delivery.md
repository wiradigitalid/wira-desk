---
type: mandate
id: DEC-019
status: accepted
touches:
  - .control/decisions/DEC-019-autopilot-mandate-for-spec-5-delivery.md
  - .control/memlog/autopilot-DEC-019.md
  - .control/registry/decisions.yaml
supersedes: null
superseded_by: null
created: '2026-09-08'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-08
---

# DEC-019 — Autopilot mandate for SPEC-5 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1`
through `SPEC-4` closed, `SPEC-5` open at 0/3 tickets with its tickets already reviewed (`55f7b3b`,
edge-case-hunter) — through every runnable `FR`/spec/ticket to a single reviewable PR, without further
owner check-ins until Finish or a parked row. Parameters live on this decision's row in `decisions.yaml`
under `mandate:`.

In scope: `SPEC-5`'s three tickets (`SPEC-5-01` to `SPEC-5-03`), delivering follow-up defect fixes
`DEF-14` (tooltip oversized/overlaps title), `DEF-15` (row height toggle jump), and `DEF-13`
(percentage field reverts mid-keystroke).

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke
testing by agent. Build safety constraint: ensure single build location and prevent concurrent builds /
race conditions by pinning `CARGO_TARGET_DIR` to the warm workspace target and serializing builds.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch and its worktree are the only place ticket work lands, and
`main` is reached through exactly one PR the owner merges.
