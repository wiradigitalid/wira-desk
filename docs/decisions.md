# Design decisions

Why parts of this codebase look the way they do. Everything here is a decision whose
reasoning is not visible from the code alone — usually because the obvious alternative was
tried first and broke something, or because a Win32 API does not behave the way its name
suggests.

Read this before "simplifying" anything in the keyboard hook, the cycling order, or the
activation path. Several of the shapes below look redundant and are not.

## The keyboard hook

**The callback allocates nothing, locks nothing, logs nothing, and touches no file.** It runs
on every key event on the desktop, so anything unbounded there is felt as input lag
system-wide. The VM/RDP identity check obeys this too: `HookIdentityCollector` owns fixed
arrays and reuses them per event, comparisons run directly between pre-normalised policy
strings and raw UTF-16, and diagnostics are deferred to an atomic counter the Worker reports
later.

**A modifier release is never swallowed.** An earlier version swallowed the `Win` key-up to
stop the shell opening the Start Menu. The focused application then believed `Win` was still
held, so every later keystroke became `Win`+key — a far worse bug than an occasional Start
Menu. The shortcut's main key is still swallowed; the modifier release always gets through.

**`SendInput` is never called from inside the callback.** Suppressing the Start Menu needs an
unassigned key injected while `Win` is still down, which turns a lone press into a
combination. Doing that from the callback raced the activation the Worker was performing and
stopped cycling from moving focus at all. It now runs on the Worker, after activation, and
only if `Win` is genuinely still held.

**Synthetic input is rejected.** Since the daemon injects a key itself, processing injected
events would let the hook consume its own injection.

**The bypass latch exists because a chord has two halves.** When a matched chord turns out to
belong to a VM or Remote Desktop guest, the whole chord must pass through — including the
releases. Evaluating the bypass per event breaks that: focus can change mid-chord, letting a
key-down through while the matching key-up is swallowed, which leaves a modifier stuck inside
the guest session. So the decision is latched once and cleared only when every modifier is
released.

**The bypass is evaluated after the shortcut matches, not before.** A matched chord is rare;
a non-matching keystroke is the common case and must not pay for an identity query.

## Cycling order

**Candidates rotate least-recently-used first, not next-in-Z-order.** Taking the first
candidate after the active window seems obviously right and deadlocks at two windows, because
Windows raises a window whenever it is activated:

```text
[A B C] -> pick B -> [B A C] -> pick A -> [A B C] -> pick B -> ...
```

C is never reached. Reversing the order fixes it, and two-window behaviour is unchanged.

**Cloaked windows are excluded, and cloaking is a separate fact from visibility.**
`IsWindowVisible` only reports the `WS_VISIBLE` style bit, which cloaked windows keep set —
DWM simply never draws them. Suspended UWP surfaces and windows belonging to another virtual
desktop all look like ordinary windows through that check, which is how an invisible
full-screen window ends up in the rotation. `DWMWA_CLOAKED` answers the actual question, and
it reads DWM's own state without messaging the owning thread, so a hung window answers as
fast as a healthy one. A failed query degrades to *not* cloaked, keeping a real window
reachable rather than silently dropping it.

`WS_EX_NOREDIRECTIONBITMAP` looks like a useful second marker for the same problem. It is
not: a window-enumeration probe confirmed Chrome's genuine windows carry it too, so using it
would remove Chrome from cycling entirely.

**Nothing is cached between commands.** One `EnumWindows` sweep per accepted shortcut, no
`static`, no memoisation. Window state changes constantly, and a stale candidate list is
worse than a slightly more expensive sweep.

**A process id is used only to open the process; it never becomes identity.** The executable
basename is the sole same-application key, so two windows of the same app group together even
across separate processes.

**One scratch buffer per sweep, not per window.** Resolving an executable path needs a buffer
large enough for extended-length paths, and allocating one per window meant tens of megabytes
of short-lived allocation per keystroke on a busy desktop. The buffer is now owned by the
sweep and reused. The capacity is unchanged — the fix was the churn, not the size — and the
invariant that makes reuse safe is that the API's in/out length parameter is reset to the
buffer's full length on every call.

## Activation

