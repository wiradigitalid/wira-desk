---
type: structure
scope: document
verified: 2026-08-21
commit: pending
---

# Document Structure

Written and refreshed by `wdi-init` intent `structure`.

## Verified

2026-08-21 — WDI corpus landed G1–G4 (brownfield harvest).

## Top level

```text
.constitution/
  method/                     # WDI method (overwritten on update)
  project/                    # Product rules, inventory-readers (pending)
.control/                     # Registries, structure maps, memlog
.what/
  _product-brief/             # G1 brief
  _prd/wira-desk/             # G2 PRD + addendum (ID map)
  business-rules.md           # G3 cross-component rules
  window-management/          # PC: cycling, tray, snapping
  settings/                   # PC: config, onboarding, a11y
.how/
  _platform/                  # G3 spine, C4, inventories, design-system
  window-management/          # G4 SDD outline
  settings/                   # G4 SDD + UX design
_bmad-output/                 # BMAD archive (input until retirement)
  prior-knowledge/            # Harvest queue, landing map
design-system/                # UX assets
docs/                         # Engineering decisions, threat model
```
