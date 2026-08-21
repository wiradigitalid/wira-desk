---
status: Reference
created: 2026-08-21
author: orchestrator (corpus migration plan)
---

# Harvest queue — BMAD archive → WDI corpus

Ordered work list for brownfield Wira Desk. **Nothing in this file is authority** — it schedules reads
from `_bmad-output/` into the corpus through the skill that owns each slot.

Rules (from `corpus-guide.md`):

- A file in `prior-knowledge/` MUST NOT be copied into `.what/` or `.how/` by hand.
- Each row is harvested by invoking the named skill; the skill reads the source path and lands output.
- **Heavy** rows need owner confirmation before the skill writes.
- Exploration output (brainstorm, MoM, readiness reports, review rubrics) stays in archive — never promoted.

## Phase 0 — Owner decisions (before any harvest)

| # | Decision | Where recorded | Status |
|---|---|---|---|
| 0.1 | Brownfield gates already passed: G1, G2, G3 (minimum) | `.control/registry/index.yaml` → `gates_passed` | **done** (2026-08-21) |
| 0.2 | PC slicing: `window-management` + `settings` | `components.yaml` | **done** (2026-08-21) |
| 0.3 | Global `mode: deep`; both PCs `mode: deep`, `risk_accepted: medium` | `index.yaml` + `components.yaml` | **done** (2026-08-21) |
| 0.4 | `risk_accepted` per component | `components.yaml` | **done** (2026-08-21) |

Proposed brownfield `gates_passed` once owner agrees artifacts are ratified:

```yaml
gates_passed: [G1, G2, G3]
```

## Phase 1 — G1 Problem (`wdi-problem`)

| Order | Source (read-only input) | Corpus target | Weight |
|---|---|---|---|
| 1.1 | `_bmad-output/brainstorming/brainstorm-wintick-switcher-2026-07-04/brainstorm-intent.md` | `.what/_product-brief/brief.md` | Heavy |
| 1.2 | distill from PRD problem section if brief gaps | `.what/_product-brief/brief.md` (update) | Heavy |

Skill dispatches `bmad-product-brief` — orchestrator verifies against `brief-guide.md`.

**Gate:** owner holds G1 (20 min checklist in `delivery-flow-guide.md`).

## Phase 2 — G2 Product

### 2a — PRD (`wdi-product` intent `update`)

Brownfield: PRD exists → always `update`, never second PRD folder.

| Order | Source | Corpus target | Weight |
|---|---|---|---|
| 2.1 | `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md` | `.what/_prd/wira-desk/prd.md` | Heavy |
| 2.2 | `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/addendum.md` | `.what/_prd/wira-desk/addendum.md` | Heavy |
| 2.3 | old FR/NFR numbering → new registry ids | `.what/_prd/wira-desk/addendum.md` § ID map | Heavy |

Also seed `.control/registry/requirements.yaml` from landed FR/NFR/CAP.

### 2b — Components (`wdi-init` intent `component`)

After PRD lands — birth registry row + SRS/SDD skeletons:

- `product_components`: `[wira-desk]`
- `.what/wira-desk/SRS-wira-desk.md` (skeleton)
- `.how/wira-desk/SDD-wira-desk.md` (skeleton)

### 2c — UX (`wdi-ux`) — optional at G2, needs PC from 2b

| Order | Source | Corpus target | Weight |
|---|---|---|---|
| 2.4 | `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/EXPERIENCE.md` | `.what/wira-desk/04-usecases/EXPERIENCE.md` | Light |
| 2.5 | `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/DESIGN.md` | `.how/wira-desk/01-ux/DESIGN.md` | Light |
| 2.6 | `design-system/` assets | `.how/_platform/design-system.md` + asset paths | Light |

**Gate:** owner holds G2 (45 min).

## Phase 3 — G3 Blueprint (`wdi-blueprint`)

Run **`catalog` before `platform`**.

### 3a — Intent `catalog`

