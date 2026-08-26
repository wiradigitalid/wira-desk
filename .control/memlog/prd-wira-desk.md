---
topic: Wira Desk PRD
artifact: .what/_prd/wira-desk/prd.md
skill: wdi-product intent update -> bmad-prd
updated: 2026-08-26T21:23
---

- (decision) Intent update on the wira-desk PRD: CAP-12 born (move the active window to another monitor), FR-22 (top/bottom half snapping) under CAP-2, FR-23 (next monitor, proportional, same virtual desktop) under CAP-12, UJ-4 added as the journey both realize. Reason: owner request 2026-08-26.
- (change) FR-14 and FR-15 chord literals moved from the Ctrl+Win family to Ctrl+Alt, and FR-15's default to ctrl+alt+shift+down, per DEC-008. FR-14 gained a consequence stating Win+Ctrl+Left/Right is now unconfigurable.
- (change) Section 8.2 Out-of-Scope line 'Multi-monitor window repositioning shortcuts (delegated to native Windows Win+Shift+Arrows)' retired and replaced by the narrower exclusion of NAMED-monitor movement, naming DEC-007. Section 8.1 In Scope updated to the Ctrl+Alt family and the monitor move.
- (override) _bmad/custom/bmad-prd.toml still carried run_folder_pattern = FILL-initiative-slug and prd-FILL-initiative-slug.md; both replaced with the wira-desk slug so this memlog lands in the file the toml already mandates. Repo defect, not a customisation choice.
- (assumption) OQ-20 (Ctrl+Alt+Arrow vs graphics-driver screen rotation) and OQ-21 (EnumDisplayMonitors order matches physical arrangement) filed to .control/questions/assumptions.md; neither passes a blocking test.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
