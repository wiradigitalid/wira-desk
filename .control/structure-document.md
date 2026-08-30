---
type: structure
scope: document
verified: '2026-08-30'
commit: '50d616d'
---

# Document Structure

Written and refreshed by `wdi-init` intent `structure`. Rules live in
`.constitution/method/structure-guide.md`.

## Verified

2026-08-30 — read from the tree on disk at commit `50d616d`.

## Top level

```text
wira-desk/
├── .constitution/              # WDI Method rules — see below
├── .control/                   # Registries, decisions, questions, structure maps, memlog — see below
├── .what/                      # Promises corpus — see Product Components table below
├── .how/                       # Mechanism corpus — see Product Components table below
├── _bmad-output/                # BMad skill archive: specs, planning artifacts, prior-knowledge
├── docs/                         # Engineering rationale, threat model, packaging choice (predates the method)
└── .work/                         # Scratch, emptied when a task closes
```

## Product Components

Both registered in `.control/registry/components.yaml`; both have content on both sides of the
placement test.

| PC | `.what/<pc>/` | `.how/<pc>/` |
| --- | --- | --- |
| `window-management` | `SRS-window-management.md`, `02-rules/`, `03-domain/` (domain-model, state-machines), `04-usecases/` (UC-1, UC-2, UC-3, UC-7), `05-scenarios/` (SCN-01..03) | `SDD-window-management.md`, `02-contracts/`, `04-components/`, `05-model/`, `06-flows/` — no `01-ux/` (no screens of its own) |
| `settings` | `SRS-settings.md`, `02-rules/`, `03-domain/` (domain-model, state-machines), `04-usecases/` (EXPERIENCE, UC-4, UC-5, UC-6), `05-scenarios/` (SCN-01..03) | `SDD-settings.md`, `01-ux/` (DESIGN.md), `02-contracts/`, `04-components/`, `05-model/`, `06-flows/` |

## `.constitution/`

```text
.constitution/
├── method/                    # WDI Method package — overwritten wholesale by `wdi-method update`
│   ├── document/                 # guides, templates, the BMad skill register
│   ├── scripts/                    # validate.py, inventory.py, memlog.py (via _bmad/scripts)
│   └── why/                          # status: Reference — explains, never binds
└── project/                    # This product's own room — update never overwrites it
    ├── codebase-stack-guide.md, codebase-conventions-guide.md, codebase-brownfield-guide.md   # all status: Draft
    └── inventory-readers.py       # not yet written
```

## `.control/`

```text
.control/
├── registry/                  # components.yaml, index.yaml, waves.yaml, risks.yaml, defects.yaml
├── decisions/                  # DEC-001 .. DEC-010, all status: applied
├── questions/                   # blocking.md, assumptions.md, external.md, answered.md
├── memlog/                       # one .md per Product Component, plus spine.md
├── meetings/                      # minutes
├── generated/                      # validate.py --generate output — never hand-written
├── product-glossary.md
├── project-non-technical-log.md
├── structure-codebase.md
└── structure-document.md
```

## `.what/` and `.how/` — product level

```text
.what/
├── _product-brief/             # brief.md — G1
├── _prd/wira-desk/               # prd.md — G2
└── business-rules.md              # G3 cross-component rules

.how/
└── _platform/                  # ARCHITECTURE-SPINE.md, c4-l1/l2/l3, cross-cutting.md,
                                  # design-system.md, the three inventories (db/api/screen)
```

## `_bmad-output/`

```text
_bmad-output/
├── brainstorming/              # exploration output — homeless by rule, never promoted
├── planning-artifacts/          # architecture/, prds/, ux-designs/ — pre-method-era archive
├── prior-knowledge/               # harvest-queue.md — the brownfield-harvest landing map
├── implementation-artifacts/        # per-story validation reports
└── specs/                             # one folder per wave: SPEC.md (+ stories/ for w1 and w4 only —
                                        # spec-first-run-tutorial and spec-settings-window have none;
                                        # reported by wdi-reconcile as V18 drift, not fixed here)
```

---

★ = key file: entry point, wiring root, the single place a rule is enforced, or a file that must be
opened before behaviour in its folder can be changed.
