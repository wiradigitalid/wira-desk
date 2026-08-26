---
type: decision
id: DEC-010
status: applied
touches:
  - .how/window-management/04-components/LC-arrangement-engine.md
  - .how/window-management/SDD-window-management.md
  - .what/window-management/02-rules/rules-window-management.md
  - .what/window-management/04-usecases/UC-7-move-window-next-monitor.md
  - .how/window-management/06-flows/flow-monitor-move.md
supersedes: null
superseded_by: null
created: '2026-08-26'
accepted: '2026-08-26'
accepted_by: Product Owner (in session)
applied: '2026-08-26'
---

# DEC-010 — The border clamp resolves its monitor from the planned rect, not from the window

## Decision

**`Win32WindowMover::apply` clamps the frame-inset-compensated rect against the monitor containing
the *planned rect*, not the monitor containing the *window*.**

The resolution changes from `MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL)` to
`MonitorFromRect(&planned, MONITOR_DEFAULTTONULL)`. Everything else about the clamp is unchanged:
still `rcMonitor` rather than `rcWork`, still the target's own monitor rather than the union of all
of them, still degrading to the unclamped rect when the query fails.

For every arrangement command that existed before this wave the two resolutions name the **same
monitor**, because the planned rect is always inside the work area of the window's own monitor.
Snap left, snap right, snap top, snap bottom, maximize, and the overlapping stack are unaffected,
and that sameness is asserted by test rather than argued.

For a monitor move they name **different** monitors, and the window's one is the wrong one.

## Why

`MoveToNextMonitor` is the first command in this product whose planned rect lies on a monitor the
window is not on yet. At the moment `apply` runs, the window is still on the source monitor, so
resolving from the window clamps a rect planned for the *destination* against the bounds of the
*source*. On a two-monitor desktop that does not shave a few pixels off an edge — it collapses the
destination rect into the source monitor's bounds, and the window either does not move or lands
somewhere nobody planned. The feature would ship broken in the ordinary case rather than in an edge
case.

The second reason is why the clamp exists at all, and it is measured rather than reasoned.
`25f52f0` reverted a union-based clamp after Task Manager, snapped on the main monitor, began
jumping hundreds of pixels off-position on a 150% / 175% mixed-DPI pair. The traced cause: the
compensated outer rect bled a few pixels across the shared boundary, and **Windows decided the
window had moved to the neighbouring monitor and forcibly rescaled and repositioned it for that
monitor's DPI.** Letting a compensated rect touch a different-DPI monitor is therefore not a
cosmetic risk; it hands placement authority to Windows.

That finding lands on a monitor move harder than anywhere else, because a monitor move deliberately
targets a monitor whose DPI usually differs. Clamping to the destination monitor's own pixels is
exactly the guard that keeps the compensated rect from touching a third monitor, or bleeding back
across into the source. Resolving from the window would aim that guard at the one monitor the window
is about to leave.

`DEC-007` recorded the cross-DPI cost as "the placement lands a few pixels off at the edge" and
accepted it. That estimate was written before `25f52f0` landed on `main`. It understates the failure
mode: unclamped, the outcome is Windows relocating the window, not a small inset. `DEC-007` is
`applied` and is not edited; this decision records the correction and the mitigation, which is not
the two-pass placement `DEC-007` declined but the clamp that already exists, pointed at the right
monitor.

## Cost

**The two resolutions are indistinguishable on a single-monitor machine**, which is where most
development and all of the current test suite happens. A future refactor can swap `MonitorFromRect`
back to `MonitorFromWindow` and every test still passes, every manual check on one monitor still
looks right, and the monitor move silently breaks. The mitigation is a unit test asserting the two
resolutions agree for a same-monitor plan and disagree for a cross-monitor one — pure geometry, no
Win32 — plus this file explaining why the line reads the way it does.

**`MonitorFromRect` takes a `RECT`, so the planned rect has to be converted before the query**, a
second conversion on a path that already converts once. Trivial in cost, but it is one more place
the coordinate convention has to be right.

**A planned rect that lies on no monitor now degrades to unclamped** where previously the window's
own monitor would have been found. That is the correct degradation — clamping to an unrelated
monitor is worse than not clamping — but it means a nonsensical plan is no longer partially
corrected by accident.

## Alternatives

**Carry the destination monitor on `Placement` and hand it to the mover.** Refused. It widens the
frozen `Placement` contract for the benefit of one command, and it puts a Win32 handle into a struct
the arrangement module deliberately keeps free of them — `arrangement/mod.rs` is pure geometry with
no User32. `MonitorFromRect` gets the same answer from the rect the mover already has.

**Special-case the monitor-move command inside `apply`.** Refused. The mover receives a `Placement`
and nothing else by design; giving it a command discriminant so it can branch would put arrangement
policy in the file whose whole job is applying geometry, and every future command would have to
remember to declare itself.

**Do a two-pass placement: move first, then re-measure and correct.** This is what `DEC-007`
already refused, and its reason still holds — the second pass would have to wait on Windows'
asynchronous DPI-change reflow, which nothing in this codebase waits on anywhere.

**Skip the clamp entirely for a monitor move.** Refused, and it is the most tempting option because
it looks like it removes a complication. `25f52f0` is precisely the measurement that says an
unclamped compensated rect crossing a DPI boundary is the dangerous case, and a monitor move is the
command most likely to produce one.

## Reversal trigger

Revisit if `MonitorFromRect` is observed choosing a monitor a reader would not expect for a rect
spanning two monitors — its documented rule is largest intersection, which is the right rule here,
but it has never been exercised in this codebase.

## Trace

Came from reading `crates/daemon/src/arrangement/win32.rs` at `25f52f0` while implementing W4, not
from a report. `main` had advanced by seven commits during the corpus pass, and `bc0a076`,
`38f7b56`, and `25f52f0` — the border clamp, the union attempt, and its revert — all landed after
`DEC-007` was written. The revert's commit message is the measurement this decision rests on.

Refines the clamp introduced by `bc0a076` and keeps the per-monitor choice `25f52f0` restored.
Corrects a cost estimate in `DEC-007` without editing it. Does not touch `AD-14`: the monitor set is
still enumerated fresh, and `MonitorFromRect` is a query, not a cache.