| Order | Source | Corpus target | Weight |
|---|---|---|---|
| 3.1 | PRD + `_bmad-output/specs/spec-wintick/SPEC.md` (CAP → UC) | `.what/wira-desk/SRS-wira-desk.md` § UC Catalogue, § Actor Register | Heavy |
| 3.2 | same sources | `.what/wira-desk/03-domain/domain-model.md` | Heavy |
| 3.3 | cross-cutting rules from SPEC constraints + PRD | `.what/business-rules.md` | Light |
| 3.4 | terms from PRD/SPEC/UX | `.control/product-glossary.md` | Light |

### 3b — Intent `platform`

| Order | Source | Corpus target | Weight |
|---|---|---|---|
| 3.5 | `_bmad-output/planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md` | `.how/_platform/ARCHITECTURE-SPINE.md` | Heavy |
| 3.6 | spine + `crates/` layout | `.how/_platform/c4-l2-containers.md`, L3 as needed | Light |
| 3.7 | settings UI from UX | `.how/_platform/inventory-screen.md` | Light |
| 3.8 | config/TOML from code + SPEC | `.how/_platform/inventory-db.md` (minimal at catalog) | Light |
| 3.9 | IPC/constants from `crates/shared` | `.how/_platform/cross-cutting.md` | Light |

**Gate:** owner reads `.control/generated/blueprint.md` and holds G3 (45 min).

## Phase 4 — G4 Component

At **`mode: catalog` → skipped**. No `wdi-component` depth unless owner raises `mode`.

Implementation story files in `_bmad-output/implementation-artifacts/` remain archive until a wave closes.

## Phase 5 — G5 Release / waves (`wdi-build`)

| Order | Source | Corpus target | Weight |
|---|---|---|---|
| 5.1 | `_bmad-output/planning-artifacts/epics.md` + `sprint-status.yaml` | `.control/registry/waves.yaml` | Heavy |
| 5.2 | `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-02.md` | `.control/decisions/DEC-NNN-*.md` via `wdi-decision` | Heavy |
| 5.3 | `_bmad-output/specs/spec-wintick/conventions.md` | merge → `.constitution/project/codebase-conventions-guide.md` | Light (wave close) |

Future coding waves: `wdi-build` dispatches **`bmad-spec`** — new `SPEC.md` is a **projection** of `.what/` + `.how/`, not a copy of legacy `spec-wintick/SPEC.md`.

## Phase 6 — Infrastructure (parallel anytime after G2)

| Skill | Intent | Output |
|---|---|---|
| `wdi-init` | `readers` | `.constitution/project/inventory-readers.py` |
| `wdi-init` | `structure` | refresh structure maps after first landing |
| `wdi-reconcile` | — | drift report after each phase |

OpenCode role: implement `inventory-readers.py`, run `validate.py --generate`, bulk registry YAML scaffolding — orchestrator dispatches and verifies.

## Archive — never harvest (keep in `_bmad-output/`)

- `planning-artifacts/mom-*.md`
- `planning-artifacts/implementation-readiness-report-*.md`
- `**/review-rubric.md`, `**/validation-report.md`
- `implementation-artifacts/*-handover-*.md` (until wave closed)
- `brainstorm-intent.md` after brief lands (provenance only)

## Retiring `_bmad-output/` (all three MUST hold)

1. Every promise mapped — ID table lives in `.what/_prd/wira-desk/addendum.md`, not here.
2. Every live citation re-pointed to corpus or code.
3. `DEC-` records the retirement.

Then run `wdi-reconcile` for stragglers.

## Suggested session order (one harvest per owner session)

1. Owner confirms 0.1 + 0.2 → orchestrator runs Phase 1 → gate G1
2. Phase 2a–2b → gate G2
3. Phase 2c (UX) if wanted same session
4. Phase 3a → 3b → gate G3
5. Phase 5.1–5.2 (waves + course correction DEC)
6. Phase 6 readers + reconcile
7. Begin coding wave via `wdi-build` (OpenCode for Rust)
