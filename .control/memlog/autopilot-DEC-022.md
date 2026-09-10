---
artifact: .control/decisions/DEC-022-autopilot-mandate-for-spec-8-delivery.md
skill: wdi-autopilot
date: 2026-09-10
---

# Memlog — autopilot run DEC-022

## Resume

- Iteration: 1 (SPEC-8-01 completed)
- Run branch: autopilot/DEC-022, PR not open yet
- Stopped at: —
- Blocked: —
- Parked: —
- Next: SPEC-8-02 (Low-level mouse hook, dual lifecycle, tilt debounce, and worker dispatch)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-022 for SPEC-8 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-022 | DEC-022, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Preflight | Peer review | Configured cursor-agent cli with composer-2.5 model for independent code and doc review | Single-agent self-review | Reviewer independence lost | decisions.yaml, mandate row |
| Iter 1 | SPEC-8-01 | Reindexed Settings shell from 4 to 5 panes, adding dedicated Mouse pane with master toggle and 4 preset selectors | Embedding mouse settings in General or Shortcuts pane | Cluttered taxonomy and broken focus order | crates/settings/src/app.rs, ui/panes/mouse_pane.slint |
| Iter 1 | SPEC-8-01 | Adopted allocation-free DRY slug parser for MouseActionPreset recommended by cursor-agent peer review | Allocating lowercase String on every lookup | Micro-allocations on config validation | crates/shared/src/config.rs |

