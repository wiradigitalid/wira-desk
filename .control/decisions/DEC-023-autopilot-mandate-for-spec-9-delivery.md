---
type: mandate
id: DEC-023
status: applied
touches:
  - .control/decisions/DEC-023-autopilot-mandate-for-spec-9-delivery.md
  - .control/memlog/autopilot-DEC-023.md
  - .control/registry/decisions.yaml
  - .control/registry/specs.yaml
  - .scratch/smoke-dec-023.md
  - .scratch/spec-9-mouse-and-settings-polish/SPEC.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/01-debug-trace-rotation-and-appdata-inventory.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/02-toggle-vertical-alignment-and-setting-toggle-row.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/03-card-divider-full-bleed-consistency.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/04-mouse-action-preset-catalog-expansion.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/05-grouped-preset-dropdown-overlay.md
  - .scratch/spec-9-mouse-and-settings-polish/issues/06-tilt-wheel-defaults-and-gesture-lockout.md
  - 3p.md
  - CONTRIBUTING.md
  - docs/README.md
  - crates/daemon/src/hook.rs
  - crates/daemon/src/log.rs
  - crates/daemon/src/util.rs
  - crates/settings/src/app.rs
  - crates/settings/src/main.rs
  - crates/settings/src/persistence.rs
  - crates/settings/ui/components/setting_toggle_row.slint
  - crates/settings/ui/main_window.slint
  - crates/settings/ui/onboarding.slint
  - crates/settings/ui/panes/about_pane.slint
  - crates/settings/ui/panes/general_pane.slint
  - crates/settings/ui/panes/mouse_pane.slint
  - crates/shared/src/config.rs
supersedes: null
superseded_by: null
created: '2026-09-11'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-11
---

# DEC-023 — Autopilot mandate for SPEC-9 delivery

## Decision

`wdi-autopilot` is mandated to carry Wira Desk from its current position — G1-G4 passed, `SPEC-1` through
`SPEC-8` closed, `SPEC-9` open at 0/6 tickets with its tickets already reviewed (`a0750a2`, edge-case-hunter) —
through every runnable `FR`/spec/ticket to a single reviewable PR, without further owner check-ins until Finish
or a parked row. Parameters live on this decision's row in `decisions.yaml` under `mandate:`.

In scope: `SPEC-9`'s six tickets:
1. `SPEC-9-01`: Debug trace log rotation and AppData inventory documentation (`window-management`, UC-13, FR-30)
2. `SPEC-9-02`: Vertical alignment of toggle switches and SettingToggleRow component (`settings`, UC-14, FR-32)
3. `SPEC-9-03`: CardDivider full-bleed consistency across all settings panes (`settings`, UC-14, FR-32)
4. `SPEC-9-04`: Mouse action preset catalog expansion to full window management suite (`settings`, UC-14, FR-30, FR-32)
5. `SPEC-9-05`: Grouped preset dropdown overlay in Settings UI (`settings`, UC-14, FR-32)
6. `SPEC-9-06`: Tilt wheel default inversion and sustained hold gesture lockout (`window-management`, UC-13, FR-30, FR-31)

## Why

The owner requested all open FR/ticket/specs to be completed unattended via loop with automated smoke testing
by agent, coding executed directly by Claude (claude-byok) with mandatory self code review, independent peer
review executed twice using `cursor-agent --force --model "composer-2.5"`, and safe/un-conflicted application
build in the current repository target to prevent race conditions.

## Cost

If the split turns out wrong, the mandate is superseded through a new `DEC-` — one setting changes, not
the whole run. Until Finish, the run branch is the only place ticket work lands, and `main` is reached
through exactly one PR the owner merges.
