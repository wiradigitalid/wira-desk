# 06: Aspect-ratio tiles and height-bounded row packing

**What to build:**
Replace the switcher's fixed-cell grid with Windows Alt+Tab's model — **uniform tile height, per-tile
width driven by the source window's aspect ratio** — and bound the row count by work-area height.

Split out of SPEC-14-04 part 2 after the owner supplied a screenshot of their own Alt+Tab. That
ticket's part 2 was "raise two constants and add a height bound"; this is a layout-engine
replacement, and it shares no code and no risk with SPEC-14-04's Shift matching.

**Read `DEC-026` → "Card geometry and DPI" first.** It carries the model, the measured numbers, the
clamp, and the reasoning. This ticket carries the work.

---

## 1. The model

| Quantity | Value | Note |
| --- | --- | --- |
| `CARD_H` | `212` logical | **Uniform.** The one fixed dimension |
| `HEADER_H` | `32` logical | Raised from `28` |
| Preview height | `172` logical | `CARD_H − HEADER_H − 8` |
| Preview width | `172 × aspect` | **Per card** |
| Card width | preview width `+ 8` | ≈119 … 421 logical under the clamp |
| Aspect clamp | `0.60 … 2.40` | Mandatory — see below |

All geometry — card, header, gutter, margin — becomes **logical units at 96 DPI**, scaled to physical
pixels by the origin monitor's effective DPI at layout time. `monitor_dpi`
(`arrangement/win32.rs:166`) already exists and the daemon is PerMonitorV2 aware
(`wiradesk.manifest`), so no new Win32 surface is needed.

**Scaling is not cosmetic; it repairs a live defect.** `overlay.rs:504-505` already scales the card's
icon and font by monitor DPI (`icon_size = (16·dpi + 48)/96`) while `HEADER_H` stays physical. The
icon computes to **28 px at 175% — exactly today's header height, zero margin — and 32 px at 200%,
where it overflows the header.** Raising `HEADER_H` to 32 does not fix that on its own; making the
geometry logical does, because header and contents then scale together.

**The clamp is mandatory.** An ultrawide maximized window is 3.56:1 and unclamped would produce a
612-logical tile that alone can exceed a row; a portrait window is ≈0.56:1 and would produce a tile
too narrow for its own title and icon. A clamped tile letterboxes inside its preview rect — that is
the correct, visible answer, and better than a tile that lies about its window's shape.

**Aspect source: `GetWindowRect` on the Worker at overlay-open time.** Do **not** add a rect to
`WindowFacts` (`cycling/mod.rs:99-113`); that drags `Win32CandidateSource`, the frozen fixtures, and
`ReferencePolicy` with it — the same contract change SPEC-14-03 part 2 is gated on, and there is no
reason to couple them. `layout.rs` stays pure by receiving aspect ratios as an argument.
`DwmQueryThumbnailSourceSize` is exact but needs the thumbnail registered first, inverting the
current compute-then-register order; reach for it only if letterboxing proves visible.

A window whose rect cannot be read, or whose height is zero, falls back to 16:9 rather than being
dropped or dividing by zero.

## 2. Row packing replaces `compute_grid`

`compute_grid` returns `(cols, rows, per_page)` and `compute_layout` derives every card position from
`col`/`row` arithmetic. With variable widths there is no `cols`.

- Fill a row while `running_width + card_width + GUTTER` fits the content width
  (`work_area.width − 2·MARGIN`), then wrap.
- `card_x` becomes a running accumulator; `card_y` still steps by `CARD_H + GUTTER`.
- **Page capacity is a result of packing, not an input.** `total_pages` cannot be
  `count.div_ceil(per_page)` any more — pack until the row budget is spent, and the remainder starts
  the next page.
- Row count is bounded by **work-area height**, which the current `compute_grid` never consults: it
  derives columns from width and then pins rows at `MAX_ROWS = 3`. `compute_layout` also centres the
  overlay (`overlay_y = work_area.y + (work_area.height − overlay_h) / 2`), so an over-tall overlay is
  clipped at **both** ends, not merely the bottom.
