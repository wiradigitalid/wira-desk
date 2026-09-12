---
spec: SPEC-13
release: "0.4.0"
prd: wira-desk
fr: []
status: closed
---

# SPEC-13 — Same-App Visual Switcher: Hold the Cycle Chord to See the Windows

## Problem Statement

Wira Desk cycles focus between windows of the active application with ``Win + ` `` (`FR-1`),
confined to the active monitor and virtual desktop (`FR-2`). The cycle is blind: it moves
focus and shows nothing. Between two and three windows that is the right behaviour and the
fastest possible one. From six windows upward it stops working as navigation — the user
cannot see which window is next, cannot see how many there are, and reaches the one they
want by stepping through every window in between.

Windows offers nothing that closes this gap. `Alt + Tab` mixes every application on the
desktop into one list. `Win + Tab` replaces the whole screen with a task overview of
everything. Taskbar thumbnail peek needs the mouse, a hover, and a squint at the bottom edge
of the screen. macOS has App Exposé; Windows has no equivalent.

The owner's ask, recorded in `.work/wdi-daily-what-to-build/`: *like Alt+Tab, but same app*,
plus *if there are more windows than can be displayed, I want a UI/UX solution that is
pleasant to look at*, plus two questions this spec has to answer rather than assert — is
this a big job, and does it badly increase runtime memory.

## Solution

Holding the cycle chord — the one the user already has bound, ``Win + ` `` by default — past a
short threshold puts a heads-up display on the active monitor showing every window the blind
cycle would have visited, as live previews. Releasing the modifier activates the highlighted
one. The tap is untouched.

### 1. Tap and hold, with the tap unchanged

The published success metric `SM-1` states that focus transfer happens under a millisecond
after the keypress. Waiting to see whether a press becomes a hold would break it, so the
switcher does not wait:

- **Key-down** performs the blind cycle exactly as today — same code path, same latency, no
  new work in the hook callback — and arms a hold timer on the Worker.
- **Key-up before the threshold** disarms the timer. Nothing was drawn. The tap behaved
  exactly as it does now.
- **The timer elapsing with the chord's modifiers still physically down** opens the overlay,
  with the selection already on the window the key-down just activated.

A hold therefore raises one window before the overlay appears. That is deliberate, and it is
what `Alt + Tab` does inside an application; the alternative is a delayed first cycle, which
`SM-1` forbids.

