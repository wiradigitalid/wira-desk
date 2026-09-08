---
artifact: .control/decisions/DEC-021-autopilot-mandate-for-spec-7-delivery.md
skill: wdi-autopilot
date: 2026-09-08
---

# Memlog — autopilot run DEC-021

## Resume

- Iteration: 1, commit 05602b6
- Run branch: autopilot/DEC-021, PR not open yet
- Stopped at: Preflight accepted, starting Iteration 1
- Blocked: —
- Parked: —
- Next: SPEC-7-01 (DEF-18 — an out-of-range typed percentage is not reverted at Enter/blur, only at save)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-021 for SPEC-7 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-021 | DEC-021, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
