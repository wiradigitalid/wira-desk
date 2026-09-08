---
id: SPEC-6-01
component: settings
satisfies: [UC-4]
blocked_by: []
status: open
tests:
  - shortcut_row_slint_snapshot::tests::tooltip_paints_above_the_next_row_when_it_overflows_into_it
  - shortcuts_pane_slint_snapshot::tests::the_last_rows_tooltip_paints_above_the_key_check_panel
---

# 01: Defect DEF-16 — hover tooltip paints behind a later sibling instead of above it

**What to build:** Fix `DEF-16` (`.control/registry/defects.yaml`). `SPEC-5-01` fixed `DEF-14`'s
oversized, near-transparent tooltip; the owner's retest of that fix found the resized tooltip has
a second, unrelated fault: on the pane's last row (Overlapping Stack), the tooltip renders behind
the always-visible Key Check panel below it, and on an ordinary row, the tooltip renders behind
the NEXT row's own -/+ stepper, percentage field, and keycap button. Read `DEF-16` in full before
starting — the root cause is already established there, not left for this ticket to find.

**Blocked by:** None (can start immediately; independent of `SPEC-6-02`, which touches a different
control on the same row markup)

- [ ] **Confirm the established root cause before changing anything**, per this repo's standing
      rule to verify rather than assume: the tooltip Rectangle in
      `crates/settings/ui/components/shortcut_row.slint` (currently around line 420) sets
      `z: 100`, but `z` only orders an element among its own parent's children — here, the
      `ShortcutRow`'s own children. It does not elevate the tooltip above a DIFFERENT
      `ShortcutRow` instance (the next row in `shortcuts_pane.slint`'s list) or above `KeyCheck`
      (`main_window.slint`), both painted later in the outer tree's document order regardless of
      this row's own `z`.
- [ ] Root-cause whether Slint's `PopupWindow` (a component built for exactly this shape — anchored
      to a parent element, rendered in its own top-level layer) actually paints above a sibling
      `ShortcutRow` and above `KeyCheck` in this window's structure, before committing to it as the
      fix. `DEF-16`'s `fix_direction` names it as worth checking, not as a decided design — this
      repo's standing rule against patching blind on an unverified cause applies to the FIX
      mechanism here, not only to root-causing the symptom.
- [ ] Whatever the mechanism, the tooltip renders as the topmost element at its own screen region
      when it overflows into a later sibling — both cases the owner reproduced: a middle row's
      tooltip over the next row's controls, and the last row's tooltip over the Key Check panel.
- [ ] A new test renders at least two adjacent rows (or a row plus `KeyCheck`) together and asserts
      the tooltip's bounds are the topmost element at its own pixel region — not merely that the
      tooltip does not overlap its OWN row's title, which is what `SPEC-5-01`'s two guards already
      check and is not the gap this ticket closes.
- [ ] `SPEC-5-01`'s existing tooltip guards
      (`tooltip_does_not_overlap_the_row_title`, `tooltip_height_fits_its_own_text`) stay green and
      unweakened — this ticket changes paint order, not the tooltip's size or its own-row
      positioning.
- [ ] While fixing paint order, the builder MAY also make the tooltip's positioning cursor-aware
      (or otherwise adaptive — flipping to open upward or sideways near a screen edge) rather than
      always anchoring to the row's bottom edge, since it is the same underlying "fixed-direction,
      anchor-based" tooltip design that produces `DEF-16` in the first place. This is a judgement
      call, not a required acceptance criterion — the required bar for this ticket is paint order,
      not distance from the cursor — but is worth doing in the same pass if the paint-order fix
      already changes how the tooltip is anchored.
- [ ] Full test suite green once, not only this ticket's own tests.

## Out of scope, deliberately

- `DEF-17` (stepper buttons freezing on an out-of-range typed value) — a different control on the
  same row, `SPEC-6-02`.
