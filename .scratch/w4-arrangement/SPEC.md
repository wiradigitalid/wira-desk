---
id: SPEC-w4-arrangement
companions:
  - chord-table.md
  - failure-modes.md
  - architecture-diagrams.md
sources:
  - .what/_prd/wira-desk/prd.md
  - .what/business-rules.md
  - .what/window-management/SRS-window-management.md
  - .what/window-management/02-rules/rules-window-management.md
  - .what/window-management/03-domain/domain-model.md
  - .what/window-management/04-usecases/UC-2-snap-window-half.md
  - .what/window-management/04-usecases/UC-7-move-window-next-monitor.md
  - .what/window-management/05-scenarios/SCN-03-duplicate-chord-unbinds-later-action.md
  - .what/settings/SRS-settings.md
  - .what/settings/02-rules/rules-settings.md
  - .what/settings/04-usecases/UC-4-change-shortcut.md
  - .what/settings/04-usecases/EXPERIENCE.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/window-management/SDD-window-management.md
  - .how/window-management/04-components/LC-arrangement-engine.md
  - .how/window-management/05-model/data-model.md
  - .how/window-management/06-flows/flow-monitor-move.md
  - .how/settings/SDD-settings.md
  - .how/settings/01-ux/DESIGN.md
  - .how/settings/05-model/data-model.md
  - .control/decisions/DEC-007-inter-monitor-movement-becomes-a-product-command.md
  - .control/decisions/DEC-008-arrangement-chords-move-to-the-ctrl-alt-family.md
  - .control/decisions/DEC-009-a-duplicate-chord-unbinds-the-later-action.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# W4 — Arrangement grows to nine actions

## Why

A vision to realize, and one shipped defect to close. The product already treats the monitor as a boundary it enforces — cycling never leaves the screen the user is looking at — yet it offers no way to cross that boundary on purpose, and it can only divide a screen left and right. On a short laptop panel, left and right halves are the useless division; top and bottom are the useful one. Both gaps were the owner's own report.

The defect is separate and worse. The shipped snap defaults are `Ctrl+Win+Left` and `Ctrl+Win+Right`, which are Windows' own chords for switching virtual desktops. The low-level hook sees them first and swallows them, so the product silently takes a shell function — exactly what `DEC-003` forbids, on a chord `DEC-003` itself cited as evidence the defaults were well behaved.

Affected: every user of the snapping feature, all of whom have lost virtual-desktop navigation without being told; and any user on a screen too short for vertical thirds to help.

## Capabilities

- **CAP-2** — *Snap and resize the active window with keyboard shortcuts, DPI-aware per monitor.* Extended by this wave.
  - **intent:** A user can put the active window in any one of the four halves of its monitor's work area, from the keyboard.
  - **success:** Pressing the top-half chord leaves the window occupying the upper half of the work area with the taskbar uncovered; the top and bottom halves together cover the work area with no uncovered row and no overlapping row between them; an odd pixel height divides the same way on every press, so repeating a chord never shifts the window.

- **CAP-12** — *Move the active window to another physical monitor from the keyboard, keeping its share of the work area.* New in this wave.
  - **intent:** A user can move the active window to the next attached monitor without losing the arrangement they just built.
  - **success:** A window occupying the left half of one monitor's work area occupies the left half of the next monitor's work area after the move, including when the two monitors differ in resolution or display scaling. With one monitor attached, nothing moves and nothing is reported. Pressing the chord once per attached monitor returns the window to where it started. The window remains on the virtual desktop it was on.

- **CAP-3** — *Load user preferences from a local TOML file.* Extended by this wave.
  - **intent:** The daemon behaves predictably when configuration names one chord for two actions, instead of picking a winner in silence.
  - **success:** At startup, the action earlier in the declared precedence order keeps the chord, the later action is unreachable, and exactly one warning names both fields and the chord — while every unrelated setting in the file still takes effect. On an explicit reload, the whole candidate configuration is refused and every actor stays on its last-known-good snapshot.

- **CAP-5** — *Settings and onboarding via a separate binary.* Extended by this wave.
  - **intent:** A user can find and rebind any of the nine editable chords in one place, grouped so nine rows stay scannable.
  - **success:** Every editable action appears as exactly one row in the Shortcuts pane; the pane's draw order, its keyboard focus order, and the collision precedence order are all read from one declared sequence; no chord field exists in any other pane; and the pane that holds only the overlapping-stack toggle and its width slider no longer claims to hold snapping.

## Constraints

