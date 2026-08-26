---
topic: window-management component depth
artifact: .what/window-management/SRS-window-management.md
skill: wdi-component intents behaviour + design
updated: 2026-08-26T21:23
---

- (change) behaviour: UC-2 widened to all four halves (satisfies FR-14 + FR-22, title now 'Snap the active window to half the screen'); UC-7 written for the monitor move; SCN-03 written for the duplicate-chord condition; LBR-WM-7 (monitor-move semantics) and LBR-WM-8 (deterministic half division) added; four invariants added to domain-model.md.
- (decision) UC-2 widened rather than a new UC written for vertical halves. Reason: a UC title must be a sentence a user would say, and 'snap to half the screen' is one sentence covering four chords; a second near-identical UC would have duplicated every alternate and failure flow.
- (change) design: AD-14 quoted verbatim into Inherited Constraints; AD-2's quote resynchronised with the amended spine; two Failure Behaviour boundaries added (Monitor Enumeration, Duplicate Chord Configuration); B-DisplaySet and E-MonitorSet added to ABCE; UC-7 and SCN-03 walkthroughs added; data-model gained the monitor-set entity and a dictionary; 06-flows/flow-monitor-move.md written.
- (assumption) Every new claim about code carries [MISSING] or [PARTIAL] because none of it is written yet — this is a design, not an as-built record. No claim was raised to verified.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
