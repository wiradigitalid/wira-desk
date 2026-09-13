---
type: decision
id: DEC-026
status: accepted
touches:
  - .control/decisions/DEC-026-cross-monitor-switcher-shift-cycle-and-window-eligibility.md
  - .control/registry/decisions.yaml
  - .what/window-management/03-domain/domain-model.md
  - .scratch/spec-14-visual-switcher-refinements-and-settings-polish/SPEC.md
  - crates/daemon/src/hook.rs
  - crates/daemon/src/worker.rs
  - crates/daemon/src/context/mod.rs
  - crates/daemon/src/cycling/eligibility.rs
  - crates/daemon/src/switcher/mod.rs
  - crates/daemon/src/switcher/layout.rs
  - crates/shared/src/commands.rs
  - crates/shared/src/shortcut.rs
  - crates/settings/src/persistence.rs
  - crates/settings/ui/panes/general_pane.slint
supersedes: null
superseded_by: null
created: '2026-09-13'
accepted_by: kodesh87 (Product Owner, in session), 2026-09-13
---

# DEC-026 — Cross-Monitor Visual Switcher, Shift-Backward Cycling, and Window Eligibility Refinement

## Context

Following manual desktop evaluation of SPEC-12 and SPEC-13, three core behavioral questions arose regarding multi-monitor workflows, reverse navigation, and ghost/helper windows:

1. **Multi-Monitor Cycling vs Visual Switcher Scope:**
   The Spatial Preservation Invariant (FR-2, CAP-7, AD-3, AD-9, AD-14) bounds window cycling to the active physical monitor and active virtual desktop. The owner confirmed that while blind cycling must remain locked to the active monitor for quick muscle memory, the Visual Switcher overlay should provide a bird's-eye view across **all physical monitors** on the current virtual desktop.
2. **Reverse/Backward Navigation:**
   Windows multitasking (Alt+Tab) uses `Shift` to navigate in reverse. Users expect `Shift` combined with the cycle shortcut (e.g. `Alt+Shift+Backtick` or `Win+Shift+Backtick`) to navigate backwards.
3. **Helper / Ghost Windows:**
   Popup helper windows (such as `PopupHC` or zero-sized tooltips) occasionally appeared as cards. Window eligibility must filter these out consistently across both cycling and switcher.

## Decision

1. **Cross-Monitor Visual Switcher (Amending Spatial Preservation Invariant):**
   - **Blind Cycle (rapid tap):** Adheres strictly to the active physical monitor and current virtual desktop (`Spatial Preservation Lock`).
   - **Visual Switcher (hold):** Enumerates all candidate windows of the active application across **all physical monitors** on the current virtual desktop.
   - **Activation Behavior (Option A):** When a candidate from another monitor is chosen, focus shifts to that monitor and activates the target window in-place. The window is NOT moved or dragged to the active monitor.
2. **Shift as Universal Direction-Reversing Modifier:**
   - Holding `Shift` with the cycle shortcut reverses the cycle order (`Command::SwitcherPrev` or backward blind cycle).
   - No separate "Backward Cycle" shortcut input is added in Settings.
   - `Shift` is disallowed as a primary modifier in the cycle shortcut configuration (e.g., `ctrl+shift+backtick` is refused) to prevent chord collisions and ensure `Shift` remains strictly dedicated to direction reversal.
3. **Window Eligibility Sanitization:**
   - Both blind cycling and the visual switcher share unified eligibility rules: exclude zero-extent windows, empty titles, and helper/popup host surfaces lacking standard top-level application presence.
4. **Concise UI/UX Copywriting:**
   - Descriptions on the General pane and Shortcuts pane remain concise, clear, and to the point without verbose text.

## Why

- **Option A Activation:** Respects the user's multi-monitor layout without disrupting window placement across screens.
- **Dedicated Shift Reversal:** Matches universal Windows Alt+Tab ergonomics naturally without cluttering Settings UI.
- **Unified Filtering:** Prevents stray helper/popup cards from degrading visual switcher fidelity.

## Cost

- Amends the Spatial Preservation Invariant by adding an explicit exception for the visual switcher overlay.
- Secondary monitors must be queried during visual switcher candidate enumeration, adding minimal Win32 enum calls inside the hold window.

---

## Boundaries settled (peer review, 2026-09-13)

