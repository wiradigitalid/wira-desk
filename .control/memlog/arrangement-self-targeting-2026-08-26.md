---
artifact: .control/decisions/DEC-006-arrangement-never-targets-a-wira-desk-window.md
skill: orchestrator (no wrapper invoked)
date: 2026-08-26
---

# Memlog — arrangement self-targeting, DEC-006 drafted and applied

Owner reported on 2026-08-26 that snapping or stacking while the Settings window held the foreground
left a large invisible region over the desktop that swallowed mouse clicks, and asked for the
behavioural boundary to be decided before any code was written. A three-question consultation prompt
came with the report, proposing a daemon-side exclusion, a `WM_GETMINMAXINFO` clamp, and a choice
between a no-op and retargeting.

`wdi-systematic-debugging` and `wdi-decision` were named to the owner and **not invoked** — the project
rule forbids invoking a skill automatically. The `DEC-` file, the registry row, the two open questions,
and this memlog were written directly. If the owner wants the wrappers' own numbering and verification
pass, the files can be re-landed through them.

## What was verified before deciding

Read against the code and against the installed dependency sources, because the proposal's central
mechanism turned out to be one that is already in place and already failing:

| Claim checked | Finding |
|---|---|
| `resolve_context()` targets the foreground window with no owner filter | **True.** `arrangement/win32.rs` guards only `hwnd != 0`, `IsWindow`, cloaked, monitor, and DPI. No executable, class, or process-id check exists anywhere on that path |
| The Settings window is fixed-size, frameless, and transparent | **True.** `main_window.slint` sets `no-frame`, `background: Colors.transparent`, and binds `width`/`height` to 760×610 (580×380 while onboarding) |
| Something in the toolkit stack resists an external resize | **False, and this is the finding that changed the recommendation.** Slint sets the window non-resizable and installs min/max inner size; winit turns that into `ptMinTrackSize`/`ptMaxTrackSize` in `WM_GETMINMAXINFO`; and winit's `WM_WINDOWPOSCHANGING` handler only tracks the monitor of a fullscreen window and never touches `cx`/`cy` |
| `ptMaxTrackSize` would stop `SetWindowPos` | **False.** It governs user drag-resize tracking and maximize. The proof is local: it is *already* written for this window, and the ghost frame still appears |
| The window class is `"SlintWindow"` | **False.** The toolkit registers a generic class. The existing single-instance check reaches for the window *title* instead, which is a worse discriminator still — any window titled "Wira Desk" matches |
| `arrangement/win32.rs` is one of several resize paths | **False, and useful.** It is the only `SetWindowPos` in the whole workspace, which is what makes a single gate in `resolve_context_for` cover all four commands |
| The exclusion belongs beside the VM/RDP bypass in the hook | **No.** The bypass's disposition is *passthrough*, and passing `Ctrl+Win+Left` back to Windows fires its previous-virtual-desktop action — `reservation()` deliberately does not reserve `Ctrl+Win+Arrow`. This gate's disposition has to be a swallow, which the hook already performs for free before the Worker runs |

## Where the recommendation departed from the proposal

Three departures, each forced by one of the findings above rather than by preference:

- **The clamp moved from `WM_GETMINMAXINFO` to `WM_WINDOWPOSCHANGING`.** The proposed message is not
  merely weaker; it is the one already installed and already not working.
- **The exclusion moved from the hook to the Worker.** Same conclusion, different seam: the Worker
  already pays the process-identity cost on the stack path, the hook callback is budgeted under 10 ms,
  and the hook's neighbouring bypass carries the opposite disposition.
- **The discriminator moved from window class to executable basename.** `AD-4` already ratified
  executable-name identity for cycling exclusion, so this extends a mechanism rather than adding one.
  A process id registered by Settings was refused outright on `OQ-17`'s existing finding about id reuse.

