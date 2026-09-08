---
artifact: .control/decisions/DEC-021-autopilot-mandate-for-spec-7-delivery.md
skill: wdi-autopilot
date: 2026-09-08
---

# Memlog — autopilot run DEC-021

## Resume

**The run is finished.** This block is its end state, not a step that never came.

- `SPEC-7` closed — ticket `SPEC-7-01` `done`. Defect `DEF-18` `fixed`.
- Scope: all open tickets and specs completed; promise progress 100% (52/52 counted RTM rows green).
- Run branch: `autopilot/DEC-021`. One PR opened from this branch.
- Suite: **573 passed / 0 failed / 2 ignored**; `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`, `validate.py --generate` all clean.
- Publication hygiene: `scripts/verify-public-export.ps1` passes 10/10 checks.
- Smoke test: automated agent run (`build.ps1 -Mode prod`) completed with 10 PASS / 0 FAIL / 0 NOT VERIFIABLE recorded in `.scratch/smoke-dec-021.md`.
- Blocked: —
- Parked: —
- The loop is cancelled. Mandate `DEC-021` is `applied`. The owner merges; the run never does.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-021 for SPEC-7 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-021 | DEC-021, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Iter 1 | SPEC-7-01 / DEF-18 | Centralized departure handling into commit_departure() in shortcut_row.slint checking percent_min/max, reverting typed_text and suppressing percent_changed on out-of-bounds | Committing out-of-range typed text into model draft on departure | Out-of-range value enters draft and is refused only later at save | crates/settings/ui/components/shortcut_row.slint |
| Iter 1 | SPEC-7-01 / DEF-18 | Guarded cross-row departures in main.rs against applying out-of-range uncommitted percentages | Unconditional take-and-apply in cross-row event handlers | Out-of-range uncommitted value leaks into draft during navigation | crates/settings/src/main.rs |
