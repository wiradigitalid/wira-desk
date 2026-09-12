---
type: mandate
id: DEC-025
status: applied
touches:
  - .control/decisions/DEC-025-autopilot-mandate-for-spec-12-and-spec-13-delivery.md
  - .control/memlog/autopilot-DEC-025.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .scratch/smoke-dec-025.md
  - .scratch/spec-12-about-pane-and-settings-polish/SPEC.md
  - .scratch/spec-13-same-app-visual-switcher/SPEC.md
  - 3p.md
  - crates/daemon/Cargo.toml
  - crates/daemon/src/arrangement/mod.rs
  - crates/daemon/src/config.rs
  - crates/daemon/src/hook.rs
  - crates/daemon/src/main.rs
  - crates/daemon/src/switcher/layout.rs
  - crates/daemon/src/switcher/mod.rs
  - crates/daemon/src/switcher/overlay.rs
  - crates/daemon/src/switcher/selection.rs
  - crates/daemon/src/switcher/thumbnail.rs
  - crates/daemon/src/tray.rs
  - crates/daemon/src/worker.rs
  - crates/settings/src/app.rs
  - crates/settings/src/main.rs
  - crates/settings/src/persistence.rs
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/about_pane.slint
  - crates/settings/ui/panes/general_pane.slint
  - crates/shared/src/commands.rs
  - crates/shared/src/config.rs
supersedes: null
superseded_by: null
created: '2026-09-12'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-12
---

# DEC-025 — Autopilot mandate for SPEC-12 and SPEC-13 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-11` closed, `SPEC-12` and `SPEC-13` open with their tickets planned — through every runnable
`FR`/spec/ticket to completion, without further owner check-ins until Finish or a parked row. Parameters
live on this decision's row in `decisions.yaml` under `mandate:`.

In scope:
- `SPEC-12`: About pane and settings polish (3 tickets: SPEC-12-01, SPEC-12-02, SPEC-12-03)
- `SPEC-13`: Same-app visual switcher (open tickets planned in SPEC-13)

Execution constraints specified by owner:
1. Coding directly implemented by coordinator directly with mandatory self code review.
2. Code review performed 2 times: self code review (in-session default, no separate dispatch) + independent peer review shell-out via:
   `$env:ANTHROPIC_BASE_URL = $null; $env:ANTHROPIC_API_KEY = $null; $env:ANTHROPIC_AUTH_TOKEN = $null; $env:CLAUDE_CONFIG_DIR = "$HOME\.claude"; claude --model claude-sonnet-5 --effort high --dangerously-skip-permissions -p "<prompt>"`
3. Automated smoke testing executed by agent.
4. Peer review for coding/documents/analysis delegated to Claude Sonnet 5 shell-out — mandate: if draft/code touches architecture spine, SRS, SDD, or SPEC, run `wdi-review` and edit documents directly without prompting.
5. Application build/run: one shared worktree used across all build/run operations in this run (`worktree: .`).

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing,
dual code review (self + Claude Sonnet 5 high-effort peer review), and collision-safe builds.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