**`SetForegroundWindow` returning `TRUE` does not mean focus moved.** Windows returns success
even when it merely flashes the taskbar button, which is documented behaviour when the caller
lacks foreground rights. Trusting the return value made the daemon log a successful
activation while the user's focus had not moved — and because success ends the pass, no other
candidate was tried. Activation is therefore confirmed by reading `GetForegroundWindow`.

**The confirmation polls briefly, and that is not a bug.** Windows applies a foreground change
asynchronously, so an immediate read catches the transition when no window owns the foreground
yet, reporting a successful activation as a failure and moving on to another window — visible
as focus flicker. The wait is on the OS applying the change, never on the target application
responding, which is what keeps hung windows treated exactly like healthy ones.

**`BringWindowToTop` was removed deliberately.** It raises a window without focusing it, so
when the foreground request then failed the user was left looking at a raised but inactive
window. A successful `SetForegroundWindow` raises the window on its own.

**`AttachThreadInput` has a direction, and getting it wrong fails silently.** Windows grants
foreground rights to the thread owning the foreground input queue, so *this* thread must
attach to the foreground thread in order to inherit them. Attaching the foreground thread to
the target thread — the original mistake — leaves the caller with no rights at all, so the
fallback never worked.

**Foreground rights follow the last input event.** Windows attributes them to the process that
received it, which is why activation succeeds under a real keypress and cannot be driven by
synthetic input at all: injected input is attributed to the injecting process. Any test
harness that posts commands instead of using physical keys will see activation fail no matter
how healthy the product is.

## Context safety

**The two context decisions fail in opposite directions, on purpose.**

| Decision | On uncertainty | Why |
| --- | --- | --- |
| Spatial eligibility (monitor, virtual desktop) | Fail **closed** — not eligible | Guessing throws focus out of the user's workspace |
| Foreground bypass (VM / Remote Desktop) | Fail **open** — pass the keystroke through | Guessing swallows a keystroke inside a guest session |

Because of that split, `Option` in these contracts means **unknown**, not absent. The two
readings lead to opposite decisions, so the distinction is documented at the type.

**`MONITOR_DEFAULTTONULL`, never `DEFAULTTONEAREST`.** A window intersecting no monitor must
report unknown so the contract can fail closed. `DEFAULTTONEAREST` would silently attribute it
to some monitor and let cycling jump workspaces.

**A positive bypass match wins even when the other identifier is unknown.** Requiring both
would stop the bypass for a VM window whose class lookup happened to fail — exactly when
being conservative matters most.

**The virtual-desktop COM interface is declared by hand** because no binding ships for it.
Interface calls dispatch purely on vtable slot offset, so a reordered field or a mistranscribed
GUID would not fail loudly — it would call a different function through a different signature.
Both GUIDs are pinned nibble by nibble and every vtable slot is pinned individually, since
swapping two pointer-sized fields leaves the struct's total size unchanged. The interface is
created once per thread and kept, because creating it per keystroke meant a COM initialisation
and instantiation on every shortcut press.

## Window arrangement

**Rectangle edges are half-open.** That is not a style preference: it makes "left half plus
right half exactly covers the work area" true by construction, instead of true via an
off-by-one correction that someone will eventually forget.

**Both halves are derived from a single split value,** so the left rectangle's right edge and
the right rectangle's left edge are always equal. An odd width splits deterministically.

**Placement uses the monitor work area, not its full bounds,** so a maximised window does not
cover the taskbar.

**Converting a Win32 `RECT` is a field copy with no coordinate adjustment.** The absence of
adjustment is the point — it is what keeps physical pixels intact across DPI contexts. DPI is
carried for traceability and never used to rescale.

**`SWP_ASYNCWINDOWPOS` is pinned by a test,** and the test references the same flag constant
the production path passes. Losing that flag would not be visible in review, but it would let
a hung window block the Worker, which a user feels immediately.

**Stacked windows are distributed across the travel** — work-area width minus window width —
rather than across the full width, which keeps every rectangle inside the work area by
construction. Intermediate arithmetic widens before narrowing so a large monitor cannot
overflow mid-calculation.

**Stacking compares monitor handles directly** rather than going through the spatial contract.
The duplication is deliberate: stacking must work whether or not virtual-desktop filtering is
available.

## Configuration

