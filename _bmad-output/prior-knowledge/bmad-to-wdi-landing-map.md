---
status: Reference
created: 2026-08-21
author: wdi-init setup (orchestrator)
---

# BMAD archive → WDI corpus landing map

This table maps `_bmad-output/` artifacts to their WDI permanent homes. **Landing is not done yet** — each row names the skill that owns the move and whether owner confirmation is required (heavy landing per `corpus-guide.md`).

Legend: **Light** = skill may act then report · **Heavy** = owner confirms first · **Keep** = remain in archive (retirement condition applies)

## G1 — Brief

| Source (BMAD) | WDI target | Skill | Weight |
| --- | --- | --- | --- |
| `brainstorming/.../brainstorm-intent.md` | `.what/_product-brief/brief.md` (distilled) | `wdi-problem` | Heavy — births BG/CAP ids |
| — | `.what/_product-brief/addendum.md` | `wdi-problem` | Heavy |

## G2 — Product

| Source (BMAD) | WDI target | Skill | Weight |
| --- | --- | --- | --- |
| `planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md` | `.what/_prd/wira-desk/prd.md` | `wdi-product` | Heavy — FR/NFR ids |
| `planning-artifacts/prds/.../addendum.md` | `.what/_prd/wira-desk/addendum.md` | `wdi-product` | Heavy |
| `planning-artifacts/ux-designs/.../EXPERIENCE.md` | `.what/wira-desk/04-usecases/EXPERIENCE.md` | `wdi-ux` | Light |
| `planning-artifacts/ux-designs/.../DESIGN.md` | `.how/wira-desk/01-ux/DESIGN.md` | `wdi-ux` | Light |
| `design-system/` | `.how/_platform/design-system.md` + assets path in repo | `wdi-ux` | Light |

## G3 — Blueprint

| Source (BMAD) | WDI target | Skill | Weight |
| --- | --- | --- | --- |
| `planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md` | `.how/_platform/ARCHITECTURE-SPINE.md` | `wdi-blueprint` | Heavy — AD-N ids |
| `specs/spec-wintick/SPEC.md` | `.what/wira-desk/SRS-wira-desk.md` (UC catalogue, constraints) | `wdi-blueprint` | Heavy |
| `specs/spec-wintick/conventions.md` | merge into `.constitution/project/codebase-conventions-guide.md` | `wdi-build` at wave close | Light |
| `planning-artifacts/epics.md` | `.control/registry/waves.yaml` + wave folders | `wdi-build` | Heavy |
| — | `.how/_platform/c4-l2-containers.md` (derive from spine + crates) | `wdi-blueprint` | Light |
| — | `.how/_platform/inventory-screen.md` (settings UI screens) | `wdi-blueprint` | Light |

## G4 — Component

| Source (BMAD) | WDI target | Skill | Weight |
| --- | --- | --- | --- |
| `implementation-artifacts/<story>.md` | `_bmad-output/` until wave closes, then retire | `wdi-build` | Keep until wave close |
| `implementation-artifacts/sprint-status.yaml` | `.control/registry/waves.yaml` | `wdi-build` | Heavy |
| `planning-artifacts/sprint-change-proposal-2026-08-02.md` | `.control/decisions/DEC-NNN-course-correction.md` | `wdi-decision` | Heavy |

## Historical — do not land in corpus

| Source | Disposition |
| --- | --- |
| `planning-artifacts/mom-*.md` | **Keep** in `_bmad-output/`; optional summary → `.control/meetings/` via `wdi-log` |
| `planning-artifacts/implementation-readiness-report-*.md` | **Keep** — audit snapshots |
| `planning-artifacts/*/review-rubric.md` | **Keep** — process artifacts |
| `implementation-artifacts/*-validation-report-*.md` | **Keep** until referenced DEC/wave closed |
| `implementation-artifacts/*-handover-*.md` | **Keep** — inter-session handovers |

## Registry conversions (when landing starts)

| BMAD concept | WDI registry |
| --- | --- |
| CAP-N (SPEC) | `requirements.yaml` capability rows |
| FR-N / NFR-N (PRD) | `requirements.yaml` |
| AD-N (spine) | cited in spine + optional `decisions.yaml` |
| Epic / Story | `waves.yaml` |
| Sprint status | `waves.yaml` status fields |

## Retirement condition for `_bmad-output/`

Per `corpus-guide.md`: a run folder may retire when nothing still cites it by path. After landing G1–G3 artifacts and closing the current brownfield wave, run `wdi-reconcile` to list stragglers before deleting or archiving the BMAD tree.
