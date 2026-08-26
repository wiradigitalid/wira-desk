---
topic: window-management component depth
artifact: .what/window-management/SRS-window-management.md
skill: wdi-component intents behaviour + design
updated: 2026-08-26T22:10
---

- (change) behaviour: UC-2 widened to all four halves (satisfies FR-14 + FR-22, title now 'Snap the active window to half the screen'); UC-7 written for the monitor move; SCN-03 written for the duplicate-chord condition; LBR-WM-7 (monitor-move semantics) and LBR-WM-8 (deterministic half division) added; four invariants added to domain-model.md.
- (decision) UC-2 widened rather than a new UC written for vertical halves. Reason: a UC title must be a sentence a user would say, and 'snap to half the screen' is one sentence covering four chords; a second near-identical UC would have duplicated every alternate and failure flow.
- (change) design: AD-14 quoted verbatim into Inherited Constraints; AD-2's quote resynchronised with the amended spine; two Failure Behaviour boundaries added (Monitor Enumeration, Duplicate Chord Configuration); B-DisplaySet and E-MonitorSet added to ABCE; UC-7 and SCN-03 walkthroughs added; data-model gained the monitor-set entity and a dictionary; 06-flows/flow-monitor-move.md written.
- (assumption) Every new claim about code carries [MISSING] or [PARTIAL] because none of it is written yet — this is a design, not an as-built record. No claim was raised to verified.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
- (event) W4 closed 2026-08-26 at release 0.4.0. Six stories landed, 391 to 438 tests. Two method deviations on the owner's explicit instruction: no isolated worktree (worked on main in the shared checkout) and no dispatched workers or code-review panel (the coordinator wrote the code), so cross-model review independence was not obtained. DEC-010 opened and applied mid-wave from a defect found by reading arrangement/win32.rs: the border clamp resolved its monitor from the window, which would have collapsed every monitor-move placement.
- (event) Story 4's guard was seen failing before being trusted, per the repo rule: breaking it turned exactly five asserting tests red and restoring it turned them green. The intermediate reading that said 'still red after restore' was wrong — Copy-Item preserved the backup timestamp so cargo served the broken build.
