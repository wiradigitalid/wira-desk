---
topic: Wira Desk UX spines
artifact: .how/settings/01-ux/DESIGN.md
skill: wdi-ux -> bmad-ux intent update
updated: 2026-08-26T21:23
---

- (decision) Every editable chord stays in one pane (Shortcuts), grouped into three labelled cards (Switching 2, Snap & resize 5, Move & arrange 2). Reason: the daemon's capture lease is armed from which pane is showing (DEC-004), so shortcut fields in two panes means two panes arming the observe lease - a regression in the DEC-005 key check. A fourth pane was refused for the same reason.
- (change) Corrected a drift both spines carried: DESIGN.md and EXPERIENCE.md both described snapping chords under a 'Layout & Snapping' pane and the switcher chords under 'Shortcuts'. The shipped build has always drawn all chord rows in Shortcuts. The build is right; both documents were wrong. Pane 3 is now named 'Layout' and holds no chord.
- (decision) Group order and within-group row order are the declared sequence from LBR-ST-14, not the enum numbering: inside Snap & resize the rows follow the arrow keys (left, right, top, bottom) with Maximize last. The groups mirror the three config sections on disk, so what the user sees and what the product stores stop disagreeing.
- (change) Added a six-state table for a shortcut row (Resting, Listening, Refused-grammar, Refused-reserved, Collides, Empty) and Flow 6 for UJ-4. Recorded that the 'unbound action' state from DEC-009 has no representation in this pane and is deliberately not designed in this pass.
- (assumption) No new domain noun was introduced by this pass; 'work area', 'next monitor', 'proportional placement', and 'unbound action' all entered .control/product-glossary.md through wdi-blueprint earlier in the same session.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
