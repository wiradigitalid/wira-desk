---
type: mandate
id: DEC-016
status: applied
touches:
  - .control/decisions/DEC-016-autopilot-mandate-for-spec-2-spec-3-delivery.md
  - .control/memlog/autopilot-DEC-016.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .how/_platform/design-system.md
  - .how/_platform/inventory-db.md
  - .how/settings/04-components/LC-settings-shell.md
  - .how/settings/SDD-settings.md
  - .scratch/builder-brief-spec-2-01-fix1.md
  - .scratch/builder-brief-spec-2-01.md
  - .scratch/builder-brief-spec-3-01.md
  - .scratch/builder-brief-spec-3-02.md
  - .scratch/smoke-test-dec-016.md
  - .scratch/spec-2-shortcuts-pane-fixes/issues/01-commit-on-departure-and-stepper-parity.md
  - .scratch/spec-3-per-action-shortcut-enable-disable/issues/01-per-action-enable-flag.md
  - .scratch/spec-3-per-action-shortcut-enable-disable/issues/02-hook-registration-exclusion.md
  - .what/business-rules.md
  - 3p.md
  - Cargo.lock
  - crates/daemon/src/arrangement/mod.rs
  - crates/daemon/src/arrangement/stack.rs
  - crates/daemon/src/config.rs
  - crates/daemon/src/hook.rs
  - crates/settings/Cargo.toml
  - crates/settings/build.rs
  - crates/settings/src/app.rs
  - crates/settings/src/layout_pane_slint_snapshot.rs
  - crates/settings/src/main.rs
  - crates/settings/src/persistence.rs
  - crates/settings/src/shortcut_row_slint_snapshot.rs
  - crates/settings/src/theme.rs
  - crates/settings/ui/components/shortcut_row.slint
  - crates/settings/ui/components/toggle_switch.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/layout_pane.slint
  - crates/settings/ui/panes/shortcuts_pane.slint
  - crates/shared/src/config.rs
  - crates/shared/src/constants.rs
supersedes: null
superseded_by: null
created: '2026-09-07'
accepted_by: kodesh87, 2026-09-07
---

# DEC-016 — Autopilot mandate for SPEC-2/SPEC-3 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-2` and
`SPEC-3` open, `SPEC-2-02` (`UC-8`/`FR-25`) already closed as a preflight backfill — through every open
`FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish or a parked row.
Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

## Why

The owner wants `SPEC-2` (Shortcuts pane defect fixes) and `SPEC-3` (`FR-28`/`FR-29`, per-action
shortcut enable/disable) delivered unattended, following the same split `DEC-012` already proved out:
coding and smoke-testing are assigned to a separate `claude` CLI profile (`CLAUDE_CONFIG_DIR=~/.claude-byok`),
invoked headlessly by the coordinator inside the run's isolated sibling worktree (see `worktree` on this
decision's row in `decisions.yaml`; branch `autopilot/DEC-016`, cut from `main` at `0d3eb1f`), so the
coordinator's own review/doc/coordination work never races a build there. The coordinator does not run
`cargo build`/`cargo test`/`./build.ps1` or launch the app in the shared worktree; those, and the smoke
test itself, are claude-byok's alone. The mandate's `loop` cadence rides the owner's own cron job started
this turn (10 minutes) rather than a second one.

## Cost

If this split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch and its worktree are the only place ticket work lands;
anything claude-byok produces outside that worktree is not part of this mandate and is not merged by it.
