# 01: About pane Card 3 inline publisher link and full-bleed dividers

**What to build:**
Two changes in `crates/settings/ui/panes/about_pane.slint`, both inside Card 3 (`:241`).

1. **Inline the publisher link.** Remove the standalone "Publisher website (wiradigital.id)" row
   (`:280-308`) and its `CardDivider`. Rebuild the attribution line (`:349`) as three parts:
   - plain caption: `An open-source utility by `
   - link: `Wira Digital Indonesia` + inline `OpenLinkIcon` (`↗`), `Palette.text_primary` at rest,
     `Palette.accent_hover` on hover, pointer cursor, `accessible-role: button`,
     `accessible-label: "Publisher website (wiradigital.id)"`, `clicked => { root.open_publisher_url(); }`
   - plain caption: ` • Licensed under GPL-3.0`

   Keep the existing colour split: the two plain parts stay `Palette.text_secondary`, only the link
   part is brighter. `root.open_publisher_url()` and its Rust binding are unchanged — this ticket
   moves the call site, it does not add a callback.

2. **Make Card 3's dividers full-bleed.** The cause is the parent, not the divider: Card 3's
   `VerticalLayout` sets `padding: 16px` (`:244`), which insets every child. `CardDivider`
   (`components/card.slint:10`) is a bare 1 px `Rectangle` and is already the same component the
   Updates card uses. Restructure Card 3 to `padding: 0px` with per-section padding, the way the
   Updates card does it (`about_pane.slint:114-126`).

   Do **not** try to fix this by setting `x: 0; width: 100%` on `CardDivider`. In Slint a layout
   owns the geometry of its direct children and overwrites both properties during the layout pass;
   the override is a no-op that looks like a fix in the diff.

**Watch out:** Slint cannot style a substring of one `Text`, so the attribution line becomes a run
of three elements — and a plain `HorizontalLayout` of `Text`s does not wrap. The line is ~70
characters; laid out unwrapped it will push the About pane wider than the `ScrollView` at
`main_window.slint:334` and grow a horizontal scrollbar, which is the exact defect ticket 02 exists
to remove. Whatever container is chosen must still wrap, or must be proven to fit at the pane's
minimum width.

"Proven to fit" is only a claim until a number is named. Read the window's actual minimum width from
`main_window.slint` and the card's own horizontal padding, subtract, and assert the run fits inside
what is left — or make the container wrap and assert that instead. An acceptance criterion phrased
against "the minimum rendered width" without that number is untestable, and the pane will regress
back to a scrollbar with a green suite.

If neither option is reachable in Slint at acceptable cost, breaking the line deliberately — the
attribution on one line, the link on the next — is a legitimate outcome. Say so in the commit
message rather than shipping an unwrapped run that happens to fit on the developer's monitor.

**Blocked by:** None

**Status:** ready-for-agent

- [ ] The standalone "Publisher website (wiradigital.id)" row and its adjacent `CardDivider` are gone from Card 3.
- [ ] The attribution line reads `An open-source utility by Wira Digital Indonesia ↗ • Licensed under GPL-3.0`, with only "Wira Digital Indonesia" and the icon styled as a link.
- [ ] Hovering the link changes both the text and the `OpenLinkIcon` to `Palette.accent_hover` and shows a pointer cursor; the two plain parts do not change.
- [ ] Clicking the link invokes `root.open_publisher_url()` and opens `https://wiradigital.id`.
- [ ] The link carries `accessible-role: button` and an accessible label naming the destination; the plain parts carry none.
- [ ] Card 3's `VerticalLayout` uses `padding: 0px` with per-section padding, and its remaining dividers span the full card width — verified against the Updates card, not against a geometry override on `CardDivider`.
- [ ] The About pane's minimum content width is **named as a number** taken from `main_window.slint`, and the attribution run is asserted to fit inside it minus Card 3's horizontal padding — or the run wraps, and the wrap is asserted.
- [ ] At that width the pane shows **no** horizontal scrollbar, and neither does it at the width ticket 02's hold-delay caption is asserted against.
- [ ] Removing the standalone row does not leave an orphaned `CardDivider` with no section beneath it.
- [ ] Keyboard focus reaches the inline link, and the pane's focus order has no gap where the removed row used to be (`app::tests::focus_order_has_no_duplicates` still passes).
- [ ] Automated tests cover: the inline link invokes `open_publisher_url`, the standalone row is absent, and the attribution line does not force horizontal overflow.
