---
type: rules
scope: component
component: window-management
status: reviewed
created: '2026-08-21'
updated: '2026-09-03'
---

# Business Rules — window-management

Local component business rules binding the `window-management` Product Component. Global cross-component rules (`BR-1` through `BR-7`) live in `.what/business-rules.md`.

## Rules

| id | Rule | Binds | Source | Status |
| --- | --- | --- | --- | --- |
| LBR-WM-1 | A modifier superset or an extra key chord beyond what is configured (e.g. `Win+Ctrl+\`` when only `Win+\`` is configured) must not trigger cycling or snapping, and the keystroke must reach the rest of the system unaffected. | `window-management` | FR-6 | active |
| LBR-WM-2 | A minimized, cloaked, tool, shell-surface, or ghost window is excluded from cycling candidates; an unresponsive ("Not Responding") normal window remains eligible. | `window-management` | FR-4, FR-5 | active |
| LBR-WM-3 | Window enumeration and candidate inspection must use only non-blocking queries; a blocking cross-process call must never be used on the path that serves a keypress. | `window-management` | NFR-4, AD-2 | active |
| LBR-WM-4 | When the hook-to-worker command channel is full, a newly intercepted command is dropped immediately rather than blocking the input stream. | `window-management` | NFR-3, AD-2 | active |
| LBR-WM-5 | When hook failure reaches the escalation threshold, exactly one toast notification is dispatched per failure episode; further notifications stay suppressed until hook health is restored. | `window-management` | AD-7 | active |
| LBR-WM-6 | A window belonging to Wira Desk itself must never be an arrangement target; the chord is consumed and nothing moves, is retargeted, or raises a popup. | `window-management` | DEC-006 | active |
| LBR-WM-7 | A monitor-move command visits monitors in one fixed order, sampled fresh per invocation, wrapping from the last back to the first; the destination is derived from the window's share of its source work area, never from copying pixel dimensions; a maximized window is restored before being placed; with one monitor attached the command is a successful no-op; the virtual desktop never changes as a side effect. | `window-management` | FR-23, DEC-007, AD-14 | active |
| LBR-WM-8 | A half-screen snap divides the work area at one boundary computed fresh on every press, so the two halves exactly tile the work area with neither a gap nor an overlap; an odd extent gives the floor to the first half; a half that would be empty is refused rather than emitted as a zero-extent placement. | `window-management` | FR-14, FR-22 | active |
| LBR-WM-9 | A custom-percentage edge snap resizes the active window to the percentage configured for that edge — of the work area's width for left/right, of its height for top/bottom — computed fresh on every press and independent of every other edge's configured percentage; a percentage that would produce a zero or negative extent is refused rather than emitted as a degenerate placement. | `window-management` | FR-26 | active |
| LBR-WM-10 | A thirds snap divides the work area's width into three columns computed fresh on every press, so the three columns exactly tile the work area with neither a gap nor an overlap; a width not evenly divisible by three gives the remainder to the middle column; a column that would be empty is refused rather than emitted as a zero-extent placement. | `window-management` | FR-27 | active |

## Rationale — LBR-WM-6

The settings window is frameless, transparent, and fixed-size, so an external resize can leave an invisible region that still owns its hit-test area and would otherwise swallow mouse clicks. Passing the chord back to Windows instead of consuming it would fire its own virtual-desktop action, which `DEC-006` refuses.

## Rationale — LBR-WM-7

Coordinate ordering is undefined for vertically stacked or L-shaped arrangements, and a named monitor needs an identity Windows does not offer cheaply (`DEC-007`). Proportional mapping is what makes an arrangement survive a move between monitors of different size or scaling. Where the destination placement's clamp resolves against belongs to `DEC-010`, not to this rule — this rule states only the promise the user sees.

## Rationale — LBR-WM-8

Both halves being derived from one boundary makes "the halves exactly tile the work area" true by construction rather than by an off-by-one convention every reader has to remember, and it holds for both axes so the vertical division added at FR-22 inherits the same guarantee rather than reinventing it.

## Rationale — LBR-WM-9

Each edge's percentage is independent rather than paired with its opposite edge, because a single-key press is not a two-window layout decision the way `FR-14`'s halves are — the user snapping left at 70% is not promising anything about what happens if they later snap right, so there is nothing to keep tiled and nothing gained by coupling the two.

## Rationale — LBR-WM-10

The remainder goes to the middle column rather than the first, unlike `LBR-WM-8`'s floor-to-the-first-half rule: a thirds layout is read as symmetric (left, center, right), and a stray pixel on an outer column would be visible against the other outer column in a way the same pixel hidden in the middle is not.

## Retired

*(No retired local rules)*