The retarget option was refused on two independent grounds — it would need an `EnumWindows` sweep and
an eligibility policy inside a module deliberately decoupled from both, and it moves a window the user
cannot see behind the one they are looking at, which is the unverifiable-outcome failure `DEC-005`
refuses under a different name.

## Status at the end of the drafting pass

Superseded by the section below, and kept because it records what was withheld and why. At the close of the
drafting pass `DEC-006` was `status: draft` with `touches: []`, and **nothing in `.what/` or `.how/` was touched.**
The same two rules that held for `DEC-003` through `DEC-005` hold here: an agent MUST NOT accept its
own `DEC-`, and `touches` is filled at apply from what actually changed. Carrying a draft into the
documents it governs would forge the evidence `applied` exists to record.

So the edits that were drafted and then withheld, for the apply step after the owner ratifies:

| Document | What apply owes it |
|---|---|
| `.what/window-management/02-rules/rules-window-management.md` | `LBR-WM-6` — arrangement target eligibility, bound to `LC-arrangement-engine` |
| `.what/window-management/SRS-window-management.md` | One Constraints bullet, alongside the existing VM-bypass bullet |
| `.what/window-management/04-usecases/UC-2-snap-window-half.md` | One Alternate Flow row, and `LBR-WM-6` in Business Rules |
| `.how/window-management/04-components/LC-arrangement-engine.md` | The eligibility gate in Responsibility, and a Notes entry |
| `.what/settings/02-rules/rules-settings.md` | `LBR-ST-10` — the painted-size invariant, worded toolkit-neutrally |
| `.how/settings/04-components/LC-settings-shell.md` | A Notes entry for the message-boundary clamp |

No new scenario is planned. `LBR-WM-6`, the UC-2 alternate flow, and this decision already carry the
behaviour between them, and a fourth restatement is the duplication `OQ-13` is already open about.

`OQ-18` and `OQ-19` were added for the two things that cannot be settled from a desk: whether the
`WM_WINDOWPOSCHANGING` clamp actually holds against an `SWP_ASYNCWINDOWPOS` request, and whether any
third-party window manager really resizes a non-resizable frameless window — which is the entire
justification for paying for the clamp on top of the exclusion.

## Ratified and applied, same day

The owner ratified `DEC-006` in session on 2026-08-26 and instructed the apply, so the status moved
`draft` → `applied` in one step rather than resting at `accepted`. Recorded here because the
intermediate state has no artefact of its own: the ratification was spoken, not written, and this
paragraph is the only place it exists.

All six documents the table above reserved were written, and `touches:` was filled from what actually
changed rather than from that table — they agree, and the agreement was checked rather than assumed.
Two identifiers were added: `LBR-WM-6` (arrangement target eligibility) and `LBR-ST-13` (the painted-size
invariant). `LBR-ST-13` and not `LBR-ST-10`, which the draft's reserved-edits table had guessed —
`DEC-003` through `DEC-005` had already consumed 10 through 12 by the time this applied, which is
exactly why that table named documents rather than numbers.

Two things the apply deliberately did **not** widen into, per the rule that applying must not improvise:

- No new scenario file. `LBR-WM-6`, the `UC-2` alternate-flow row, and the decision carry the
  behaviour between them, and `OQ-13` is already open about restating one rule in three places.
- The stale egui/eframe description in the spine's Settings-binary entry and in `LC-settings-shell`.
  The `LC-settings-shell` note added here is worded toolkit-neutrally so it does not deepen that drift,
  but the drift itself stays for `wdi-reconcile`.

## Drift found and left alone

`AD-11` still reads "Settings Binary: egui + ShellExecute Launch", and `LC-settings-shell` still
describes an "egui/eframe presentation layer", while `crates/settings` is Slint 1.17 with the Skia
renderer. Pre-existing, unrelated to this decision, and belongs to `wdi-reconcile` rather than to an
apply pass for `DEC-006`. Recorded here so the next reader does not take it as this decision's damage.