The four numbered clauses above state the owner's intent. A line-by-line read of the code they land
in found eight places where that intent does not survive contact unless a rule is written down. Each
is settled here rather than left to the implementing agent, because each has a silent failure mode.
Where a clause adds substance the owner has not seen, it says so.

### B-1 — Shift-relaxed matching is a **second pass**, never a relaxed first pass

Clause 2 forbids `Shift` inside the *cycle* chord. That does not, on its own, protect any *other*
action whose chord contains `Shift`. `Cycle` is first in `Chords::in_declared_order` (`hook.rs:822`)
and `match_shortcut` (`hook.rs:1023`) returns the first match, so a cycle chord matched with `Shift`
treated as "don't care" shadows every chord differing from it only by `Shift` — including the
shipped default stack chord `ctrl+alt+shift+s` (`shared/src/config.rs:435`) whenever cycle is bound
to `ctrl+alt+s`. `config::validate`'s duplicate detection (`daemon/src/config.rs:196-206`) compares
parsed `Shortcut` values, which differ, so it raises nothing. The collision is silent.

**The rule.** `match_shortcut` resolves in two passes, in this order:

1. **Exact pass** — the live `ModifierState` compared for equality against all sixteen slots, as
   today. Unchanged semantics, unchanged precedence.
2. **Shift-relaxed pass** — reached **only** when the exact pass returns `None`, and evaluated
   **only** against the two `Cycle` slots (`primary`, `fallback`), with `shift` cleared from the live
   state. A match here resolves to the backward cycle and to nothing else.

One-Chord-One-Action (BR-6, DEC-009) is preserved verbatim: no chord reaches two actions, because a
chord that reaches a configured action never gets to pass 2 at all.

### B-2 — The derived Shift variant is reservation-checked, and a collision refuses the base chord

DEC-003's `shared::shortcut::reservation` (`shortcut.rs:172`) is modifier-exact. Under B-1 the Shift
variant of the cycle chord becomes reachable without ever having been configured, so nothing checks
it. This is reachable, not theoretical: `Ctrl+Escape` is **not** in the catalogue and is a legal
cycle binding today, while `Ctrl+Shift+Escape` **is** catalogued as `Reservation::Immutable`, owner
*"Task Manager"*. Binding cycle to `ctrl+escape` would therefore swallow the user's Task Manager
escape hatch — exactly the product invariant DEC-003 exists to hold.

**The rule.** Validation rejects a cycle chord whose `Shift` variant carries a reservation, naming
that reservation's owner. Checked in `settings::persistence::validate_config` and in
`daemon::config::validate`'s reservation walk, both of which already iterate the declared order.

### B-3 — The `Shift` ban is enforced at save time; a legacy config is never rejected

`daemon::config::validate` refuses a reload **wholesale** — one bad field and every setting reverts.
A user who bound cycle to `ctrl+shift+backtick` before this decision did nothing wrong and must not
lose their whole configuration to a rule written after the fact.

**The rule.** The ban on `Shift` as a cycle-chord modifier is a **Settings save-time** rejection
(`validate_config`, where the field name is known and a human is present to be told). On daemon
reload a pre-existing Shift-carrying cycle chord is **accepted and matched exactly**, and simply
receives no shift-relaxed variant — there is no backward entry for it, because its `Shift` is already
spoken for. B-2's reservation refusal is the one exception and applies on both paths.

`settings::persistence::validate_shortcut` takes only the input string and cannot carry a
field-specific rule; the rule belongs in `validate_config`, which knows the field.

### B-4 — Backward blind cycle needs a command byte; direction MUST NOT travel in the modifier side-channel

Clause 2 promises "`Command::SwitcherPrev` **or backward blind cycle**". No backward blind cycle
exists: there is no `Command::CyclePrev`, both cycle slots map to `Command::Cycle`, and `cycle_order`
(`cycling/mod.rs:228`) produces one direction only.

The tempting implementation — let the Worker read direction from `SWITCHER_LAST_CYCLE_MODS`, the
`AtomicU8` the Hook already writes at `hook.rs:626` and the Worker already reads at `worker.rs:102` —
is **a race**, and this decision is what turns it from harmless into user-visible. The Hook writes
that atomic on *every* cycle while commands queue independently in the ring. A second chord press
arriving before the Worker drains the first overwrites the direction of a command already in flight:
the user taps the chord, then taps it with `Shift`, and the **first** tap cycles backward. Today the
same race only mis-arms a timer.

