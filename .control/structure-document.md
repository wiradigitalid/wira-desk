---
type: structure
scope: document
verified: 2026-08-21
commit: pending
---

# Document Structure

Written and refreshed by `wdi-init` intent `structure`. Rules live in
`.constitution/method/structure-guide.md`.

## Verified

2026-08-21 — brownfield scaffold after WDI Method 0.5.10 install. Corpus landing not started.

## Top level

```text
.constitution/
  method/                     # WDI method (overwritten on update)
  project/                    # Product rules (constitution, codebase guides)
.control/                     # Registries, structure maps, questions, decisions
.what/                        # Promises (empty — pending landing from _bmad-output)
.how/                         # Blueprint (empty — pending landing)
  _platform/                  # Spine, C4, inventories (skeleton only)
_bmad-output/                 # BMAD archive + prior-knowledge migration notes
  prior-knowledge/            # wdi-init inventory and landing map
  planning-artifacts/         # PRD, architecture, epics, MoM, readiness reports
  implementation-artifacts/   # Story specs, sprint-status, validation reports
  specs/                      # SPEC kernel
  brainstorming/              # Original intent
design-system/                # UX assets (partial redaction for public)
docs/                         # Product engineering docs (decisions, threat-model)
```

## Planned corpus (after landing)

```text
.what/
  _product-brief/
  _prd/wira-desk/
  wira-desk/                  # SRS, use cases, domain (PC: wira-desk)
.how/
  _platform/                  # ARCHITECTURE-SPINE, C4, inventories, design-system
  wira-desk/                  # SDD, UX design, integrations
```
