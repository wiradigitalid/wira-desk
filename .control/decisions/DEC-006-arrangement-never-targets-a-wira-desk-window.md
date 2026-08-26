---
type: decision
id: DEC-006
status: draft
touches: []
supersedes: null
superseded_by: null
created: '2026-08-26'
---

# DEC-006 — Arrangement never targets a Wira Desk window, and the Settings window's painted size is its Win32 size

## Decision

Two halves. The first decides the behaviour a user sees; the second makes that behaviour an invariant
rather than a courtesy one caller extends.

**Arrangement resolves no target when the foreground window belongs to Wira Desk.** The gate lives in
`resolve_context_for` — the single chokepoint every arrangement command already passes through — and
not in the hook.

| Foreground window | Snap left / right / maximize, overlapping stack |
|---|---|
| Any other application's window | Planned and applied, exactly as today |
| `wiradesk-settings.exe` — the Settings window or the onboarding modal | No target resolved. Nothing moves |
| The daemon's own window | No target resolved. Nothing moves |

Identification is by the **process image basename** of the target's process, compared against
`SETTINGS_EXE_NAME`, plus a comparison against the daemon's own process id. Not by window class — the
UI toolkit registers a generic one that names nothing. Not by a process id the Settings process
registers with the daemon: `OQ-17` already records why an id is the wrong handle to hold.

The disposition is a **consumed no-op**, not a passthrough. The chord stays claimed: the hook has
already swallowed it by the time the Worker resolves a target, and the Worker's refusal surfaces as a
Tier-2 diagnostic under `AD-7` — no popup, no toast, no window movement. Passthrough is refused
explicitly, because `Ctrl+Win+Left` is Windows' own previous-virtual-desktop chord and `reservation()`
deliberately does not reserve `Ctrl+Win+Arrow`. Handing it back would trade a ghost frame for a
surprise desktop switch.

Retargeting to some other window is refused. See Alternatives.

**The Settings window's painted size is its Win32 size.** The window is frameless, transparent, and
laid out at a fixed size, so a frame larger than the layout is not a cosmetic mismatch — it is a
region the user cannot see that still owns its hit-test area. Any external size change is clamped at
the window's own message boundary, in `WM_WINDOWPOSCHANGING`, before the toolkit sees it. Position
changes pass through untouched; only size is clamped.

## Why

The reported symptom: with the Settings window focused, pressing a snap or stack chord left a large
invisible region over the desktop that swallowed mouse clicks. The cause is a window that lies to the
operating system. `SetWindowPos` grows the outer frame to half-screen or full-screen; the layout stays
at its fixed size and keeps painting in the top-left corner; and the rest of the frame is transparent
but still hit-tests to that window, because DWM's composited alpha has nothing to do with hit-testing.

Nothing in the stack resists this, and that was verified by reading it rather than assumed:

| Layer | What it does | Does it stop `SetWindowPos` |
|---|---|---|
| Slint | Sets the window non-resizable and installs min and max inner size when layout constraints are fixed | No — it delegates to the windowing layer |
| winit `WM_GETMINMAXINFO` | Writes `ptMinTrackSize` and `ptMaxTrackSize` from those constraints | No. Those fields govern user drag-resize *tracking* and maximize, not an arbitrary `SetWindowPos` |
| winit `WM_WINDOWPOSCHANGING` | Tracks which monitor a fullscreen window moved to | No. It never inspects or clamps `cx`/`cy` |

The second row is the load-bearing one, and it is also the proof: `ptMaxTrackSize` is **already** being
set on this window, and the symptom still occurs. A decision to clamp there would have been written,
shipped, and found to change nothing. The message that can refuse a programmatic resize is
`WM_WINDOWPOSCHANGING`, whose `WINDOWPOS` is writable before the default handling runs. The arrangement
path posts with `SWP_ASYNCWINDOWPOS`, so the request is queued to the target's own thread and its
message loop does see it.

**Why both halves, and not just the first.** The exclusion fixes exactly one caller. `SetWindowPos` on
that window is reachable by any third-party tiling tool and by any script, and the same ghost frame
comes back with none of Wira Desk's code involved. Windows' own Snap is well behaved here — it honours
the absent `WS_THICKFRAME` and moves a fixed-size window without resizing it — so the operating system
is not the risk. Other window managers are. The clamp is what makes the invariant true; the exclusion
is what makes the product's own behaviour sensible.

**Why the gate is not in the hook.** It looks like a sibling of the VM/RDP bypass (`AD-6`) and it is
not. That bypass answers *whose chord is this*, and its disposition is passthrough because the chord
belongs to the guest. This answers *is this a legal target*, and its disposition must be a swallow.
Putting it beside `eval_bypass` would inherit the wrong disposition and spend a process-identity query
inside a callback budgeted under 10 ms (`NFR-2`, `NFR-3`), to reach a conclusion the Worker can reach
for free — the Worker already pays that cost on the stack path through `capture_active_context`. One
gate in `resolve_context_for` covers all four commands, because all four already resolve their context
there.

