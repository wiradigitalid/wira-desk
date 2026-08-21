---
artifact: .what/_product-brief/brief.md
skill: orchestrator + opencode
date: 2026-08-21
---

# Memlog — deep corpus rewrite

Owner requested full G1–G4 corpus at `mode: deep`, `risk_accepted: medium` for both PCs.

## Registry state (unchanged values, clarified comments)

- `index.yaml` → `mode: deep` (global default for G4 depth)
- `components.yaml` → both PCs `mode: deep`, `risk_accepted: medium`
- `gates_passed: [G1, G2]` — G3/G4 await owner ratification

## Work performed

OpenCode (`9router/combo`, `--dir` required) rewrote G1 brief+addendum, G2 PRD+UX, G3 platform+SRS kernels, G4 SDDs and most use cases. Orchestrator completed remaining G4 LC stubs, data models, flows, and scenarios after OpenCode stalled on large batches.

## Validation

`py -3 .constitution/method/scripts/validate.py --generate` → GREEN.
