---
artifact: .control/decisions/DEC-020-autopilot-mandate-for-spec-6-delivery.md
skill: wdi-autopilot
date: 2026-09-08
---

# Memlog — autopilot run DEC-020

## Resume

- Iteration: 1, commit f503dbd
- Run branch: autopilot/DEC-020, PR not open yet
- Stopped at: Preflight accepted, starting Iteration 1
- Blocked: —
- Parked: —
- Next: SPEC-6-01 (DEF-16 — hover tooltip paints behind a later sibling instead of above it)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-020 for SPEC-6 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-020 | DEC-020, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
