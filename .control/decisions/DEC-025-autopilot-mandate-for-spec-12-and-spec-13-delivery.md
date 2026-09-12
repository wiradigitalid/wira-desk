---
type: mandate
id: DEC-025
status: accepted
touches:
  - .control/decisions/DEC-025-autopilot-mandate-for-spec-12-and-spec-13-delivery.md
  - .control/memlog/autopilot-DEC-025.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .scratch/spec-12-about-pane-and-settings-polish/SPEC.md
  - .scratch/spec-13-same-app-visual-switcher/SPEC.md
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
4. Peer review for coding/documents/analysis delegated to Claude Sonnet 5 shell-out — mandate: if draft/code touches architecture spine, SRS, SDD, or SPEC, boleh langsung jalankan `wdi-review` dan edit dokumennya sendiri, tidak perlu tanya dulu.
5. Application build/run: satu worktree yang sama dipakai bersama untuk semua build/run di run ini (`worktree: .`).

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing,
dual code review (self + Claude Sonnet 5 high-effort peer review), and collision-safe builds.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
