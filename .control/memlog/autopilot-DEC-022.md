---
artifact: .control/decisions/DEC-022-autopilot-mandate-for-spec-8-delivery.md
skill: wdi-autopilot
date: 2026-09-10
---

# Memlog — autopilot run DEC-022

## Resume

**The run is finished.** This block is its end state, not a step that never came.

- `SPEC-8` closed — both tickets (`SPEC-8-01`, `SPEC-8-02`) `done`. Capability `CAP-17` and requirements `FR-30`, `FR-31`, `FR-32` fully delivered.
- Scope: all open tickets and specs completed; promise progress 100% (54/54 counted RTM rows green).
- Run branch: `autopilot/DEC-022`. One PR opened from this branch.
- Suite: **591 passed / 0 failed / 2 ignored**; `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`, `validate.py --generate` all clean.
- Publication hygiene: `scripts/verify-public-export.ps1` passes 10/10 checks.
- Smoke test: automated agent run (`build.ps1 -Mode prod`) completed with 13 PASS / 0 FAIL / 0 NOT VERIFIABLE recorded in `.scratch/smoke-dec-022.md`.
- Blocked: —
- Parked: —
- The loop is cancelled. Mandate `DEC-022` is `applied`. The owner merges; the run never does.

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-022 for SPEC-8 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-022 | DEC-022, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Preflight | Peer review | Configured cursor-agent cli with composer-2.5 model for independent code and doc review | Single-agent self-review | Reviewer independence lost | decisions.yaml, mandate row |
| Iter 1 | SPEC-8-01 | Reindexed Settings shell from 4 to 5 panes, adding dedicated Mouse pane with master toggle and 4 preset selectors | Embedding mouse settings in General or Shortcuts pane | Cluttered taxonomy and broken focus order | crates/settings/src/app.rs, ui/panes/mouse_pane.slint |
| Iter 1 | SPEC-8-01 | Adopted allocation-free DRY slug parser for MouseActionPreset recommended by cursor-agent peer review | Allocating lowercase String on every lookup | Micro-allocations on config validation | crates/shared/src/config.rs |
| Iter 1 | SPEC-8-02 | Co-located WH_MOUSE_LL on existing dedicated Hook Thread with unified lifecycle and heartbeat refresh | Spawning a second hook thread | Unnecessary OS thread overhead and synchronization | crates/daemon/src/hook.rs |
| Iter 1 | SPEC-8-02 | Debounced horizontal tilt wheel at 150ms per SCN-04 and advanced timestamp only on successful enqueue | Advance timestamp on dropped ticks | Retries stalled on saturated ring buffer | crates/daemon/src/hook.rs |
| Iter 1 | SPEC-8-02 | Synthesized virtual desktop navigation chords on Worker actor followed by suppress_start_menu() | Dispatching on Hook Thread | Blocking input callback and racing Windows focus rights | crates/daemon/src/worker.rs |