**The rule.** Direction is carried **in the ring**, as a distinct command byte (`Command::CyclePrev`),
decided by the Hook at match time and immutable thereafter. It is throttled exactly as
`Command::Cycle` is — it is **not** added to `is_exempt_from_throttle`. `cycle_order` grows a
direction parameter, and the backward order is the same rotation without the final `reverse()`.

### B-5 — `Shift` never joins `switcher_mods` or `SWITCHER_CHORD_MODS`

Clause 2 makes `Shift` a direction reverser, not a chord modifier. Two live sites capture the
modifier state wholesale and would silently make it one:

- `hook.rs:628` sets `rt.switcher_mods = rt.mods`. The overlay commits on
  `!rt.mods.has_any_of(&rt.switcher_mods)` (`hook.rs:446`), true only when **every** shared modifier
  is up. With `Shift` in the set, releasing `Alt` while still holding `Shift` leaves the overlay open
  with no chord modifier down, and the watchdog (`worker.rs:215`) will not rescue it — it commits only
  when *no* modifier at all is down, and `Shift` is one.
- `worker.rs:497` sets `SWITCHER_CHORD_MODS` from the same state, and `are_chord_modifiers_down`
  (`worker.rs:162`) requires **every** recorded modifier to still be physically down when the hold
  timer fires. With `Shift` recorded, a user who releases `Shift` a moment early — the natural motion
  — silently gets no overlay at all.

**The rule.** `Shift` is cleared from the state stored in both. Commit and hold-gate semantics are
decided by the non-Shift modifiers alone, and are therefore byte-for-byte what they are today.

### B-6 — Backward entry opens on the last card, and the blind hop before it must agree

`open_visual_switcher` calls `SWITCHER.open(origin, work_area, eligible_ordered, 0)` (`worker.rs:295`)
— the selected index is a hardcoded `0`.

**The rule.** A backward entry opens at `len - 1`. Because `card_order_for_candidates`
(`switcher/mod.rs:168`) *is* `cycle_order`, index `len - 1` is exactly the window a backward blind
cycle would have activated — the overlay and the blind path agree by construction rather than by
coincidence.

This holds only once B-4 lands. Until it does, a `Shift`-held chord runs a **forward** blind
activation and then opens an overlay claiming to be backward: the first thing the user sees is the
opposite of what they asked for. B-4 and B-6 ship together or neither ships.

### B-7 — Cross-monitor is a scope parameter, not a dropped check

`evaluate_spatial` (`context/mod.rs:76`) tests the monitor **first** and returns early, and it rejects
on `origin_monitor == None` with `MonitorUnavailable` before the virtual-desktop test is ever reached.
"Drop the physical-monitor half" is therefore not a deletion: done naively it also deletes the
origin-unavailable rejection for the switcher, or keeps it and rejects every candidate on a machine
where the origin monitor cannot be resolved.

**The rule.** `evaluate_spatial` takes an explicit scope — `SameMonitor` (blind cycle, today's
behaviour unchanged) or `AnyMonitorOnCurrentDesktop` (visual switcher). Under `AnyMonitorOnCurrentDesktop`
the origin-monitor lookup is not consulted at all and its failure is not an exclusion; the
virtual-desktop test (AD-9) runs unchanged and is the sole remaining spatial gate. The exclusion
precedence comment at `eligibility.rs:14-24` is a separate contract and is untouched — this is the
spatial gate, not the eligibility policy.

### B-8 — Option A's visible consequence: the next blind cycle follows the focus

Activating a candidate on another monitor makes that window the foreground, so the **next** blind
cycle captures its origin there and is locked to the **new** monitor. That is Option A behaving
correctly, and it is the single most likely thing to be reported back as a regression. It is stated
here so it is tested rather than re-litigated.

## Owner answers (2026-09-13, in session)

