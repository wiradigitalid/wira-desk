---
artifact: .control/decisions/DEC-022-autopilot-mandate-for-spec-8-delivery.md
skill: wdi-autopilot
date: 2026-09-10
---

# Memlog — autopilot run DEC-022

## Resume

- Iteration: 1 (preflight accepted)
- Run branch: autopilot/DEC-022, PR not open yet
- Stopped at: —
- Blocked: —
- Parked: —
- Next: SPEC-8-01 (Mouse configuration, preset validation, and Settings Mouse pane)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-022 for SPEC-8 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-022 | DEC-022, decisions.yaml |
| Preflight | Build safety | Pinned single target dir in main checkout to prevent race condition and thrashing | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Preflight | Peer review | Configured cursor-agent cli with composer-2.5 model for independent code and doc review | Single-agent self-review | Reviewer independence lost | decisions.yaml, mandate row |
