---
topic: settings component depth
artifact: .what/settings/SRS-settings.md
skill: wdi-component intents behaviour + design
updated: 2026-08-26T21:23
---

- (change) behaviour: LBR-ST-14 added — one declared sequence is the single source of the Shortcuts pane draw order, its focus order, and the collision precedence order; grouping under headings permitted but must not reorder. UC-4 gained the Win+Ctrl+Left/Right refusal row from DEC-008 and a precondition naming LBR-ST-14. SRS Constraints gained BR-6 and LBR-ST-14 rows.
- (decision) The nine-field list is written into .how/settings/05-model/data-model.md as the declared sequence, rather than into DESIGN.md. Reason: it is a data ordering that governs collision precedence, not a layout; the grouping that reads it belongs to wdi-ux and is explicitly deferred there.
- (change) Field-count references corrected from six to nine in .how/settings/SDD-settings.md Failure Behaviour and in SCN-03's rationale. LBR-ST list reference in the SRS Slots corrected from LBR-ST-1..9 to 1..14 — it was already stale at 13 before this pass.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