- Row count must never reach zero. What happens when not even one row fits is defined in code.

## 3. Spatial navigation has to become geometric — the largest hidden cost

`selection::select_up` computes `current_index − cols` (`selection.rs:51-52`) and `select_down`
mirrors it. `SwitcherLayout.cols` also feeds `overlay.rs:171` (`per_page`), `:174` (`cols()`), and
`:229`. With rows of differing card counts that arithmetic is simply wrong.

"The card above" becomes **the card in the previous row with the greatest horizontal overlap** with
the current card. Same rule downward.

**This is a behaviour change to SPEC-13-02, which the owner already accepted as delivered.** It is
not optional — leaving index arithmetic in place against packed rows means Up/Down jump to visibly
wrong cards — but it must be called out as touching shipped, signed-off work rather than slipped in.

## 4. Two tests invert rather than extend

- `columns_derive_from_the_work_area_and_never_overflow_it` (`layout.rs:139`) loses its subject.
  It becomes **"no packed row ever exceeds the content width"**, which is the invariant that actually
  matters. It currently sweeps four widths at a pinned height of `1080`, so it stays green through
  every overflow this ticket is meant to prevent; the matrix must carry short work areas too.
- `candidates_beyond_one_page_paginate_rather_than_shrink` (`layout.rs:171`) asserts
  `chrome_rect.width == CARD_W`. Width is no longer invariant — **height** is. The assertion inverts
  to `chrome_rect.height == scaled(CARD_H)`, and must not simply be deleted to make the change land.

**Honour SPEC-13-02's floor, re-read.** Its accepted criterion was that cards paginate rather than
shrink. The floor is now the uniform **height**: `212` logical, and a constrained work area
paginates rather than shrinking it.

**Blocked by:** None. Independent of SPEC-14-03 and of SPEC-14-04.

**Status:** ready-for-agent

- [ ] `CARD_H = 212`, `HEADER_H = 32`, and the unit is stated in code as logical-at-96-DPI, not left to the reader.
- [ ] Card, header, gutter, and margin geometry scales by the origin monitor's effective DPI: at 100% a 16:9 card is 314×212 physical, at 200% it is 628×424.
- [ ] Tile width follows the source window's aspect ratio; a 16:9 window yields a 306×172 preview and a 21:9 window a visibly wider tile than a 4:3 one, at identical height.
- [ ] Every card on a page has the **same height** and that height equals the scaled `CARD_H` — asserted, because it is the new invariant.
- [ ] Aspect is clamped to `0.60 … 2.40`; an ultrawide (3.56:1) and a portrait (0.56:1) window both produce in-band tiles and letterbox inside their preview rect.
- [ ] A window whose rect cannot be read, or whose height is zero, falls back to 16:9 — no panic, no divide-by-zero, no dropped card.
- [ ] Aspect ratios reach `layout.rs` as an argument; `layout.rs` calls no Win32 and `WindowFacts` gains no rect.
- [ ] No packed row exceeds the content width, swept across at least 1366, 1920, 2560, and 3840 wide **and** 768, 800, and 1080 tall, with mixed aspect ratios in the candidate set.
- [ ] Row count derives from work-area height; the overlay never exceeds the work area, and `overlay_y` is never above `work_area.y`.
- [ ] Row count is never zero, and the not-even-one-row-fits case is defined in code.
- [ ] `total_pages` comes from actual packing, not from `count.div_ceil(per_page)`; no page is ever empty.
- [ ] `select_up`/`select_down` pick the card in the adjacent row with the greatest horizontal overlap — asserted against a fixture whose rows hold different card counts, where index arithmetic would give a different and wrong answer.
- [ ] `columns_derive_from_the_work_area_and_never_overflow_it` is reformulated as the row-width invariant and was **observed red** against the unbounded packer before the bound was added. Record that in the commit message.
- [ ] `candidates_beyond_one_page_paginate_rather_than_shrink` asserts the scaled **height** and still passes; it was not deleted to make the scaling land.
- [ ] `monitor_dpi`'s doc-comment no longer claims that no planner scales by it.
