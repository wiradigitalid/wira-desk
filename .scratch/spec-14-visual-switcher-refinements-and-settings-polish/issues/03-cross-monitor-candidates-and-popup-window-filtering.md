# 03: Visual switcher cross-monitor candidates, and diagnosis of the helper-window cards

**What to build:**
Two deliverables. Part 1 is now unblocked by `DEC-026`. Part 2 is **not** ready for code and must
not be guessed at.

---

## 1. Cross-monitor candidates for the overlay only — `DEC-026` accepted

`DEC-026` (accepted 2026-09-13) amends the Spatial Preservation Invariant
(`.what/window-management/03-domain/domain-model.md:55`) with a second deliberate crossing: the
visual switcher enumerates across all physical monitors on the current virtual desktop, while the
blind cycle stays monitor-locked. Read `DEC-026` §B-7 and §B-8 before touching the spatial gate —
they settle the two things this ticket would otherwise get wrong.

- `open_visual_switcher` (`worker.rs:235`) collects same-app candidates across **all physical
  monitors**.
- The **virtual-desktop** half (AD-9, `IsWindowOnCurrentVirtualDesktop`) stays. It is not part of the
  decision and is not negotiable — a window on another desktop must still never appear.
- `run_context_safe_cycle` (`worker.rs:578`) is untouched. Blind cycling stays monitor-locked.
- Overlay placement is unchanged: still centred on the origin window's work area
  (`worker.rs:265-283`). Only the candidate set widens.

**This is a scope parameter, not a dropped check** (`DEC-026` §B-7). `evaluate_spatial`
(`context/mod.rs:76`) tests the monitor **first** and returns early, and it rejects on
`origin_monitor == None` with `MonitorUnavailable` *before* the virtual-desktop test is reached.
Deleting the monitor comparison alone therefore either deletes the origin-unavailable rejection for
the switcher, or keeps it and rejects every candidate on a machine where the origin monitor cannot be
resolved. Give `evaluate_spatial` an explicit scope instead:

- `SpatialScope::SameMonitor` — the blind cycle. Behaviour byte-for-byte as today, including both
  `MonitorUnavailable` branches.
- `SpatialScope::AnyMonitorOnCurrentDesktop` — the switcher. The origin-monitor lookup is not
  consulted at all and its failure is not an exclusion; the virtual-desktop test runs unchanged and is
  the sole remaining spatial gate.

The exclusion precedence comment at `eligibility.rs:14-24` is a **different** contract and is not
touched by this ticket. This is the spatial gate, not the eligibility policy.

**The next blind cycle follows the focus** (`DEC-026` §B-8). Activating a card on another monitor
makes that window the foreground, so the next blind cycle is locked to the **new** monitor. That is
Option A behaving correctly and is the single most likely thing to come back as a bug report. Test it
rather than leave it to be re-litigated.

**Retire or retarget the parity guard.**
`switcher::tests::switcher_candidate_set_equals_blind_cycle_eligible_set` (`switcher/mod.rs:182`) is
SPEC-13-01's named guard for the equality this ticket deliberately breaks — and it **will not fail**,
because it calls `collect_eligible_candidates` twice with byte-identical arguments and asserts the two
agree. It is a tautology that would stay green if the production switcher were deleted. Do not leave
it as-is: point it at the two real scopes and assert the intended divergence, naming `DEC-026`. The
fixture it already builds is the right one — `WindowId(3)` on `MONITOR_B`, origin on `MONITOR_A` —
so the retargeted test should assert that `3` is absent under `SameMonitor` and present under
`AnyMonitorOnCurrentDesktop`, while `WindowId(2)` (another virtual desktop) is absent under both.

**Watch the cost.** The candidate set grows with monitor count, and each card registers a live DWM
thumbnail (`switcher/thumbnail.rs`). More cards means more registrations, more pages, and more work
inside the hold window. `every_registration_is_unregistered_on_dismissal` must still pass. Note also
that `Thumbnails::register` returns `Result<isize, i32>` and the behaviour on `Err` is currently
unstated — a single failed registration must degrade that one card, never abort the overlay.

## 2. Helper-window cards ("PopupHC") — diagnose before proposing a fix

**`DEC-026` clause 3 authorises the sanitisation. It does not identify the subject**, and its own
"Owner decisions still open" says so. Root cause remains unestablished, and the first draft's guess is
contradicted by the code. `evaluate_facts` (`cycling/eligibility.rs:39`) already excludes, in a frozen
precedence order: ghost classes, shell surfaces, `!visible`, cloaked, iconic, `tool_window`
(`WS_EX_TOOLWINDOW`), unavailable identity, and different application. Whatever the owner saw passed
all eight — visible, uncloaked, non-minimized, not a tool window, same executable as the foreground
app.

