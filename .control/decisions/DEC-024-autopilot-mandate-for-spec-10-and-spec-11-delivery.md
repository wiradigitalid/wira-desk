---
type: mandate
id: DEC-024
status: applied
touches:
  - .control/decisions/DEC-024-autopilot-mandate-for-spec-10-and-spec-11-delivery.md
  - .control/memlog/autopilot-DEC-024.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .scratch/smoke-dec-024.md
  - .scratch/spec-10-mouse-navigation-followup/SPEC.md
  - .scratch/spec-10-mouse-navigation-followup/issues/01-standardize-snap-middle-third-label.md
  - .scratch/spec-10-mouse-navigation-followup/issues/02-dismiss-dropdown-on-pane-switch.md
  - .scratch/spec-10-mouse-navigation-followup/issues/03-daemon-process-lifecycle-and-preset-dispatch.md
  - .scratch/spec-10-mouse-navigation-followup/issues/04-hardware-tilt-lockout-live-verification.md
  - .scratch/spec-11-app-distribution-and-branding/SPEC.md
  - .scratch/spec-11-app-distribution-and-branding/issues/01-static-crt-linking.md
  - .scratch/spec-11-app-distribution-and-branding/issues/02-binary-dependency-ci-guard.md
  - .scratch/spec-11-app-distribution-and-branding/issues/03-about-pane-branding-and-navigation.md
  - .scratch/spec-11-app-distribution-and-branding/issues/04-security-disclosure-and-network-boundary.md
  - 3p.md
  - README.md
  - SECURITY.md
  - docs/threat-model.md
  - packaging/wiradesk.iss
  - scripts/restart-daemon.ps1
  - scripts/verify-release-binary.ps1
  - .github/workflows/release.yml
  - crates/daemon/src/config.rs
  - crates/daemon/src/hook.rs
  - crates/daemon/src/worker.rs
  - crates/settings/src/app.rs
  - crates/settings/src/main.rs
  - crates/settings/src/update.rs
  - crates/settings/ui/components/sidebar.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/panes/about_pane.slint
  - crates/shared/src/binary.rs
  - crates/shared/src/commands.rs
  - crates/shared/src/config.rs
  - crates/shared/src/lib.rs
supersedes: null
superseded_by: null
created: '2026-09-11'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-11
---

# DEC-024 — Autopilot mandate for SPEC-10 and SPEC-11 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-9` closed, `SPEC-10` and `SPEC-11` open with their tickets planned — through every runnable
`FR`/spec/ticket to completion, without further owner check-ins until Finish or a parked row. Parameters
live on this decision's row in `decisions.yaml` under `mandate:`.

In scope:
- `SPEC-10`: Mouse navigation follow-up (4 tickets: SPEC-10-01, SPEC-10-02, SPEC-10-03, SPEC-10-04)
- `SPEC-11`: App distribution and branding (4 tickets: SPEC-11-01, SPEC-11-02, SPEC-11-03, SPEC-11-04)

Execution constraints specified by owner:
1. Coding directly implemented by Claude with mandatory self code review.
2. Independent peer review performed twice: Claude self/sub-agent review + cursor-agent CLI using `composer-2.5` (`cursor-agent --force --model "composer-2.5"`).
3. Automated smoke testing executed by agent.
4. Peer review for coding/documents/analysis delegated to `cursor-agent --force --model "composer-2.5"`.
5. Application builds executed safely in a shared worktree/target to eliminate race conditions.

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing,
dual code review (self + cursor-agent composer-2.5 peer review), and collision-safe builds.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
