# 02: Mouse preset dropdown category header inertness and click absorption

**What to build:** Ensure that clicking non-selectable category headers in `PresetDropdownOverlay` does not close the dropdown overlay or trigger any selection, making category headers completely inert and absorbing pointer clicks.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] In `crates/settings/ui/main_window.slint`, category header rows in the `dropdown_items` list intercept click events with an empty/noop touch handler so clicks do not fall through to the background backdrop.
- [ ] Clicking any category header (e.g., "Virtual Desktops", "Windows Shell", "Switching", "Snap to Half", "Snap to Third", "Snap to Custom", "Arrange & Move", "Passthrough") leaves the dropdown overlay open (`root.dropdown_open == true`).
- [ ] Clicking a category header does NOT select any preset or mutate the current draft configuration.
- [ ] Category headers maintain non-interactive visual styling: no hover background highlight and default non-pointer cursor. (Cursor shape is a manual/visual check — Slint has no automated hook for `mouse-cursor`; confirm by hand during review, don't assume the test suite covers it.)
- [ ] Automated tests verify that clicking a header row does not dismiss the overlay.
