---
type: decision
id: DEC-007
status: applied
touches:
  - .what/_prd/wira-desk/prd.md
  - .control/registry/requirements.yaml
  - .control/registry/usecases.yaml
  - .control/product-glossary.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .what/window-management/02-rules/rules-window-management.md
  - .what/window-management/03-domain/domain-model.md
  - .what/window-management/04-usecases/UC-7-move-window-next-monitor.md
  - .what/window-management/SRS-window-management.md
  - .how/window-management/SDD-window-management.md
  - .how/window-management/04-components/LC-arrangement-engine.md
  - .how/window-management/05-model/data-model.md
  - .how/window-management/06-flows/flow-monitor-move.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/settings/01-ux/DESIGN.md
supersedes: null
superseded_by: null
created: '2026-08-26'
accepted: '2026-08-26'
accepted_by: Product Owner (in session)
applied: '2026-08-26'
---

# DEC-007 — Moving the active window between monitors is Wira Desk's own command, no longer delegated to Windows

## Decision

**Wira Desk owns a command that moves the active window to the next physical monitor, and the
delegation to Windows' native `Win + Shift + Arrow` is withdrawn.**

The command is `Command::MoveToNextMonitor`, wire value `8`. It resolves the foreground window's
monitor, picks the next monitor in a fixed enumeration order, and places the window on that monitor
at the **same proportion of the work area** it occupied on the one it left. The virtual desktop is
not part of the operation: `SetWindowPos` does not move a window between virtual desktops, so the
window stays on the desktop it was on, and `AD-9` is untouched.

Three properties are fixed here, because each of them has a wrong answer that looks reasonable:

| Property | Fixed as | Not |
|---|---|---|
| Monitor order | The order `EnumDisplayMonitors` reports, sampled fresh per invocation, wrapping from last to first | Left-to-right by coordinate, which is undefined for stacked monitors |
| Geometry | Proportional to the destination work area | The source rect's absolute width and height |
| Single monitor | A successful no-op. Nothing moves, nothing is reported to the user | An error, a toast, or a wrap onto the same monitor |

**The frozen contract that forbade this is withdrawn by name.** `arrangement/mod.rs` carries a test,
`command_set_contains_no_inter_monitor_arrangement`, which sweeps `0u8..=5` and asserts every value
is accounted for, with the delegation written into it as a comment. That test is replaced by one that
sweeps the new range and asserts the same completeness property over it. The wire values `0..=5` keep
their meanings; the range is **extended**, never renumbered.

## Why

The delegation was reasonable when it was written and stopped being reasonable for one measurable
reason: `Win + Shift + Arrow` moves a window to the adjacent monitor **and** re-maximizes or
re-snaps it according to Windows' own idea of its state, discarding the arrangement Wira Desk just
applied. A user who snaps a window to the left half and then moves it to the second monitor does not
get a left half on the second monitor. The two features do not compose, which is the thing a
delegation is supposed to buy.

The second reason is spatial, and it is the product's own core promise. `CAP-7` and `FR-2` lock
cycling to the monitor the user is looking at, deliberately, so the peripheral workspace is never
disturbed. That promise makes the monitor a first-class boundary in this product — and a product that
treats the monitor as a boundary it enforces, while having no way to cross it on purpose, is
incomplete rather than minimal. The owner asked for the crossing to be deliberate and keyboard-driven,
which is the same shape as everything else the product does.

Proportional placement rather than absolute is not a refinement, it is the whole point of doing this
ourselves. The `arrangement` contract already fixes coordinates as physical pixels and forbids
planners from scaling by DPI; a monitor move is the first operation in this product that spans two
work areas of possibly different size and DPI, so it is the first place where "copy the rect" and
"keep the arrangement" diverge. Copying a half-screen rect from a 1920-wide monitor onto a 3840-wide
one produces a quarter, and the user reads that as the feature being broken.

## Cost

**A frozen contract is being withdrawn, and that is the real price.** The test that forbade this was
not an oversight; it was a decision written as executable text so it could not be eroded quietly.
Withdrawing it costs the guarantee that the command set is closed, and every future reader of
`commands.rs` now has one more precedent for extending it. The mitigation is that the replacement test
asserts the same completeness property over the new range, so the set stays closed at its new size —
but the precedent is real and this decision does not pretend otherwise.

