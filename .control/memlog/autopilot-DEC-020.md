---
artifact: .control/decisions/DEC-020-autopilot-mandate-for-spec-6-delivery.md
skill: wdi-autopilot
date: 2026-09-08
---

# Memlog — autopilot run DEC-020

## Resume

**The run is finished.** This block is its end state, not a step that never came.

- `SPEC-6` closed — both tickets (`SPEC-6-01`, `SPEC-6-02`) `done`. Both defects (`DEF-16`, `DEF-17`) `fixed`.
- Scope: all open tickets and specs completed; promise progress 100% (52/52 counted RTM rows green).
- Run branch: `autopilot/DEC-020`. One PR opened from this branch.
- Suite: **569 passed / 0 failed / 2 ignored**; `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`, `validate.py --generate` all clean.
- Publication hygiene: `scripts/verify-public-export.ps1` passes 10/10 checks.
- Smoke test: automated agent run (`build.ps1 -Mode prod`) completed with 11 PASS / 0 FAIL / 0 NOT VERIFIABLE recorded in `.scratch/smoke-dec-020.md`.
- Blocked: —
- Parked: —
- The loop is cancelled. Mandate `DEC-020` is `applied`. The owner merges; the run never does.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-020 for SPEC-6 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-020 | DEC-020, decisions.yaml |
| Preflight | Build & Worktree safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Iter 1 | SPEC-6-01 / DEF-16 | Elevated floating tooltip to window-level overlay in mica content area, bubbling callbacks through pane hierarchy | Scoped child Rectangle or raw PopupWindow | Tooltip paints behind sibling rows or KeyCheck, or steals focus | crates/settings/ui/components/shortcut_row.slint, crates/settings/ui/panes/shortcuts_pane.slint, crates/settings/ui/main_window.slint, shortcut_row_slint_snapshot.rs, shortcuts_pane_slint_snapshot.rs |
| Iter 1 | SPEC-6-02 / DEF-17 | Clamped base in stepper step_plus/step_minus to [percent_min, percent_max] so out-of-range typed values recover | Unclamped base guard assumption | Stepper buttons freeze on values outside range | crates/settings/ui/components/shortcut_row.slint, shortcut_row_slint_snapshot.rs |
