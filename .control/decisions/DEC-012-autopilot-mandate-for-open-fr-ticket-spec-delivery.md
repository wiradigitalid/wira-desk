---
type: mandate
id: DEC-012
status: applied
touches:
  - .control/decisions/DEC-012-autopilot-mandate-for-open-fr-ticket-spec-delivery.md
  - .control/memlog/autopilot-DEC-012.md
  - .control/questions/assumptions.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .how/_platform/inventory-api.md
  - .how/settings/SDD-settings.md
  - .how/window-management/04-components/LC-arrangement-engine.md
  - .how/window-management/SDD-window-management.md
  - .what/window-management/SRS-window-management.md
  - 3p.md
  - _bmad-output/specs/spec-1-percentage-and-thirds-snap/issues/01-custom-percentage-edge-snap.md
  - _bmad-output/specs/spec-1-percentage-and-thirds-snap/issues/02-snap-to-thirds.md
  - crates/daemon/src/arrangement/mod.rs
  - crates/daemon/src/arrangement/snap.rs
  - crates/daemon/src/arrangement/thirds.rs
  - crates/daemon/src/config.rs
  - crates/daemon/src/hook.rs
  - crates/daemon/src/worker.rs
  - crates/settings/src/app.rs
  - crates/settings/src/main.rs
  - crates/settings/src/persistence.rs
  - crates/settings/ui/components/shortcut_row.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/shortcuts_pane.slint
  - crates/shared/src/commands.rs
  - crates/shared/src/config.rs
  - crates/shared/src/constants.rs
supersedes: null
superseded_by: null
created: '2026-09-06'
accepted_by: kodesh87, 2026-09-06
---

# DEC-012 — Autopilot mandate for all open FR/ticket/spec delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, G5 next — through
every open `FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish or a
parked row. Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

## Why

The owner wants the remaining work (currently `SPEC-1`: `FR-26`/`FR-27`, percentage/thirds snap on
`window-management`) delivered unattended. Coding and smoke-testing are explicitly assigned to a separate
`claude` CLI profile (`CLAUDE_CONFIG_DIR=~/.claude-byok`), invoked headlessly by the coordinator one call at
a time inside the run's isolated sibling worktree (see `worktree` on this decision's row in
`decisions.yaml`; branch `autopilot/DEC-012`, cut from `main` at `9216f4c`) — so the coordinator's own review/doc/coordination work
never races a build in that worktree. The coordinator does not run `cargo build`/`cargo test`/`./build.ps1`
or launch the app there; those, and the smoke test itself, are claude-byok's alone. The mandate's own `loop`
cadence rides the owner's already-running cron job (`00aa92cd`, every 10 minutes) rather than a second one.

## Cost

If this split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not the
whole run. Until Finish, the run branch and its worktree are the only place ticket work lands; anything
claude-byok produces outside that worktree is not part of this mandate and is not merged by it.
