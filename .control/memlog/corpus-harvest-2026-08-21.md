---
artifact: .what/_product-brief/brief.md
date: 2026-08-21
---

# Corpus harvest memlog — 2026-08-21

Brownfield harvest G1–G4 executed in one pass (owner pre-approved gates).

| Gate | Skill | Output |
| --- | --- | --- |
| G1 | wdi-problem (manual harvest) | `.what/_product-brief/brief.md` |
| G2 | wdi-product + wdi-ux | `.what/_prd/wira-desk/`, EXPERIENCE, DESIGN |
| G2 tail | wdi-init component | `components.yaml`, SRS/SDD skeletons filled at G3/G4 |
| G3 | wdi-blueprint | SRS kernels, domain, glossary, business rules, `_platform/*` |
| G4 | wdi-component outline | 3 UC flows × 2 PCs, local rules, SDD Decision Summary + Structure |

PC slicing: `window-management` + `settings`. Containers: `daemon`, `settings`. Mode: `outline`.

Sources read: `_bmad-output/planning-artifacts/*`, `spec-wintick/SPEC.md`, architecture spine archive.
