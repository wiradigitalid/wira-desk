---
topic: Wira Desk architecture spine
artifact: .how/_platform/ARCHITECTURE-SPINE.md
skill: wdi-blueprint intents catalog + platform
updated: 2026-08-26T21:23
---

- (change) AD-2 command set extended to 6=SnapTop, 7=SnapBottom, 8=MoveToNextMonitor; Binds gained CAP-12; the extend-never-renumber rule and the out-of-set-decodes-to-Nop rule written down explicitly, because both were only asserted in tests.
- (decision) AD-14 added (Monitor Enumeration: Stateless Just-in-Time) rather than widening AD-3. Reason: AD-3 governs Z-order via EnumWindows; the display set is a different API with a different staleness failure (unplug), and folding it in would have been read as covered by implication. Sits behind DEC-007.
- (change) Capability -> Architecture Map gained CAP-12 Monitor Movement; CAP-2's Governed-by gained AD-14.
- (event) Reported, not fixed: inventory-screen.md and AD-11 both name eframe/egui as the Settings UI toolkit, but crates/settings builds ui/*.slint through slint_build and the panes are Slint components. Pre-existing drift, outside this pass's scope.
- (override) Ordering deviation: this artifact was written while DEC-007, DEC-008, and DEC-009 stood at draft. The owner commissioned the whole batch in one instruction. DEC-006's precedent holds documents until the decision is ratified, and that remains the better order; recorded rather than hidden. All three ratified by the Product Owner in session and raised to applied on 2026-08-26.
