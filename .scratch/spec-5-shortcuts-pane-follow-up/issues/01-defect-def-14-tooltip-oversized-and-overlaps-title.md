---
id: SPEC-5-01
component: settings
satisfies: [UC-4]
blocked_by: []
status: done
tests:
  - shortcut_row_slint_snapshot::tests::tooltip_does_not_overlap_the_row_title
  - shortcut_row_slint_snapshot::tests::tooltip_height_fits_its_own_text
---

# 01: Defect DEF-14 — hover tooltip renders oversized and overlaps the row's title

**What to build:** Fix `DEF-14` (`.control/registry/defects.yaml`). `SPEC-4-03` moved a
shortcut row's description from a permanently visible line to a hover/focus tooltip, and the tooltip does
appear on hover. The owner reports its rendered shape is much taller than the single line of text it holds,
and because the shape is near-transparent with only a thin outline, that extra height reads as a faint box
overlapping the row's own title rather than a normal tooltip sitting beside or below the hovered control
without touching neighbouring text. Read `DEF-14` in full before starting; the owner's screenshot (attached
to the finding this ticket was opened from) shows the "Fallback switch shortcut" row's tooltip visibly
extending up into and past the row's title line.

**Blocked by:** None (can start immediately)

- [x] **Root-cause the current tooltip's size and placement in `crates/settings/ui/components/shortcut_row.slint`
      before changing anything.** Establish what currently sizes the tooltip's container — a fixed height on
      the element carrying the description, a default from whatever tooltip mechanism Slint 1.17 provides, or
      something else — rather than assuming a specific cause and patching blind.
- [x] The tooltip's rendered height fits its own text (one line, for the descriptions this pane currently
      ships) rather than a taller default box. A new snapshot test asserts the tooltip element's rendered
      height is close to its text's natural line height, not a fixed larger constant.
- [x] The tooltip no longer overlaps the row's title when shown on hover or keyboard focus. A new snapshot
      test asserts the tooltip's rendered bounds do not intersect the title `Text` element's bounds for at
      least one row exercised in the existing tooltip test
      (`description_renders_as_a_tooltip_not_a_visible_line`'s row is reasonable prior art to reuse).
- [x] Positioning follows how a tooltip conventionally behaves elsewhere — appearing beside or just below the
      hovered control, close-fitted to its own text — rather than a specific pixel offset the owner
      prescribed. The acceptance bar is "does not overlap the title" and "closely fits its own text," not a
      named position; use your own judgement for the concrete placement.
- [x] `SPEC-4-03`'s own tooltip guard
      (`shortcut_row_slint_snapshot::tests::description_renders_as_a_tooltip_not_a_visible_line`) stays green
      and unweakened — this ticket changes the tooltip's size and position, not whether it exists as a
      tooltip rather than a permanently visible line.
- [x] The "fits its own text" check is exercised against the LONGEST description string this pane currently
      ships, not only a short one-line example — a fix that fits a short description while still misfitting
      or clipping the longest one is not done. (Edge-case-hunter finding, resolved in this pass.)
- [x] A row near the bottom of the pane's visible area is explicitly considered: if the chosen tooltip
      placement can open downward past what is visible without scrolling, either constrain it to open upward
      when there is no room below, or say plainly in the closing report that this case was checked and is out
      of reach for this ticket's own test harness (a real viewport clip is not something a Slint snapshot test
      can see). Do not leave it unconsidered. (Edge-case-hunter finding, resolved in this pass.)
      *Note on bottom-of-pane row:* The tooltip placement opens downward (`y: root.height + 2px`) fitting its
      compact 22px single-line height. For rows at the bottom of the scroll viewport, whether the downward box
      clips against the Flickable viewport edge is governed by Slint's scroll container clipping, which is
      out of reach of the headless TestingBackend snapshot harness. Stated plainly and recorded as a residual
      item on DEF-14.
- [x] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- `DEF-15` (row container height changing with toggle state) — a different element of the same row, its own
  ticket (`SPEC-5-02`), sequenced right after this one because both touch the same markup.
- `DEF-13` (percentage field reverting mid-keystroke) — unrelated control on the same row, `SPEC-5-03`.