**Monitor enumeration is new Win32 surface in a crate that had none.** Today
`daemon/context/spatial.rs` reaches for exactly one call, `MonitorFromWindow`, and never asks what
other monitors exist. Adding `EnumDisplayMonitors` plus a `GetMonitorInfoW` per monitor brings a
callback-based API, an `unsafe` block per call site, and a `SAFETY:` note for each — in a workspace
where `undocumented_unsafe_blocks` is `deny`. It also brings the failure mode enumeration always
brings: a monitor that disappears between enumeration and placement.

**Frame-inset compensation is measured on the wrong monitor.** `frame_insets(hwnd)` measures the gap
between `GetWindowRect` and the extended frame bounds before `SetWindowPos`, at the source monitor's
DPI. After a move to a monitor with different scaling, that measurement no longer describes the
window. The placement lands a few pixels off at the edge, and this decision **accepts that** rather
than specifying a two-pass placement — the second pass would have to re-measure after Windows has
finished its own DPI-change reflow, which is asynchronous and not something this codebase currently
waits on anywhere. Recorded as a known limitation, not as a defect to be filed later.

**`Win + Shift + Arrow` still exists and now does something different from our command.** Two chords
move a window between monitors and they disagree about what happens to its arrangement. The product
does not, and will not, reserve or suppress the Windows one.

## Alternatives

**Keep the delegation and document the composition failure.** Refused. The composition failure is the
whole complaint; documenting it converts a missing feature into a known bug and buys nothing.

**Move to a *named* monitor (primary, secondary) rather than the next one.** Refused for now on
cost, not merit: it needs a stable monitor identity that survives unplug, sleep, and driver updates,
and Windows does not offer one cheaply — `HMONITOR` is a handle, not an identity, and device paths
change. "Next, wrapping" needs no identity at all and is correct on any number of monitors. Named
here so the reversal trigger has something to point at.

**Direction-based movement (`next monitor to the left`).** Refused. It reintroduces exactly the
coordinate ordering this decision refuses: for monitors stacked vertically, or arranged in an L, "to
the left" is either undefined or surprising, and the surprise happens on the user's own desk where we
cannot reproduce it.

**Preserve the window's absolute size and only translate its origin.** Refused. It is simpler and it
is wrong at any DPI or resolution difference, which is the common case rather than the edge case —
mixed-DPI is the normal state of a laptop with an external display.

**Put the planner in `arrangement/snap.rs`.** Refused on boundary grounds. Every function in that file
takes exactly one `WorkArea` and is pure geometry over it; this operation needs two work areas and a
monitor list. Adding it there would make the file's one clear invariant untrue. It gets its own module.

## Reversal trigger

Revisit the enumeration order if a user reports that "next monitor" cycles in an order that does not
match how their displays are physically arranged — at that point the cheap ordering has failed its
only job, and named monitors become worth their cost.

Revisit the accepted frame-inset imprecision if the mis-landing is reported by anyone other than its
author, or if it exceeds a few pixels on any real configuration. The fix is known; only its price is
being declined.

## Trace

Came from an owner request on 2026-08-26, asked together with vertical snapping and a chord-family
change, and from the codebase reading that followed it. The delegation being withdrawn is stated in
two places and both were read rather than recalled: the test comment in
`crates/daemon/src/arrangement/mod.rs` and PRD §8.2's "Out of Scope for MVP" line.

Extends `AD-2`'s `u8` command channel by three values without renumbering any. Sits beside `CAP-7`
and `FR-2`, which make the monitor a boundary this product enforces, and is the first command that
crosses that boundary deliberately. Does not touch `AD-9` — virtual desktop membership is not changed
by `SetWindowPos`, which was verified rather than assumed.

Opens no question. The two things that cannot be settled from a desk — enumeration order against a
real multi-monitor arrangement, and the cross-DPI landing error — are named as reversal triggers
rather than filed as questions, because both need a user's desk and neither blocks the work.