Per `CLAUDE.md`, run `wdi-systematic-debugging` **before** proposing a fix. Capture, for each
offending window: class name, executable, window title, `GetWindowLongW` style and ex-style bits,
client rect, and whether the taskbar lists it. Then decide.

**Constraints on whatever filter is eventually proposed:**

- `DEC-026` clause 3 names three subjects — zero-extent windows, empty titles, and helper/popup host
  surfaces. **None of the three is expressible today.** `WindowFacts` (`cycling/mod.rs:99-113`) holds
  no rect and no title, and no style bits beyond `tool_window`. All three are changes to the
  captured-facts contract, which means `Win32CandidateSource`, the fixtures in `cycling/mod.rs:300-400`,
  and `ReferencePolicy` (`cycling/mod.rs:424`) all move together. Budget for that before quoting the
  clause as though it were a one-line filter.
- A substring match on `"Popup"` is forbidden. Every existing class rule (`SHELL_SURFACE_CLASSES`,
  `CLASS_GHOST`, `cycling/mod.rs:31-41`) is an exact, case-insensitive match against a named class, and
  for good reason — a substring rule silently drops legitimate windows from the switcher, which is a
  worse and far less visible defect than the one being fixed.
- A new rule needs a stated position in the **frozen exclusion precedence** documented at
  `eligibility.rs:14-24`. That order is contract, and combination cases are asserted against it.
- `reproduces_every_frozen_fixture` and `agrees_with_reference_policy_on_every_fixture`
  (`eligibility.rs:79,97`) both have to stay green — the reference policy is a second implementation
  and must be changed in step, not silenced.
- If the eligibility policy changes, it changes for the **blind cycle too** (both call
  `evaluate_facts`). Either that is intended and stated, or the filter belongs in the switcher's own
  collection, not in the shared policy.

Note: after part 1 lands the overlay legitimately shows more windows than before. Confirm the
offending cards are still there before filtering anything.

**Blocked by:** None. Part 1 is ready (`DEC-026` accepted). Part 2 remains gated on
`wdi-systematic-debugging` and may ship in a later spec without holding part 1.

**Status:** part 1 ready-for-agent; part 2 blocked on diagnosis

- [ ] `evaluate_spatial` takes an explicit `SpatialScope`; no caller decides the monitor rule by omitting an argument or by branching on a bare boolean.
- [ ] Under `SpatialScope::SameMonitor` every existing spatial test passes unchanged, **including** both `MonitorUnavailable` rejections.
- [ ] Under `SpatialScope::AnyMonitorOnCurrentDesktop` an unresolvable origin monitor excludes **nothing** — asserted by a test with `origin_monitor: None` and eligible candidates on two monitors.
- [ ] `open_visual_switcher` collects same-app candidates across all physical monitors; windows on secondary monitors appear as cards.
- [ ] Windows on another virtual desktop still never appear in the overlay, under either scope.
- [ ] `run_context_safe_cycle` is unchanged and blind cycling remains locked to the active monitor — asserted by a test that fails if the gate is dropped from the blind path.
- [ ] Activating a cross-monitor card activates it **in place**: its position, size, and monitor are unchanged, and no `SetWindowPos` runs on the commit path (Option A, `DEC-026` clause 1).
- [ ] After a cross-monitor activation, the next blind cycle is locked to the **new** monitor — asserted, not assumed (`DEC-026` §B-8).
- [ ] `switcher_candidate_set_equals_blind_cycle_eligible_set` is retargeted at the two scopes and asserts the intended divergence naming `DEC-026`; no tautological form survives. It was **observed red** against a stubbed-out switcher scope before being accepted as a guard.
- [ ] Every DWM thumbnail registered for a cross-monitor card is unregistered on dismissal.
- [ ] A failed `Thumbnails::register` degrades one card and never aborts the overlay — asserted with a fake source that fails a chosen index.
- [ ] A `wdi-systematic-debugging` run records the class name, executable, style bits, rect, and title of the helper windows, and its conclusion is written into this ticket before any filter is written.
- [ ] Any new exclusion rule matches an exact class name, states its position in the frozen precedence order, and moves `ReferencePolicy` and the fixtures with it.
- [ ] Any rule keyed on rect or title states the `WindowFacts` contract change it requires, and moves `Win32CandidateSource`, the fixtures, and `ReferencePolicy` in the same change.
- [ ] Whether the new rule also applies to the blind cycle is stated explicitly, not left to where the code happens to sit.