- **Backward *blind* cycle is wanted**, not only backward entry into the overlay. B-4 therefore lands
  at full scope: `Command::CyclePrev` as its own ring byte **and** a direction on `cycle_order`. A
  rapid Shift-held tap activates the previous same-app window on the active monitor; holding the same
  chord opens the overlay on the last card (B-6). SPEC-14-04 carries both.

  Two call sites key on `Command::Cycle` by value and are silent no-ops if they are not widened with
  it — neither is reachable from the acceptance criteria on its own:

  - `hook.rs:625` — `if cmd == Command::Cycle.as_u8()` guards `set_last_cycle_mods` **and the whole
    switcher-arming block**. Left as-is, a Shift-held chord cycles backward correctly and then never
    arms, so the overlay never opens at all and B-6 is unreachable.
  - `worker.rs:99` — the drain `match` arm that calls `execute_cycle`. Left as-is, `CyclePrev` falls
    through to no arm and the command is silently dropped.

### Card geometry and DPI — revised against the owner's own Alt+Tab (2026-09-13)

The owner's brief was that it should not look far off Windows' own Alt+Tab, and they then supplied a
screenshot of their Alt+Tab on a 4K panel at 150%. The screenshot overturned the first proposal's
central assumption and is the reason this section was rewritten rather than amended.

**Windows 11's Alt+Tab is not a uniform grid.** Tile *height* is constant within a row; tile *width*
varies per tile, following the source window's aspect ratio. In the owner's screenshot the row-one
tiles share one height while their widths run from a narrow Explorer window to a Snagit Capture
window more than three times wider. The owner named this unprompted, before any analysis: the tiles
are not square, and a wide window gets a correspondingly wide thumbnail. A fixed `CARD_W × CARD_H`
grid — which is what the switcher ships today and what the first proposal kept — cannot produce it.

**The height the owner likes is the height already proposed**, which is the one useful thing the
first proposal got right. Measuring off the screenshot: it is a 3840-wide capture presented at 2000,
so ≈1.92×; a tile reads ≈160 px in the image → ≈307 physical → at 150% → **≈205 logical**. The
proposed `CARD_H = 212` logical is within 3% of it. These are eyeball measurements off a downscaled
image, not instrumented ones, and are quoted as approximations.

#### The model

| Quantity | Value | Note |
| --- | --- | --- |
| `CARD_H` | `212` logical | **Uniform.** The one fixed dimension. Validated ≈205 against the owner's display |
| `HEADER_H` | `32` logical | Raised from `28`; the owner's Alt+Tab header reads ≈35 logical |
| Preview height | `172` logical | `CARD_H − HEADER_H − 8` |
| Preview width | `172 × aspect` | **Per card.** `aspect` = the source window's width ÷ height |
| Card width | preview width `+ 8` | Ranges ≈119 … 421 logical under the clamp below |
| Aspect clamp | `0.60 … 2.40` | See below |

All of these stay **logical units at 96 DPI**, scaled to physical by the origin monitor's effective
DPI. That half of the first proposal is unchanged and the reasoning behind it still holds — including
that it repairs a live defect, since `overlay.rs:504-505` already scales the card's icon and font by
DPI while `HEADER_H` stays physical, so the icon computes to 28 px at 175% (exactly the header
height) and 32 px at 200% (overflowing it). Raising `HEADER_H` to 32 logical does not by itself fix
that; scaling does.

A 16:9 window yields a `306 × 172` preview in a `314 × 212` card, so the common case lands almost
exactly where the first proposal's fixed `320 × 212` did. The difference is only visible on windows
that are *not* 16:9 — which is the whole point.

**The clamp is not optional.** An ultrawide maximized window is 3.56:1; unclamped it would produce a
612-logical tile that alone can exceed a row. A portrait window is around 0.56:1 and would produce a
tile too narrow to carry its own title text and icon. The `0.60 … 2.40` band brackets what the
owner's screenshot actually shows. A clamped tile letterboxes inside its preview rect, which is the
correct and visible answer — the alternative is a tile that lies about its window's shape.

**Where the aspect comes from.** `GetWindowRect` on the Worker at overlay-open time. This deliberately
does **not** add a rect to `WindowFacts` (`cycling/mod.rs:99-113`), which would drag
`Win32CandidateSource`, the frozen fixtures, and `ReferencePolicy` along with it — the same contract
change SPEC-14-03 part 2 is gated on, and there is no reason to couple the two. `layout.rs` stays
pure by taking the aspect ratios as an argument rather than querying anything.
`DwmQueryThumbnailSourceSize` would be exact rather than approximate, but it requires the thumbnail
registered first, which inverts the current compute-layout-then-register order. Use `GetWindowRect`;
reach for the DWM call only if letterboxing proves visible in practice.

