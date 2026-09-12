# 03: Card chrome — icon, title, theme, rounded corners, and mouse selection

**What to build:**
Everything the user actually looks at, painted with double-buffered GDI in `WM_PAINT`. Each
card gains an application icon and window title in its chrome box, a selection halo, and a
themed background with rounded corners. Mouse hover moves the selection and a click commits.

This is the ticket with no precedent in this crate and the one most likely to run long: the
daemon has never drawn anything. `FR-9` forbids heavy UI runtimes and COM GUI frameworks in
the daemon, so this adds **no** crate to `crates/daemon/Cargo.toml` — `Win32_Graphics_Gdi` is
already enabled and is the whole toolkit.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Opaque window painted in `WM_PAINT` through a memory DC, sized to the visible surface
      only — not `WS_EX_LAYERED`, whose per-pixel-alpha DIB would cost a back buffer the size of
      the overlay and would strip the alpha from every GDI glyph and icon
- [ ] Rounded corners via `DWMWA_WINDOW_CORNER_PREFERENCE`; background follows the system light
      or dark theme
- [ ] Application icon and window title in each card's chrome box, never over the preview box
- [ ] Titles read in the switcher's own path, not in the `cycling/source.rs` sweep, and only for
      the cards actually drawn; the comment at the call site states why `GetWindowTextW` is
      admissible here (cached caption cross-process, documented not to hang on a hung owner)
      when the sweep forbids it
- [ ] A window that fails thumbnail registration, or whose display affinity is
      `WDA_EXCLUDEFROMCAPTURE` and composes black, shows icon and title in a card of the same
      size
- [ ] Selection halo drawn in the chrome area around the selected card
- [ ] Mouse hover moves the selection; a click commits that card. The overlay still never
      activates on click
- [ ] Text and icon metrics scale with the monitor's DPI
- [ ] All `unsafe` blocks carry `SAFETY:` comments; every GDI object created is selected out and
      deleted on the same path that created it