The timer starts at the **main key** (`` ` ``), never at the modifier. A user holds `Win` for
far longer than 150 ms before reaching `` ` `` on nearly every invocation, so a
modifier-keyed threshold would open the overlay almost every time and the tap path would be
unreachable.

### 2. The candidate set is the cycling set, never a second one

The overlay shows exactly what `run_context_safe_cycle` would have visited: one
`Win32CandidateSource` sweep, `WindowEligibility`, then the spatial gate (same monitor, same
virtual desktop). It does not re-derive the list and does not query `IVirtualDesktopManager`
on its own — that filtering already exists and already ships.

The eligible-collection half of `run_context_safe_cycle` is extracted into a function both
paths call, and a test asserts the two produce the same set from the same snapshot. A
switcher that shows one list while the tap cycles another is the worst defect this feature
can have, so the shared function is the guard against it rather than a convention.

**Minimized windows stay out.** `WindowEligibility` excludes `iconic` candidates, and `FR-5`
promises a minimized window is not restored by cycling. The overlay inherits that: a
minimized window is not a card, and no icon-fallback path exists for one. Showing minimized
windows is a change to a published promise, not a rendering detail, and belongs to its own
`DEC-` and a PRD change.

### 3. Card order follows `cycle_order`, not Z-order

`cycle_order` rotates from the active window and then **reverses**, deliberately: taking the
next window in Z-order makes the third window unreachable, and the comment in
`cycling/mod.rs` records why. Cards are laid out left to right, top to bottom in
`cycle_order` sequence, so pressing `` ` `` again always moves the halo to the visually next
card. Index 0 is the window the opening key-down already activated.

### 4. Adaptive grid derived from the monitor, and paging past it

Columns are computed from the work area, not from a fixed table:

```
usable   = work_area.width - 2 * MARGIN
cols     = clamp(floor((usable + GUTTER) / (CARD_W + GUTTER)), 1, candidate_count)
rows     = min(ceil(candidate_count / cols), MAX_ROWS)
per_page = cols * rows
```

`CARD_W`, `GUTTER`, `MARGIN` and `MAX_ROWS` (3) are constants. A fixed six-column rule would
overflow the work area of a 1366×768 laptop and waste two thirds of an ultrawide;
derivation handles both without a setting.

Above `per_page` candidates the grid pages rather than shrinking: a page indicator
(`● ○ ○`), and moving the selection past the last card on a page turns to the next page with
the selection on its first card. Paging unregisters the leaving page's thumbnails and
registers the arriving page's — bounded work, bounded handles, at most `per_page` live
thumbnails at any moment.

### 5. Previews via the DWM thumbnail API

`DwmRegisterThumbnail` and `DwmUpdateThumbnailProperties` project each source window into its
card's destination rectangle. DWM composes from the redirection surface that already exists;
nothing is captured, copied, or polled, and `Win32_Graphics_Dwm` is already a `windows-sys`
feature this crate enables.

Two properties of the API shape the design and are not negotiable:

- **A thumbnail composes above the destination window.** Nothing the overlay paints can
  appear on top of a preview. Every piece of chrome — selection halo, title, icon, page dots
  — lives in card area *outside* the destination rectangle, so the layout engine emits two
  rectangles per card: the chrome box and the preview box.
- **DWM letterboxes the source inside the destination rectangle, preserving its aspect
  ratio.** The layout engine does not compute per-window aspect ratios; it guarantees a
  uniform preview box and lets DWM fit inside it.

A registration that fails, and a window whose display affinity is `WDA_EXCLUDEFROMCAPTURE`
and therefore composes black, both degrade to icon and title in the same card. Neither
removes the card nor aborts the overlay.

### 6. The overlay window

`WS_POPUP`, extended `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, shown with
`SW_SHOWNOACTIVATE`, created and pumped on the Worker's existing message loop.

`WS_EX_NOACTIVATE` is load-bearing, not decoration. The active application's identity comes
from `GetForegroundWindow` in `capture_active_context`; an overlay that takes the foreground
makes the daemon the active application and destroys the very context the switcher is
showing. Because focus never moves, `Escape` has nothing to restore — except the window the
opening key-down raised, which is re-activated on cancel so the promise *my original window
stays focused* holds end to end.

**Not layered.** `WS_EX_LAYERED` with per-pixel alpha means `UpdateLayeredWindow` from a
premultiplied 32-bpp DIB the size of the whole overlay; GDI text and `DrawIconEx` write zero
alpha into such a surface, so every glyph and icon needs hand-fixing, and
`DWMWA_SYSTEMBACKDROP_TYPE` (Mica and Acrylic) does not apply to layered windows at all. The
overlay is an ordinary opaque window with rounded corners via
`DWMWA_WINDOW_CORNER_PREFERENCE`, painted in `WM_PAINT` with double-buffered GDI, themed
light or dark.

`FR-9` forbids heavy UI runtimes and COM GUI frameworks in the daemon. GDI adds no
dependency and no runtime: `Win32_Graphics_Gdi` is already enabled. Nothing in this spec adds
a crate to `crates/daemon/Cargo.toml`.

### 7. Navigation, commit, cancel

- `` ` `` or Right advances; `Shift` with `` ` ``, or Left, retreats; Up and Down move a full
  row. Movement past a page edge turns the page.
- Releasing every modifier in the matched chord commits: the highlighted window is activated
  through the existing `Win32Activator`, then `suppress_start_menu()` runs exactly as the
  blind path already does. A target that has closed meanwhile reports `InvalidTarget` and the
  commit falls through to the next candidate, as `run_cycle` already does.
- `Escape` dismisses and re-activates the window that was foreground before the chord.
- Mouse hover moves the halo; a click commits that card. The overlay never activates on
  click, which is what `WS_EX_NOACTIVATE` guarantees.

While the overlay is open the hook swallows **all** keyboard input except the chord's own
modifier releases — `no_modifier_release_is_ever_swallowed` is an existing invariant and this
must not break it. Auto-repeat of a held `` ` `` advances the selection at most once per
physical press.

### 8. Never stuck on screen

A missed key-up — a UAC prompt, a session lock, an RDP connect, another process injecting
input — would otherwise leave a topmost window on screen with the hook eating keys. Three
guards, all required:

- a `SetTimer` tick polls `GetAsyncKeyState` for the chord's modifiers and commits the moment
  they are all up, whether or not the key-up was seen;
- `WM_DISPLAYCHANGE`, a session change, and a foreground change to a window outside the
  candidate set all dismiss;
- an absolute lifetime cap dismisses regardless.

### 9. Shortcuts that cannot arm it

`switcher.shortcut` and `switcher.fallback_shortcut` are free text, and `Shortcut` supports a
chord with no modifier at all — which is why `has_modifier()` exists. With no modifier there
is no release to commit on, so such a chord cycles blind and never arms the visual switcher.
With several modifiers, commit happens when **every** modifier in the matched chord is up.

`FR-3`'s VM and RDP passthrough is untouched: a chord that bypasses does not arm the switcher
either, because arming happens after the bypass decision, not before.

### 10. Configuration

Two fields, added to the **existing** `SwitcherConfig` — `Config` already has a `switcher`
field holding the cycle shortcuts, so a second `switcher` section cannot exist:

- `visual_enabled: bool`, default `true`
- `visual_hold_delay_ms: u32`, default `150`, valid `100..=500`

`SwitcherConfig` carries `#[serde(default)]`, so an existing `config.toml` gains both on
first load without migration. Settings exposes the toggle, and the delay through the numeric
spinner pattern `shortcut_row.slint` already uses for snap percentages — Settings has no
stepper control and this spec does not add one.

Grid geometry is **not** configurable. `max_rows` and `card_width` as settings make the
layout contract untestable — a page size no test can state — and answer a question nobody
asked. Both are constants, sized against the monitor instead.

## User Stories

1. As a power user with many Chrome windows, holding ``Win + ` `` shows me previews of only
   Chrome's windows on this monitor, so I can pick the one I want by sight.
2. As a keyboard-focused user, tapping ``Win + ` `` cycles exactly as fast as it does today,
   with no overlay and no added delay.
3. As a user across several virtual desktops, the overlay shows only windows on the desktop I
   am on, because it shows the same set the cycle already visits.
4. As a researcher with more than a screen's worth of windows, the cards stay legible and the
   extra windows page rather than shrink.
5. As a user on a page, a page indicator tells me where I am in the list.
6. As a multi-monitor user, the overlay appears on the monitor the active window is on.
7. As a laptop user, previews cost no capture loop and no polling, so browsing windows does
   not spin the fans.
8. As a user who changed their mind, `Escape` dismisses and leaves me on the window I started
   from.
9. As a user who prefers blind cycling, one switch in Settings turns the overlay off
   permanently.

## Implementation Decisions

### Domain (`crates/daemon/src/switcher/`, pure, no Win32)

- State: `Inactive` · `Armed { chord, deadline }` · `Active { candidates, selected, page }`.
- `layout.rs` — columns and rows from a work-area rectangle, page count, and per card a
  chrome rectangle plus a preview rectangle.
- `selection.rs` — next, previous, row up, row down, and page turns at the edges, in
  `cycle_order` sequence.

### Wire

`Command` is a `u8` on a sixteen-slot ring; nothing but an opcode can cross it. New opcodes
`20..=26`: `SwitcherDisarm`, `SwitcherNext`, `SwitcherPrev`, `SwitcherUp`, `SwitcherDown`,
`SwitcherCommit`, `SwitcherCancel`. Arming needs no opcode — `Cycle` already arrives.

`ANTI_MACRO_THROTTLE_MS` (50 ms) must **not** apply to the navigation opcodes. It exists to
stop macro spam of window commands; at 50 ms it would drop roughly half of a held arrow key's
repeats and half of `` ` ``'s auto-repeat, which is exactly the input this feature is
navigated with. Moving a highlight is not a window command.

### Thread ownership

The overlay window is created, painted, and destroyed on the Worker — the same thread that
owns the message loop, the COM apartment for `IVirtualDesktopManager`, and every activation.
The hook callback gains no new work beyond one opcode push per key, which keeps `NFR-2` and
`SM-C3` intact.

### Window titles

`cycling/source.rs` forbids `GetWindowText` by design, and that prohibition stays: titles are
read in the switcher's own path, not in the enumeration sweep, and only for the at most
`per_page` windows actually being drawn. `GetWindowTextW` against another process retrieves
the cached caption and is documented not to hang on a hung owner — that is the reason it is
admissible here, and it belongs in the code as a comment rather than as tribal knowledge.

### Lifecycle

`DwmThumbnailHandle` owns one registration and unregisters in `Drop`. The overlay window is
created on first use and destroyed on dismissal along with every handle, so idle cost returns
to what it is today. Every `unsafe` block carries a `SAFETY:` comment — the workspace denies
`undocumented_unsafe_blocks`, so this is a compile error, not a review note.

## Cost

The two questions the owner asked, answered rather than asserted.

**Memory.** `NFR-1` gives the daemon a hard ceiling of ten megabytes. The design sits inside
it because the expensive option was refused: a per-pixel-alpha layered window needs a 32-bpp
back buffer the size of the overlay — an 1800×950 HUD is about 6.8 MB for one buffer — and
that alone would consume most of the ceiling. An opaque window painted in `WM_PAINT` needs a
compatible bitmap for the visible surface only, released with the window. The thumbnails
themselves cost the daemon almost nothing: DWM composes from surfaces that already exist for
windows already on screen, and the per-registration cost is a handle. Expected shape: idle
unchanged, a transient rise while the overlay is up, everything released on dismissal. This
is a design estimate and must be replaced by a measurement before the spec closes — `NFR-1`
is a ceiling, not a hope.

**Binary size.** `NFR-5` allows 500 KB hard. GDI painting, a window class, and the domain
module are kilobytes, not hundreds of them, and no crate is added. Measure at the release
gate.

**Effort.** Four tickets, and the distribution is uneven: the domain module and the hook and
worker wiring are ordinary work against seams that already exist, while the drawing — cards,
icons, titles, theme, DPI, page dots, all in GDI — is the part with no precedent in this
crate and the part most likely to run long. `size: M` is defensible only because the tracer
lands end to end before any of the chrome does.

## Testing Decisions

- **Pure layout and selection.** Columns, rows, page count, page turns and selection movement
  over 1, 2, 5, 8, 14, 20, 35 and 60 candidates, against work areas of 1366, 1920, 2560 and
  3840 logical pixels. No Win32.
- **Set parity.** One test asserts the switcher's candidate set and the blind cycle's eligible
  set are identical for the same snapshot — the guard against the two paths drifting.
- **Order.** One test asserts card order equals `cycle_order`, so the halo never moves
  backwards through the grid.
- **Tap and hold.** Key-down still enqueues `Cycle` with no added delay; a key-up before the
  deadline disarms; the deadline reached with modifiers still down opens. Driven through the
  existing `handle_key_event_with_bypass` seam, which already takes injectable time and
  bypass.
- **Modifier-less chord never arms**, and **every modifier up commits** for a multi-modifier
  chord.
- **Thumbnail lifecycle.** Registrations and unregistrations counted through a
  `ThumbnailSink` trait with a counting fake, covering dismissal and page turns. This is the
  leak guard, so it is seen failing first: remove the `Drop` impl, watch the count go
  non-zero, restore it.

## Out of Scope

- Closing a window from inside the switcher (`Delete` or `Ctrl + W`). Not in the owner's
  notes, destructive and unrecoverable from one keystroke, and it forces the candidate list
  to mutate mid-session — the hardest transition in the feature, bought for a use nobody
  asked for. A separate spec if it is ever wanted.
- Showing minimized windows — an `FR-5` change, not a rendering one.
- Switching across applications, or merging several monitors into one grid.
- Drag-to-rearrange from inside the switcher.
- Blur, Mica, or Acrylic behind the overlay.

## Open Before Build

- **This is a new capability and the corpus has no requirement for it.** `FR-1`, `FR-2` and
  `UC-1` are the blind cycle, already delivered; claiming them would record this work as
  something it is not. The precedent is `SPEC-8`, which minted `CAP-17`, `FR-30`–`FR-32` and
  `UC-13`–`UC-14` for driverless mouse navigation. `SPEC-13` owes the same: a new `CAP`, new
  `FR` rows, and a `UC`. Both components are `mode: deep` with `g4_passed: true`, so the SRS
  and SDD for `window-management` are owed too. Minting those identifiers is the owner's call
  at G2, so `fr:` and `satisfies:` stay empty until then.
- **`release: "0.4.0"` is a proposal.** `AGENTS.md` reserves the minor digit for the owner;
  nothing in this spec authorises the bump.
