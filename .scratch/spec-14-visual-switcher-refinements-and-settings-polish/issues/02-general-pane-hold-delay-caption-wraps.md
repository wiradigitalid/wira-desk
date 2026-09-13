# 02: General pane hold-delay caption wraps

**What to build:**
Add `wrap: word-wrap` to the hold-delay caption `Text` at
`crates/settings/ui/panes/general_pane.slint:79-84`
(*"Milliseconds to hold the chord before the visual overlay appears (100–500 ms)."*).

Without it the `Text` reports its full single-line width as its minimum width, the enclosing
`HorizontalLayout` (`alignment: space-between`, `:62`) widens past the card, and the `ScrollView` at
`main_window.slint:334` grows a horizontal scrollbar. The row is conditional on
`root.visual_switcher_enabled`, which is why the scrollbar appears and disappears with the toggle.
Every other long caption in the file already sets `wrap: word-wrap`; this one was missed.

Settings-only. The daemon-side half of the original ticket 02 — making the Worker honour
`visual_enabled` — is now `SPEC-14-05`, because it shares no code, no risk, and no Product Component
with this change.

**Blocked by:** None

**Status:** ready-for-agent

- [ ] `general_pane.slint:80`'s caption sets `wrap: word-wrap`.
- [ ] Toggling "Enable Visual Switcher Overlay" on and off produces no horizontal scrollbar on the General pane at the window's minimum width.
- [ ] The minus/value/plus spinner group stays right-aligned and keeps its 30×30 hit targets once the caption wraps to two lines.
- [ ] An automated test asserts the caption's wrap mode, so the regression cannot return unnoticed.
- [ ] That test reads the actual `.slint` source or the rendered element — not a Rust-side constant that can agree with a broken UI.
- [ ] The same check covers **every** caption in `general_pane.slint`, not only this one. One caption was missed because nothing enumerated them; fixing the instance and leaving the enumeration undone invites the next miss.