#### What this costs — and it is not a constant change any more

The first proposal was two constants plus a height bound. This is **a layout-engine replacement**,
and it reaches further than `layout.rs`:

- `compute_grid` returns `(cols, rows, per_page)`. With variable widths there is no `cols`, and
  `per_page = cols × rows` is wrong. Row packing replaces it: fill a row until the next card would
  exceed the content width, then wrap. Page capacity becomes a *result* of packing, not an input.
- `card_x = overlay_x + MARGIN + col × (CARD_W + GUTTER)` becomes a running accumulator.
- `SwitcherLayout.cols` is consumed by `overlay.rs:171` (`per_page`), `:174` (`cols()`), and `:229`.
- **Spatial navigation, shipped and accepted under SPEC-13-02, breaks.** `selection::select_up`
  computes `current_index − cols` (`selection.rs:51-52`), and `select_down` mirrors it. With rows of
  differing card counts that arithmetic is simply wrong. "The card above" has to become geometric —
  the card in the previous row with the greatest horizontal overlap with the current one. This is a
  behaviour change to work the owner has already signed off, and it is the largest hidden cost here.
- `columns_derive_from_the_work_area_and_never_overflow_it` loses its subject. It becomes "no packed
  row ever exceeds the content width", which is the invariant that actually matters.
- `candidates_beyond_one_page_paginate_rather_than_shrink` asserts `chrome_rect.width == CARD_W`.
  Width is no longer invariant; **height** is. The assertion inverts.
- The height bound from the original part 2 is still required and is now more load-bearing, because
  row count no longer follows from a division.

Because of this the geometry work is split out of SPEC-14-04 into **SPEC-14-06**. SPEC-14-04 part 1
(Shift matching and direction) shares no code and no risk with it, and bundling a keyboard-matching
change with a layout-engine rewrite in one registry row is the same mistake the earlier review split
ticket 02 to avoid.

### Reserved Shift variant — refuse the base chord (owner, 2026-09-13)

Settled as §B-2 already states it: a cycle chord whose Shift variant is **reserved by Windows** is
refused, naming the reservation's owner. The lighter alternative — accept the chord and silently
suppress only its relaxed variant — is **not** taken. Binding cycle to `ctrl+escape` is therefore
refused, because `Ctrl+Shift+Escape` is Task Manager.

### Mouse gains no backward cycle (owner, 2026-09-13)

`MouseActionPreset` is unchanged. No `CycleBackward` slug, label, category entry, or Settings row.
`CycleForward`'s shipped label *"Cycle Same-App Window Forward"* (`shared/src/config.rs:285`) stays as
it is; it now reads as a deliberate scope line rather than a missing pair.

## Owner decisions still open

None blocks SPEC-14. Recorded so they are not discovered late.

- **Clause 3 still has no diagnosed subject.** It names "zero-extent windows, empty titles, and
  helper/popup host surfaces". `WindowFacts` (`cycling/mod.rs:99-113`) carries neither a rect nor a
  title, so all three are changes to the captured-facts contract, not filter rules. SPEC-14-03 part 2
  stays gated on `wdi-systematic-debugging`: this decision authorises the sanitisation, it does not
  identify what is being sanitised.
- **The corpus claims this decision invalidated are now APPLIED** (2026-09-13), not left as debt.
  The Spatial Preservation Invariant itself (`domain-model.md`) now names **two** deliberate
  crossings of the monitor half instead of one, and states that the virtual-desktop half has none;
  `SRS` Non-Goals no longer lists thumbnail HUDs as out of scope; CAP-1's out-of-scope line is
  scoped to blind cycling; and CAP-7's description carries the enumeration exception.

  **UC-1 was NOT retitled, and the earlier review calling its title "actively wrong" was mistaken.**
  UC-1 covers the blind cycle, which this decision leaves monitor-locked, so *"on this monitor"* is
  accurate. What it lacked was a sibling: it now carries a Scope section pointing at the visual
  switcher as a different path with a different monitor boundary and no use case of its own.

  What remains is authoring work rather than a decision: the visual switcher still has **no FR and no
  capability**, which is why `fr:` and every `satisfies:` in SPEC-14 are empty. That is the owner's
  at G2.
