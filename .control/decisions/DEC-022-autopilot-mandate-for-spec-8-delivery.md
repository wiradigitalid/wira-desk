---
type: mandate
id: DEC-022
status: applied
touches:
  - .control/decisions/DEC-022-autopilot-mandate-for-spec-8-delivery.md
  - .control/memlog/autopilot-DEC-022.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .scratch/smoke-dec-022.md
  - .scratch/spec-8-driverless-mouse-navigation/issues/01-mouse-configuration-and-settings-pane.md
  - .scratch/spec-8-driverless-mouse-navigation/issues/02-low-level-mouse-hook-and-dispatch.md
  - 3p.md
  - crates/daemon/src/arrangement/mod.rs
  - crates/daemon/src/config.rs
  - crates/daemon/src/hook.rs
  - crates/daemon/src/ring.rs
  - crates/daemon/src/worker.rs
  - crates/settings/src/app.rs
  - crates/settings/src/main.rs
  - crates/settings/src/persistence.rs
  - crates/settings/src/theme.rs
  - crates/settings/ui/components/sidebar.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/mouse_pane.slint
  - crates/shared/src/commands.rs
  - crates/shared/src/config.rs
  - crates/shared/src/constants.rs
  - crates/shared/src/lib.rs
supersedes: null
superseded_by: null
created: '2026-09-10'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-10
---

# DEC-022 — Autopilot mandate for SPEC-8 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-7` closed, `SPEC-8` open at 0/2 tickets with its tickets already reviewed (`ef0219a`, edge-case-hunter) —
through every runnable `FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish
or a parked row. Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

In scope: `SPEC-8`'s two tickets:
1. `SPEC-8-01`: Mouse configuration, preset validation, and Settings Mouse pane (`settings`, UC-14, FR-32)
2. `SPEC-8-02`: Low-level mouse hook, dual lifecycle, tilt debounce, and worker dispatch (`window-management`, UC-13, FR-30, FR-31)

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing
by agent, coding executed directly by Claude (claude-byok) with mandatory self code review, independent peer
review executed twice using `cursor-agent --force --model "composer-2.5"`, and safe/un-conflicted application
build in the current repository target to prevent race conditions.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
