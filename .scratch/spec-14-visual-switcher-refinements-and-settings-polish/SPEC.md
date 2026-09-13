---
spec: SPEC-14
release: "0.4.0"
prd: wira-desk
fr: []
status: open
---

# SPEC-14 — Visual Switcher Refinements, Cross-Monitor Candidates, Shift-Cycle Parity, and Settings Polish

## Problem Statement

Manual desktop testing of the v0.4.0 release build (`autopilot/DEC-025`, merged as PR #26) surfaced
several bugs, ergonomic gaps, and UI polish items across the About pane, the General settings pane,
and the Same-App Visual Switcher.

Every root cause below was read out of the code at the sha this spec was drafted against. Where the
cause is **not** established, it says so rather than guessing — three of the seven items below had a
plausible-sounding root cause in the first draft that the code contradicts.

---

### 1. About Pane Card 3 — publisher link is a separate row, and the dividers are inset

- Card 3 (`crates/settings/ui/panes/about_pane.slint:241`) carries a dedicated "Publisher website
  (wiradigital.id)" action row (`:280`) in addition to the attribution footer *"An open-source
  utility by Wira Digital Indonesia • Licensed under GPL-3.0"* (`:349`). The owner wants the two
  merged: the attribution line becomes the link surface, with only **Wira Digital Indonesia**
  carrying link colour and the `↗` icon, and the separate row removed.
- Card 3's dividers look inset next to the ones in the Updates card.

  **Root cause (verified).** Not the divider. `CardDivider`
  (`crates/settings/ui/components/card.slint:10`) is a bare 1 px `Rectangle` with no `x`/`width` of
  its own, and it is the *same component* in both cards. The difference is the parent: the Updates
  card's `VerticalLayout` sets `padding: 0px` (`about_pane.slint:116`) and pushes padding down into
  each row, so its divider spans the card; Card 3's `VerticalLayout` sets `padding: 16px`
  (`:244`), which insets every child including the divider by 16 px on each side.

  The first draft prescribed `x: 0; width: 100%` on the divider. That does not work — in Slint a
  layout owns the geometry of its direct children, so `x` and `width` set on a child of a
  `VerticalLayout` are overwritten by the layout pass. The fix is the padding restructure, not a
  geometry override.

### 2. General pane grows a horizontal scrollbar when the visual switcher toggle is on

Enabling "Enable Visual Switcher Overlay" reveals the hold-delay sub-row, and a horizontal
scrollbar appears across the General pane. Disabling it hides both.

**Root cause (verified).** `crates/settings/ui/panes/general_pane.slint:79-84` — the caption
*"Milliseconds to hold the chord before the visual overlay appears (100–500 ms)."* has no
`wrap: word-wrap`. Without it the `Text` reports its full single-line width as its minimum, the
enclosing `HorizontalLayout` (`alignment: space-between`) widens past the card, and the
`ScrollView` at `main_window.slint:334` grows a horizontal bar. Every other long caption in the
file already sets `wrap: word-wrap`; this one was missed.

### 3. Disabling the visual switcher does not stop it opening

After toggling "Enable Visual Switcher" off and saving, holding the cycle chord still opens the
overlay.

**Root cause (verified — and the first draft named the wrong one).**

- `WorkerSnapshot` (`crates/daemon/src/config.rs:42`) does **not** need the switcher fields for the
  gate to exist: the switcher configuration is already carried, on `HookSnapshot`
  (`config.rs:32-33`, populated at `:223-224`) and already landed into the Hook runtime at
  `hook.rs:1881-1882`.
- The Hook **already gates on it**: `hook.rs:627` refuses to set `rt.switcher_armed` when
  `switcher_visual_enabled` is false, and `hook::tests::reload_with_visual_switcher_disabled_leaves_blind_cycling_unchanged`
  (`hook.rs:2597`) asserts exactly that, with the comment *"Switcher must NEVER arm when
  visual_enabled is false!"*.
- **That gate is dead.** `rt.switcher_armed` feeds only `HookRuntime::check_switcher_deadline`
  (`hook.rs:1349`), which is marked `#[allow(dead_code)]` and has exactly one caller in the whole
  workspace — a test at `hook.rs:2676`. Nothing in production ever calls it.
- The overlay actually opens down a **second, ungated path**: `Command::Cycle` reaches the Worker
  (`worker.rs:102`), and `execute_cycle` (`worker.rs:466`) arms `TIMER_SWITCHER_HOLD` on
  `CycleOutcome::Activated` whenever any modifier is held (`worker.rs:492-506`), with a hardcoded
  `150` ms and **no** `visual_enabled` check. `handle_timer` (`worker.rs:206`) then calls
  `open_visual_switcher`.

So the live arming path is in the Worker, the gate and its test are in the Hook, and the two have
never met. The existing test is green against a code path that cannot open anything — this is the
"a test never seen red is a claim, not proof" case `CLAUDE.md` names, and any fix that only
strengthens the Hook-side assertion will ship the bug again with a green suite.

Consequence for the fix: the Worker needs the gate, and `worker_snapshot()`'s cold-start fallback
(`worker.rs:805-817`, which rebuilds a snapshot from `Config::load_or_default` when no reload has
arrived yet) needs the fields too, or a daemon that has never reloaded will default to "enabled"
regardless of what is on disk.

### 4. The visual switcher shows only the active monitor's windows; the owner wants all monitors

With four windows of one application open — one on the left monitor, three on the right — the
overlay shows two. The blind cycle behaves as designed; the owner wants the **overlay** to be
comprehensive across physical monitors while the blind cycle stays monitor-locked.

**This is not a bug. It is a deliberate corpus invariant the owner asked to change**, and
`DEC-026` (accepted 2026-09-13) is the record of that change. The invariant it amends:

> **Spatial Preservation Invariant:** Target candidate windows for cycling or snapping must reside
> on the exact same physical monitor and virtual desktop as the foreground window (FR-2, CAP-7). A
> monitor-move command is the one deliberate crossing of the monitor half of this boundary […]
> — `.what/window-management/03-domain/domain-model.md:55`

CAP-7 "Spatial Layout Preservation" (`.what/_prd/wira-desk/prd.md:112`) realizes FR-2 and is bound
by AD-3, AD-9, and AD-14 (`.how/_platform/ARCHITECTURE-SPINE.md:217`). UC-1 is titled *"Cycle to
the next window of the same app **on this monitor**"*. The invariant names its exceptions
explicitly and lists exactly one. SPEC-14 introduces a second.

`CLAUDE.md` makes a `DEC-` mandatory for exactly this case ("One case is mandatory — contradicting
an `AD-N`"), and `DEC-026` clause 1 supplies it: the blind cycle keeps the lock, the visual switcher
crosses monitors, and a cross-monitor candidate is activated **in place** (Option A) rather than moved.

`DEC-026` §B-7 settles the one thing the decision's prose leaves open. `evaluate_spatial`
(`context/mod.rs:76`) tests the monitor **first** and returns early, rejecting on
`origin_monitor == None` before the virtual-desktop test is reached — so "drop the physical-monitor
half" is a scope parameter, not a deleted branch. Done naively it either deletes the
origin-unavailable rejection for the switcher, or keeps it and rejects every candidate on a machine
where the origin monitor cannot be resolved.

There is also a live code-level invariant to settle. `switcher::tests::switcher_candidate_set_equals_blind_cycle_eligible_set`
(`crates/daemon/src/switcher/mod.rs:182`) is SPEC-13-01's named guard for "the switcher shows what
the blind cycle would have reached". SPEC-14-03 deliberately breaks that equality — and the test
**will not notice**, because as written it calls `collect_eligible_candidates` twice with
byte-identical arguments and asserts the two results match. It is a tautology: it would stay green
if the production switcher path were deleted. It has to be either retargeted at the real
`open_visual_switcher` collection or retired against the DEC that supersedes it.

### 5. Extraneous window cards in the switcher ("PopupHC" or similar)

Cards appeared with titles the owner did not recognise, recalled as "PopupHC" or similar.

**Root cause: NOT ESTABLISHED.** The first draft asserted that "`WindowEligibility` candidate
collection lacks strict filtering for invisible, tool, or popup host windows". The code says
otherwise — `evaluate_facts` (`crates/daemon/src/cycling/eligibility.rs:39`) already excludes, in a
frozen precedence order, ghost classes, shell surfaces, `!visible`, cloaked, iconic, `tool_window`
(`WS_EX_TOOLWINDOW`), unavailable identity, and different application. Whatever these windows were,
they passed all eight of those rules: they were visible, uncloaked, non-minimized, not tool
windows, and carried the same executable identity as the foreground app.

That means the class name was not captured, and neither was the application. Guessing a filter from
a half-remembered title is how a legitimate window gets silently dropped from the switcher — a far
worse defect than the one being fixed, and an invisible one.

Per `CLAUDE.md` ("A bug, a failing test, or unexpected behaviour → skill `wdi-systematic-debugging`,
**before** any fix is proposed"), this item is **blocked on diagnosis**, not ready for
implementation. SPEC-14-03 carries the diagnosis step as its first, gating deliverable.

Note also that this may not be a defect at all once cross-monitor collection (item 4) lands: the
overlay will show strictly more windows than before, and some of those may be legitimate windows
the owner simply has not seen listed.

### 6. No Shift-held backward navigation into the switcher

In Windows Alt+Tab, holding Shift reverses the order. The owner wants `Shift` + the cycle chord to
enter the switcher moving backwards.

**Partly already built. The missing half needed a contract change, and `DEC-026` clause 2 supplies
it** — Shift is a universal direction reverser, and is refused as a modifier inside the cycle chord
itself.

- **Already works:** once the overlay is open, re-pressing the main key with Shift held routes to
  `Command::SwitcherPrev` (`hook.rs:519-526`). Arrow-key `Left`/`Right` navigation is there too
  (`:517-518`).
- **Does not work, and cannot without a decision:** *entering* backward. `match_shortcut`
  (`hook.rs:1023`) compares the live `ModifierState` to the configured `Shortcut` by **exact
  equality on all four modifier bits**. With cycle bound to `win+backtick`, pressing
  `Win+Shift+Backtick` builds `Shortcut { win: true, shift: true, vk: BACKTICK }`, which equals no
  configured chord, so `match_shortcut` returns `None`, the key passes through to the foreground
  app, and no cycle, no arming, and no overlay happen at all.

Making Shift a blanket "don't care" for the cycle chord is still wrong, and clause 2's ban on Shift
*inside* the cycle chord does not on its own prevent the collision — the collision is with **other**
actions' chords, not with the cycle chord's own. `DEC-026` §B-1 resolves it with a two-pass match:
exact over all sixteen slots first, and a shift-relaxed pass over the two `Cycle` slots only when the
exact pass returns `None`. A chord that reaches a configured action never reaches pass 2, so BR-6 /
DEC-009 holds verbatim.

The two invariants that forced that shape:

- **One Chord, One Action Invariant** (BR-6, DEC-009, `domain-model.md`): "No two actions may be
  reachable by the same chord." `Cycle` is **first** in `Chords::in_declared_order`
  (`hook.rs:822-831`), and `match_shortcut` returns the first match — so a Shift-insensitive cycle
  would shadow *every* other action whose chord differs from the cycle chord only by Shift. Concrete
  failure: bind cycle to `ctrl+alt+s` and the default stack chord `ctrl+alt+shift+s`
  (`crates/shared/src/config.rs:435`) becomes unreachable. DEC-009's duplicate detection
  (`config.rs:196-206`) compares parsed `Shortcut` values, so the two differ and it raises nothing —
  the collision is silent.
- **Reserved-chord refusal** (DEC-003). `shared::shortcut::reservation`
  (`crates/shared/src/shortcut.rs:172`) is itself modifier-exact, and treats `Alt+Shift+Tab` as a
  *separate* reservation from `Alt+Tab` (`:200-206`) precisely because the codebase already regards
  a Shift variant as a distinct chord. A Shift-insensitive cycle would claim a Shift variant that
  was never reservation-checked.

Three further hazards the decision's prose does not reach, each settled in `DEC-026` §B-2 through
§B-6 and carried into SPEC-14-04:

- **The derived Shift variant is never reservation-checked** (§B-2). `reservation`
  (`shortcut.rs:172`) is modifier-exact. `Ctrl+Escape` is **not** catalogued and is a legal cycle
  binding today; `Ctrl+Shift+Escape` **is**, as `Reservation::Immutable`, owner *"Task Manager"*.
  Binding cycle to `ctrl+escape` would swallow the user's Task Manager escape hatch — the exact
  product invariant DEC-003 exists to hold. Reachable, not theoretical.
- **A legacy config must not be rejected wholesale** (§B-3). `daemon::config::validate` refuses a
  reload entirely on one bad field. A user who bound cycle to `ctrl+shift+backtick` before this
  decision would lose every other setting to a rule written after the fact. The ban is a Settings
  save-time rejection; on reload such a chord is accepted, matched exactly, and simply gets no
  relaxed variant.
- **Direction must not travel in the modifier side-channel** (§B-4). There is no
  `Command::CyclePrev`; both cycle slots map to `Command::Cycle` and `cycle_order`
  (`cycling/mod.rs:228`) has one direction. Reading direction from `SWITCHER_LAST_CYCLE_MODS` — the
  `AtomicU8` the Hook writes at `hook.rs:626` and the Worker reads at `worker.rs:102` — is a race the
  Hook loses: it overwrites that atomic on every cycle while commands queue independently in the ring,
  so a chord press arriving before the Worker drains the previous one **reverses a command already in
  flight**. Today that race only mis-arms a timer, which is why nothing has caught it. Direction
  belongs in the ring as its own command byte.

Also unresolved by the decision's prose, and settled in §B-5: modifier-release semantics. The overlay
commits when
`!rt.mods.has_any_of(&rt.switcher_mods)` (`hook.rs:446`), and `has_any_of`
(`hook.rs:755-760`) is true while **any** shared modifier is still down. If Shift joins
`switcher_mods`, releasing Alt while still holding Shift leaves the overlay open with no Alt held —
a state the watchdog (`worker.rs:215`) will not commit either, because it only commits when *no*
modifier at all is down. The mirror case is just as bad and less obvious: `SWITCHER_CHORD_MODS`
(`worker.rs:497`) feeds `are_chord_modifiers_down` (`worker.rs:162`), which requires **every**
recorded modifier to still be physically down when the hold timer fires — so with Shift recorded, a
user who releases Shift a moment early gets no overlay at all, silently. §B-5 settles it: Shift is
cleared from both sets, and commit and hold-gate semantics stay byte-for-byte what they are today.

One consequence nobody has stated, settled in §B-6: `open_visual_switcher` calls
`SWITCHER.open(..., 0)` (`worker.rs:295`) with a hardcoded selected index. Backward entry opens at
`len - 1`, and because `card_order_for_candidates` (`switcher/mod.rs:168`) *is* `cycle_order`, that
index is exactly the window a backward blind cycle would have activated. Until the `CyclePrev` byte
lands, a Shift-held chord runs a **forward** blind activation and then opens an overlay claiming to
be backward — the user sees the opposite of what they asked for. The two ship together.

### 7. Cards and thumbnails are too small — and the wrong shape

**Measured, correcting the first draft.** `crates/daemon/src/switcher/layout.rs` has
`CARD_W = 240`, `CARD_H = 180`, `HEADER_H = 28`. The preview rect is therefore
`CARD_W - 8` × `CARD_H - HEADER_H - 8` = **232 × 144 px**, not the "~224 × 124 px after a 40 px
chrome header" the first draft claimed.

The enlargement is straightforward. The **constraint it breaks is not.** `compute_grid`
(`layout.rs:46`) derives columns from the work-area *width* and then fixes rows at
`MAX_ROWS = 3` — it never consults work-area *height*. At 240×180 a three-row overlay is
`3·180 + 2·16 + 2·24 + 24` = **644 px** tall, which fits a 1366×768 laptop work area (~728 px). At
320×240 it becomes `3·240 + 2·16 + 2·24 + 24` = **824 px**, which does not. The existing guard
`columns_derive_from_the_work_area_and_never_overflow_it` (`layout.rs:139`) sweeps four widths but
pins height at `1080` for all of them, so it will stay green while the overlay runs off the bottom
of every 768-tall and 800-tall display.

### 8. Two owner questions that are not defects, and are not in scope here

The raw notes contain two items that the first draft dropped without trace. Neither is a ticket;
both are recorded so they are not lost:

- **"Is the red tray icon a problem, how do I clear it, is it realtime?"** — a question about
  existing behaviour (`crates/daemon/src/tray.rs`, `health.rs`), not a reported defect. Answer it
  directly; open a ticket only if the answer turns out to be "the icon is wrong".
- **"Memory is 3.6 MB runtime — can it be optimised further?"** — **answered by `DEC-027`**
  (owner, 2026-09-13). It was the third reading: an SRS figure the visual switcher had legitimately
  outgrown. The budget is now 5 MB of private bytes, idle, hard ceiling unchanged at 10 MB. Note that
  the question as posed conflates two numbers — `DEF-11` measured 29.7 MB working set against 4.0 MB
  private on the same process — which is why `DEC-027` fixes the metric and not only the figure.
  Still no ticket in this spec: nothing measures it, and making it measurable is unscoped.

---

## Solution Architecture

### `SPEC-14-01` — About pane Card 3: inline publisher link, full-bleed dividers

Remove the standalone publisher row. Make the attribution line the link surface, with only
"Wira Digital Indonesia" carrying link colour, hover feedback, pointer cursor, the `OpenLinkIcon`,
and `open_publisher_url()`. Restructure Card 3's `VerticalLayout` to `padding: 0px` with per-section
padding, matching the Updates card, so `CardDivider` spans the card.

**Constraint.** Slint cannot style a substring of a single `Text`, so the attribution line becomes a
run of three elements. A plain `HorizontalLayout` of three `Text`s **cannot wrap**, which
reintroduces exactly the horizontal-overflow defect `SPEC-14-02` is fixing — at the About pane's
narrowest width the line is ~70 characters. The layout must remain wrappable (a wrapping container,
or a width constraint that guarantees the three-part run fits at the pane's minimum width).

### `SPEC-14-02` — General pane: wrap the hold-delay caption

Add `wrap: word-wrap` to `general_pane.slint:80`. Settings-only; no daemon change.

### `SPEC-14-05` — Worker honours `visual_enabled` and `visual_hold_delay_ms`

Split out of the first draft's ticket 02, which bundled a Slint text-wrap fix with a daemon timer
gate — no shared code, no shared risk, and two different Product Components in one registry row.

Carry the switcher settings to the Worker and gate the **live** arming path:

- Extend `WorkerSnapshot` (`config.rs:42`) with `visual_enabled: bool` and
  `visual_hold_delay_ms: u32`, populate them at `config.rs:226`, and **update the struct's
  doc-comment**, which currently states the Worker reads layout only and explains why. A stale
  justification left in place is worse than none.
- Extend the cold-start fallback in `worker_snapshot()` (`worker.rs:810`) with both fields, or a
  daemon that has not yet taken a reload defaults to enabled.
- In `execute_cycle` (`worker.rs:492-506`): skip arming and skip storing the origin when
  `visual_enabled` is false; use `visual_hold_delay_ms` otherwise.
- Extract the arming decision into a pure function so it can be tested without Win32 — `execute_cycle`
  calls `SetTimer` directly and has no seam today.
- Resolve the dead Hook-side gate: either wire `check_switcher_deadline` into production or delete
  it with its `#[allow(dead_code)]`, and retarget
  `reload_with_visual_switcher_disabled_leaves_blind_cycling_unchanged` at whichever path is live.
  Leaving both is how the next reader concludes the gate exists.

### `SPEC-14-03` — Cross-monitor switcher candidates, and whatever "PopupHC" is

Two deliverables, the second gated on diagnosis.

1. **Cross-monitor collection, under `DEC-026` clause 1.** `open_visual_switcher` (`worker.rs:235`)
   collects under `SpatialScope::AnyMonitorOnCurrentDesktop` while `run_context_safe_cycle`
   (`worker.rs:578`) stays on `SpatialScope::SameMonitor`. The **virtual-desktop** half (AD-9) is
   untouched on both paths. The overlay is still placed on the origin window's work area; only the
   candidate set widens. Activation is in place — no `SetWindowPos` on the commit path — and the
   *next* blind cycle is consequently locked to the newly focused monitor (§B-8), which is the
   behaviour most likely to be misreported as a regression.
2. **Helper-window filtering, only after diagnosis.** Capture the real class name, executable, and
   style bits of the offending windows first. Only then decide whether a new exclusion rule is
   warranted, and where it sits in the frozen precedence order.

### `SPEC-14-04` — Shift-backward cycling

1. **Shift-held backward cycling, under `DEC-026` clause 2.** A two-pass match (exact over all
   sixteen slots, then shift-relaxed over the two `Cycle` slots only), a `Command::CyclePrev` byte so
   direction rides the ring instead of the modifier atomic, `shift` cleared from both chord-modifier
   sets, backward entry opening at `len - 1`, a save-time refusal of `shift` inside the cycle chord,
   and a reservation check on the derived Shift variant. §B-1 through §B-6 carry the detail. The
   in-overlay Shift reversal already works and needs no change.
The card-geometry half that used to sit here is now `SPEC-14-06`.

### `SPEC-14-06` — Aspect-ratio tiles and height-bounded row packing

The owner's own Alt+Tab screenshot (4K at 150%) settled the shape: **uniform tile height, per-tile
width driven by the source window's aspect ratio**, which a fixed `CARD_W × CARD_H` grid cannot
produce. `CARD_H = 212` logical (validated ≈205 against that display), `HEADER_H = 32`, preview width
`172 × aspect` clamped to `0.60 … 2.40`, all of it logical-at-96-DPI and scaled by the origin
monitor.

This is a layout-engine replacement rather than a constant change. `compute_grid`'s
`(cols, rows, per_page)` gives way to row packing; page capacity becomes a result of packing rather
than an input; and `selection::select_up`/`select_down`, which compute `index ± cols`
(`selection.rs:51-52`), must become geometric — the adjacent-row card with the greatest horizontal
overlap. That last one is a behaviour change to SPEC-13-02, already accepted as delivered, and is the
largest hidden cost in SPEC-14.

---

## Decisions taken

Both decisions this spec owed are now carried by a single record, `DEC-026`
(`.control/decisions/DEC-026-cross-monitor-switcher-shift-cycle-and-window-eligibility.md`), accepted
by the owner on 2026-09-13. Both are the mandatory kind: they contradict a standing invariant.

| Decision | Contradicts | Gated | Status |
| --- | --- | --- | --- |
| The visual switcher crosses physical monitors; the blind cycle does not; a cross-monitor candidate activates in place | Spatial Preservation Invariant (FR-2, CAP-7, AD-3/AD-9/AD-14); UC-1's title and scope; CAP-1's "Out of Scope" line | `SPEC-14-03` | `DEC-026` clause 1 — **accepted** |
| Shift is a universal direction reverser, and is refused as a modifier inside the cycle chord | One Chord, One Action Invariant (BR-6, DEC-009); reserved-chord refusal (DEC-003) | `SPEC-14-04` | `DEC-026` clause 2 — **accepted** |
| Window eligibility excludes zero-extent, empty-titled, and helper/popup host surfaces | Nothing; it tightens an existing policy | `SPEC-14-03` part 2 | `DEC-026` clause 3 — **accepted, subject undiagnosed** |

`DEC-026`'s four clauses state intent; its **Boundaries settled** section (§B-1 … §B-8) is what makes
them buildable. Eight code sites turn the intent into a silent failure unless a rule is written down,
and five of the eight have no existing test that would catch the failure. Read that section before
implementing either ticket — the tickets reference it by sub-clause rather than restating it.

Clause 3 is accepted but has **no diagnosed subject**. It names zero-extent windows, empty titles, and
helper/popup host surfaces; `WindowFacts` (`cycling/mod.rs:99-113`) carries neither a rect nor a
title, so all three are changes to the captured-facts contract rather than filter rules. SPEC-14-03
part 2 therefore stays gated on `wdi-systematic-debugging`, and may land in a later spec without
holding part 1. The decision authorises the sanitisation; it does not say what is being sanitised.

## Corpus debt carried, not discharged

`fr: []` and every ticket's `satisfies: []` are empty for the same reason SPEC-13's registry note
gives: the visual switcher has no requirement in the corpus yet, and claiming FR-1/FR-2/UC-1 would
file this work as the blind cycle, which it is not. SPEC-14 **adds** to that debt rather than paying
it down, and two of its additions are now load-bearing staleness rather than mere lag:

- `.what/window-management/SRS-window-management.md:53` still lists "visual switcher HUDs, thumbnail
  previews, or overlay window task bars" as **out of scope**.
- The RAM claim is **discharged**: `DEC-027` (owner, 2026-09-13) raised the budget to 5 MB and fixed
  the metric as private bytes on a release build, idle. Eight corpus sites moved with it. `DEF-11`
  is retargeted rather than closed — the measurement on a fresh idle release build is still owed, and
  nothing in the workspace asserts any memory ceiling.
- `.what/_prd/wira-desk/prd.md:100-106` (CAP-1) still lists thumbnail previews and switcher HUDs as
  out of scope.
- UC-1 is still titled and scoped "on this monitor".

A reader repairing the switcher today would be steered wrong by all three. They are the owner's call
at G2; this spec records them so the call is made with the list in hand.

## User Stories & Tickets

- `SPEC-14-01`: About pane Card 3 inline publisher link and full-bleed dividers.
- `SPEC-14-02`: General pane hold-delay caption wraps.
- `SPEC-14-03`: Visual switcher cross-monitor candidates (ready), and diagnosis of the helper-window
  cards (still gated on `wdi-systematic-debugging`, and severable from part 1).
- `SPEC-14-04`: Visual switcher Shift-held backward cycling.
- `SPEC-14-05`: Worker honours `visual_enabled` and `visual_hold_delay_ms`.
- `SPEC-14-06`: Aspect-ratio tiles and height-bounded row packing.
