---
type: mandate
id: DEC-020
status: applied
touches:
  - .control/decisions/DEC-020-autopilot-mandate-for-spec-6-delivery.md
  - .control/memlog/autopilot-DEC-020.md
  - .control/registry/decisions.yaml
  - .control/registry/defects.yaml
  - .control/registry/specs.yaml
  - .scratch/smoke-dec-020.md
  - .scratch/spec-6-shortcuts-pane-second-follow-up/issues/01-defect-def-16-tooltip-paints-behind-a-later-sibling.md
  - .scratch/spec-6-shortcuts-pane-second-follow-up/issues/02-defect-def-17-stepper-freezes-on-out-of-range-typed-value.md
  - 3p.md
  - crates/settings/src/shortcut_row_slint_snapshot.rs
  - crates/settings/src/shortcuts_pane_slint_snapshot.rs
  - crates/settings/ui/components/shortcut_row.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/shortcuts_pane.slint
supersedes: null
superseded_by: null
created: '2026-09-08'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-08
---

# DEC-020 — Autopilot mandate for SPEC-6 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-5` closed, `SPEC-6` open at 0/2 tickets with its tickets already reviewed (`97c1b1d`, edge-case-hunter) —
through every runnable `FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish
or a parked row. Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

In scope: `SPEC-6`'s two tickets (`SPEC-6-01` to `SPEC-6-02`), delivering follow-up defect fixes `DEF-16`
(tooltip paints behind a later sibling / KeyCheck) and `DEF-17` (out-of-range typed percentage freezes stepper
buttons).

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing
by agent, coding/document review by Claude (claude-byok), and single/safe build location preventing race
conditions.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
