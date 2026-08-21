---
status: Reference
created: 2026-08-21
author: wdi-init setup (orchestrator)
---

# Existing documents inventory — Wira Desk

Read-only report from `wdi-init` intent `setup`. No file was moved in this step.

## Method state after install

| Item | Value |
| --- | --- |
| WDI Method | 0.5.10 (installed 2026-08-21) |
| BMAD Method | 6.11.0 (fresh install; was absent in public repo) |
| BMAD Loop | 0.11.0 |
| CIS | 0.3.1 |
| Global `mode` | `catalog` (default) |
| Gates passed | none recorded — owner must confirm brownfield gates |

## Repository layers (current)

| Path | Role | Notes |
| --- | --- | --- |
| `crates/{daemon,settings,shared}/` | Application code | Pre-release 0.1.0, builds and tests pass |
| `docs/` | Product engineering docs | `decisions.md`, `threat-model.md` |
| `_bmad-output/` | BMAD planning archive | 57+ artifacts from WinTick era; primary migration source |
| `design-system/` | UX corpus | Partially redacted for public |
| `.constitution/` | Governance | `method/` (WDI) + `project/` (product rules) |
| `.control/` | Registries | Scaffolded empty; see landing map |
| `.what/` / `.how/` | WDI corpus | Empty — landing not yet done |
| `.work/` | Ephemeral agent workspace | Empty |

## Documents already present (by WDI gate)

### G1 — Problem / brief

| Path | Looks like |
| --- | --- |
| `_bmad-output/brainstorming/brainstorm-wintick-switcher-2026-07-04/brainstorm-intent.md` | Original ideation intent; commercial/marketing redacted |

### G2 — Product

| Path | Looks like |
| --- | --- |
| `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md` | Full PRD (15 FR, 7 NFR); some commercial strategy redacted |
| `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/addendum.md` | PRD addendum |
| `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/review-rubric.md` | PRD review rubric |
| `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/DESIGN.md` | UX design spec |
| `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/EXPERIENCE.md` | User journey |
| `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/review-rubric.md` | UX review rubric |
| `_bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/validation-report.md` | UX validation |

### G3 — Blueprint

| Path | Looks like |
| --- | --- |
| `_bmad-output/specs/spec-wintick/SPEC.md` | Technical spec kernel (11 CAP, 11 constraints) |
| `_bmad-output/specs/spec-wintick/conventions.md` | Spec conventions companion |
| `_bmad-output/planning-artifacts/architecture/architecture-WinTick-2026-07-06/ARCHITECTURE-SPINE.md` | Architecture spine (AD-1…AD-12) |
| `_bmad-output/planning-artifacts/epics.md` | Epic and story breakdown |
| `design-system/` | Tokens, UI kits, brand assets (partial redaction) |

### G4 — Component / implementation

| Path | Looks like |
| --- | --- |
| `_bmad-output/implementation-artifacts/*.md` | Per-story specs, validation reports, handovers (~30 files) |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | Sprint ledger |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Deferred items |

### Process / historical (stay in archive or `.control/`)

| Path | Looks like |
| --- | --- |
| `_bmad-output/planning-artifacts/mom-*.md` | Six meeting minutes (2026-07-04 … 2026-07-09) |
| `_bmad-output/planning-artifacts/implementation-readiness-report-*.md` | Four readiness assessments |
| `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-02.md` | Course correction proposal |
| `docs/decisions.md` | Engineering decisions (already product-facing) |

## Proposed Product Components (for owner confirmation)

Single-container desktop suite — propose **one PC** initially:

| PC | Owns | Maps from |
| --- | --- | --- |
| `wira-desk` | Window cycling, arrangement, settings UI, daemon lifecycle | SPEC CAP-1…11, FR-1…21, crates workspace |

Alternative (if splitting later): `daemon` + `settings` as two PCs sharing `shared` as platform — defer until `wdi-init` intent `component` with owner approval.

## Next steps (not done in setup)

1. Owner confirms `gates_passed` for brownfield (G1–G3 at minimum).
2. `wdi-init` intent `component` — register `wira-desk` PC.
3. `wdi-blueprint` — land spine, inventories, SRS skeleton from `_bmad-output` sources.
4. `wdi-init` intent `readers` — write Rust inventory readers for validate.py.
5. `wdi-init` intent `structure` — refresh structure maps after landing begins.

See `bmad-to-wdi-landing-map.md` in this folder for the file-by-file landing table.