**A reload is all-or-nothing.** An unreadable, malformed, or semantically invalid file leaves
every actor on its last-known-good configuration and emits exactly one warning. A partially
applied reload would be worse than a rejected one, because the user would have no way to tell
which half took effect.

**An unparseable shortcut is defaulted at startup but rejected on reload.** Startup must bring
the daemon up. Reload must not silently ignore half of what the user just saved while
reporting success.

**The reload path deliberately does not use the "load or default" helper.** That helper
substitutes defaults for a corrupt file, which is right at startup and wrong here: it makes a
corrupt file indistinguishable from a valid one, and telling those apart is this module's
entire job.

**Actors receive owned snapshots by explicit message passing, never shared state.** The hook
owns its shortcut and bypass configuration; the Worker owns arrangement configuration. Each
snapshot is collected by the thread that owns it, so "never mutated concurrently" is
structural rather than a convention. The wake-up message itself carries no pointer — the
snapshot waits in a slot inside the process, and ownership follows one rule: whoever takes a
snapshot out of the slot owns it and frees it.

**Modifier order in a shortcut's canonical form is frozen.** Without a fixed order,
`win+ctrl+a` and `ctrl+win+a` produce two different strings for the same shortcut and every
text comparison is silently wrong. Validation returns the canonical form rather than unit, so
a caller structurally cannot store the user's raw input.

**The reload signal is emitted only after the write returns,** or the daemon would read a
half-written file.

## Settings UI

**A separate style is applied to every theme, not just the active one.** The GUI toolkit
stores one style per theme, so styling only the current theme leaves default-thin focus
indicators the moment the user's OS theme changes — an accessibility regression that is
invisible until it happens.

**The accessibility feature is requested explicitly** because it is not on by default. Without
it the UI Automation tree is never published, so every accessibility criterion fails silently
while the application looks perfectly fine.

**Font bytes are validated before being handed to the toolkit,** so a corrupt system font
degrades to the fallback chain instead of panicking the whole application. Font installation
is separated from theme application because installing fonts rebuilds the glyph atlas and must
not run on every theme change. The typeface actually in use is shown in the About panel, so a
fallback is visible rather than silently assumed.

**All decisions live outside the render closure.** That separation is what makes the UI tests
runnable without a window; had the state lived inside the draw callback, this surface could
only be checked by hand.

**First-run onboarding is launched from the hook-ready handler, not from startup,** so the tray
icon already exists. Otherwise a user who closes the tutorial is left with no visible sign the
daemon is running.

## Process and elevation

**Elevation exists for one purpose:** activating and moving windows owned by higher-integrity
processes, which Windows blocks otherwise. The manifest is not the only check — the daemon
re-queries its own token at startup and refuses to run unelevated, so a binary rebuilt without
the manifest fails loudly instead of half-working.

**The DLL search path is narrowed as the first statement in `main`,** before anything else
runs, because a planted DLL beside an elevated executable would load with its privileges.

**A squatted single-instance mutex is treated as "already running".** Continuing would be
worse: two live global keyboard hooks competing for the same shortcut.

**The test harness needs an environment variable** because a build script's output applies to
every target of its crate. The daemon's elevation manifest therefore also applied to the test
binary, which could not launch at all. Skipping the manifest is safe against misuse precisely
because the startup token check is independent of it.

**The auto-start task stores an absolute path and sets no working directory,** so the task
action itself cannot be hijacked. Where the binary is installed still matters — see
`../SECURITY.md`.

## Testing conventions

**Never assert on a shared global from a parallel test.** Counters and slots that live in a
`static` are visible to every test thread; asserting on them directly produced a flaky failure
once already. Either inject the state or serialise the tests that touch it.

**Pin a behaviour against the same constant the production path uses,** never a value the test
recomputes. A test that rebuilds its own copy of a flag set can never disagree with itself,
which makes it look like a guard while guarding nothing.

**Percentiles use nearest-rank, not interpolation,** so a reported value is always a sample
that was actually observed rather than one synthesised between two others.

**A test that passes for the wrong reason is worse than a missing test.** An assertion that
the daemon "never jumps to another application" passes trivially whenever the run produced no
activation at all; such an assertion needs a precondition that the interesting path was
actually reached.
