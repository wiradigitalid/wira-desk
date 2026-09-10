---
topic: Wira Desk product brief
artifact: .what/_product-brief/brief.md
skill: wdi-problem (drift fix reported by wdi-reconcile)
updated: 2026-09-10T16:53
---

- (change) Two stale chord mentions corrected: "DPI-Aware Window Snapping" bullet and "Scope In" bullet both still named `Ctrl + Win + Left/Right`/`Ctrl + Win + Arrow/Enter` as the shipped snapping shortcuts, unchanged since `DEC-008` (applied 2026-08-26) moved every shipped arrangement default to the `Ctrl+Alt` family. Corrected to `Ctrl + Alt + Left/Right` and `Ctrl + Alt + Arrow/Enter`. Found by `wdi-reconcile`'s full-product scan (2026-09-06); the promise was unchanged, only the wording was stale, so fixed directly rather than routed through a PRD-level `update`.
- (change) Updated brief with driverless mouse desktop navigation problem, capabilities, and BG-4 goal based on technical research