- Command wire values are **extended, never renumbered**. `0`–`5` keep their present meanings permanently; the three new actions take `6`, `7`, `8`. A value outside the assigned set decodes to `Nop`, never to an error. A queued command carries only the number, so renumbering silently changes what a queued command means (AD-2).
- The test `command_set_contains_no_inter_monitor_arrangement` is **replaced, not deleted**. Its successor sweeps the whole new range and asserts the same completeness property over it, so the command set stays closed at its new size (DEC-007).
- The attached monitor set, each monitor's work area, and every `HMONITOR` are enumerated fresh per command and cached nowhere — not in a `static`, not memoized, not behind a display-change subscription. An `HMONITOR` is a handle, not an identity (AD-14).
- A window moved between monitors is placed by its **share** of the destination work area, never by copying its pixel width and height (LBR-WM-7).
- A window Windows still considers maximized is restored to its normal state before it is placed. The maximized state is bound to the monitor it was maximized on, and the window otherwise springs back (LBR-WM-7, UC-7).
- No planner scales by DPI. Coordinates arrive already in physical pixels from a Per-Monitor-V2-aware process; applying DPI would scale twice. The `dpi` field is carried for traceability only.
- Every arrangement plan is computed inside the **work area**, never full monitor bounds. Reaching monitor bounds covers the taskbar.
- Arrangement never resolves a target belonging to Wira Desk itself. The chord stays consumed; nothing moves, nothing is retargeted, no popup (LBR-WM-6, DEC-006).
- Monitor enumeration runs on the worker thread, never inside the hook callback. The callback is budgeted under 10 ms and must not allocate (NFR-2, NFR-3, AD-2).
- Existing `config.toml` files are **not** rewritten. Every field carries `#[serde(default)]`, so an install predating this wave keeps every value it holds and gains only the new fields (DEC-008).
- `Win + Ctrl + Left` and `Win + Ctrl + Right` become `Reservation::ShellOwned` with owner text naming virtual-desktop switching. They cannot be configured for any action after this wave (DEC-008).
- Row order in the Shortcuts pane is the declared sequence, and it is also the collision precedence order. Grouping under headings may gather rows; it must not reorder them (LBR-ST-14).
- The daemon never writes `config.toml`. Only the settings process does (BR-1, AD-5).
- Every new `unsafe` block carries a `SAFETY:` comment stating the precondition it actually relies on. `undocumented_unsafe_blocks` and `missing_safety_doc` are `deny` in the workspace lints.
- No corpus file may be edited to make code fit. A deviation from an SDD or an `AD-N` is reported and becomes a new `DEC-`, never absorbed as a patch.

## Non-goals

- Moving a window to a *named* monitor (primary, secondary). Next-and-wrap needs no monitor identity; Windows offers no cheap identity that survives unplug and sleep (DEC-007, PRD §8.2).
- Direction-based movement ("the monitor to the left"). Coordinate order is undefined for vertically stacked or L-shaped arrangements (DEC-007).
- Correcting the cross-DPI frame-inset error. The visible frame lands a few pixels off when source and destination scaling differ; accepted rather than fixed, because a second pass would have to wait on Windows' asynchronous DPI-change reflow (DEC-007).
- Migrating existing `config.toml` files to the new chord family (DEC-008).
- Showing the **unbound action** state anywhere in the Settings UI. `DEC-009` accepts that silence and names the route out of it; this wave does not build it.
- Reserving or suppressing Windows' own `Win + Shift + Arrow`. Two chords will move a window between monitors and disagree about its arrangement (DEC-007).
- A fourth Settings pane, or splitting chord fields across panes. The capture lease is armed from which pane is showing, so two panes holding chord fields regresses the key check (DEC-004, DEC-005).
- Vertical thirds, quarters, or any grid beyond halves and maximize.

## Success signal

On a laptop docked to a larger external monitor at a different scaling factor: a specification snapped to the top half of the laptop panel and a terminal to the bottom half meet exactly, with no gap; moving the specification to the external monitor leaves it occupying the top half *there*, and the terminal and browser have not moved. Undocked, the move chord does nothing at all — no jump, no message, no error — and the snap chords keep working. Configuring `Win + Ctrl + Left` for any action is refused with a message naming virtual-desktop switching. On a machine whose `config.toml` already bound `ctrl+alt+down` to the overlapping stack, the daemon starts with the stack unreachable, the tray showing its warning state, and one log line naming both fields — not with the stack silently dead.

## Assumptions

- `Ctrl + Alt + Arrow` remains reachable on most machines despite graphics-driver control panels binding screen rotation to the same chords. Filed as `OQ-20`; `DEC-002` forbids settling it by probing.
- The order `EnumDisplayMonitors` reports monitors matches physical arrangement closely enough that "next monitor" is not surprising. Filed as `OQ-21`; recorded as a reversal trigger in `DEC-007`.
- `SetWindowPos` does not change virtual desktop membership. Read from what the call affects rather than measured on a running multi-desktop session.

## Open Questions

- None blocking. `OQ-20`, `OQ-21`, and `OQ-22` are filed in `.control/questions/assumptions.md` and none holds this wave: the first two need a user's own hardware, and the third is a layer-boundary cleanup wider than this change.