**Why exclusion is a target rule and not a new capability.** `AD-4` already establishes
executable-name identity as this product's exclusion mechanism, and `LBR-WM-2` already establishes
that the product filters targets rather than acting on whatever the OS hands it. This extends a
mechanism the architecture ratified; it does not introduce one.

## Cost

A chord silently does nothing. A user who presses `Ctrl+Win+Left` on the Settings window gets no
window movement and no message — the diagnostic goes to a Tier-2 log they are not reading. That is the
same silence `DEC-002` accepted for a different cause, and it is a real cost rather than a
technicality: the product's own window is the one place where "the shortcut is broken" and "the
shortcut is correctly declining" look identical.

The clamp adds Win32 message interception to a crate that currently has almost none, in a codebase
where `undocumented_unsafe_blocks` is `deny`. It is a small amount of code carrying a real maintenance
obligation: a `SAFETY:` note stating the actual precondition, and a subclass whose lifetime is tied to
a window the toolkit owns.

Basename comparison is spoofable. Any process that renames its executable to `wiradesk-settings.exe`
becomes un-arrangeable. This is accepted for the same reason `AD-4` accepted it for cycling — the
alternative costs more than the failure does — but it is written down rather than left implied.

The clamp makes the Settings window immovable in size to *everything*, including a future Wira Desk
feature that legitimately wants to resize it, and including a user who would rather their third-party
tiler win. There is no per-caller exception, and this decision does not propose one.

## Alternatives

**Clamp in `WM_GETMINMAXINFO` via `ptMinTrackSize` / `ptMaxTrackSize`.** Refused: measured against the
code, it is already happening and it does not work. Those fields do not govern `SetWindowPos`.

**Identify Wira Desk's windows by window class.** Refused: the toolkit registers a generic class that
identifies nothing. The existing single-instance check reaches for the window *title* instead, which is
worse — any window titled "Wira Desk" would match.

**Have the Settings process register its process id with the daemon at startup.** Refused. `OQ-17`
already records the failure mode for exactly this shape: Windows recycles process ids, so a
registration left behind by a crash points at an id an unrelated process can inherit, and an innocent
application silently stops being arrangeable with nothing on screen saying why. A basename comparison
holds no state and cannot go stale.

**Retarget to the most recent non-Wira-Desk window.** Refused twice. It requires an `EnumWindows` sweep
and an eligibility policy inside the arrangement module, which is deliberately decoupled from both —
that module duplicates `is_cloaked` rather than importing it, to keep that boundary. And it moves a
window the user cannot see, behind the one they are looking at: an outcome they cannot verify at the
moment they cause it, which is the failure `DEC-005` refuses under a different name.

**Pass the chord through to Windows instead of swallowing it.** Refused: `Ctrl+Win+Left` and
`Ctrl+Win+Right` are Windows' virtual-desktop chords, and `reservation()` does not reserve them
precisely because Wira Desk claims them. Passthrough replaces an invisible frame with a desktop switch
the user did not ask for.

**Make the Settings window genuinely resizable and let its content fill the frame.** This is the option
that dissolves the problem class instead of guarding it, and it is the most native answer — the Windows
Settings app is resizable and snappable. Refused for now on cost, not on merit: the five-pane shell and
its onboarding modal were laid out against fixed Fluent 2 metrics, and reflow rules for every pane at
arbitrary sizes is a UX wave, not a guard. Named here so the reversal trigger below has something to
point at.

## Reversal trigger

Revisit the clamp if the Settings window is ever made resizable — at that point the clamp is the thing
standing in the way, and both halves of this decision collapse into the resizable design.

Revisit the exclusion's silence if the no-op is reported as a bug by anyone other than its author. The
honest fix is a visible one — the window saying why it declined — and that is a `settings` change this
decision deliberately does not make.

Revisit the basename comparison if arrangement ever gains a target set wider than the foreground
window, because a sweep would apply this rule to hundreds of windows per command rather than one, and
the cost calculus that put it on the Worker changes with it.

## Trace

Came from an owner report on 2026-08-26 — an invisible click-swallowing region left behind after
snapping with the Settings window focused — and from the consultation that followed it. The mechanism
was verified by reading `arrangement/win32.rs`, the Slint window definition, and the installed
`i-slint-backend-winit` and `winit` message handlers; the symptom itself is the owner's observation and
was not reproduced under instrumentation.

Extends `AD-4`'s executable-name identity to arrangement targeting, and sits beside `LBR-WM-2`, which
already filters targets for cycling. Deliberately **not** placed beside `AD-6`'s hook-thread bypass,
for the reason given under Why. Reuses `OQ-17`'s finding about process-id reuse rather than
rediscovering it. Opens `OQ-18` and `OQ-19` for the two things that cannot be settled from a desk.
