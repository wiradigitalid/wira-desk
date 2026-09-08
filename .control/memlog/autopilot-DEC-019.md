---
artifact: .control/decisions/DEC-019-autopilot-mandate-for-spec-5-delivery.md
skill: wdi-autopilot
date: 2026-09-08
---

# Memlog — autopilot run DEC-019

## Resume

Iteration: 1 at 7e8723b
Run branch: autopilot/DEC-019, PR not yet open
Stopped at: in progress
Blocked: —
Parked: —
Next: SPEC-5-02 (DEF-15 — row height toggle parity)

## Decisions

| When | Where | Decided | Instead of | Cost if wrong | Landed in |
|---|---|---|---|---|---|
| Preflight | Mandate acceptance | Accepted DEC-019 for SPEC-5 delivery per owner instruction | Stopping for manual prompt | Supersede DEC-019 | DEC-019, decisions.yaml |
| Preflight | Build & Worktree safety | Single build target at ../wira-desk/target to avoid race condition and duplicate compilation | Per-worktree target dir | Workspace crate fingerprints thrash | decisions.yaml, mandate row |
| Iter 1 | SPEC-5-01 / DEF-14 | Sized tooltip to preferred-height and capped width at 480px with elevated background, preventing 50px stretching and title overlap | Fixed pixel height or inline line | Text clipping or title overlap | crates/settings/ui/components/shortcut_row.slint, shortcut_row_slint_snapshot.rs |

