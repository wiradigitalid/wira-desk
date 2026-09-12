---
spec: SPEC-12
release: "0.3.0"
prd: wira-desk
fr: []
status: open
---

# SPEC-12 — About Pane Hierarchy Overhaul, 3-Pillar Positioning, and Settings Interaction Polish

## Problem Statement

Manual user testing and design review following the delivery of SPEC-9, SPEC-10, and SPEC-11 surfaced six concrete UI/UX findings across the Settings About pane and interaction controls:

1. **Stale Single-Feature Product Description:** In `crates/settings/ui/panes/about_pane.slint`, the About pane still describes Wira Desk solely as a window switcher (*"Wira Desk switches smoothly between windows of the active application on your current monitor"*). This description predates the v0.2.0 and v0.3.0 feature deliveries and completely omits the other two core pillars: window edge snapping (halves, thirds, custom percentages) and driverless mouse navigation (thumb buttons, tilt wheel, 20 presets across 8 categories).
2. **Ambiguous Background Updater Copy:** The security and privacy disclosure in `about_pane.slint` states *"no background updater service"*. When read beside the adjacent *"Check for updates automatically"* toggle, users find this contradictory—wondering how updates can be checked automatically if there is no background service. The intended technical fact is that Wira Desk does not register or run a separate, persistent Windows Service daemon; update checks run entirely in-process within the running application when enabled.
3. **Suboptimal About Pane Information Hierarchy:** Publisher attribution (*"An open-source utility by Wira Digital Indonesia"*) is currently positioned as the top interactive row of Card 3, above the repository link and disconnected from licensing. In modern desktop application conventions (Fluent Design System, Windows Settings, PowerToys), interactive community actions (Source code, Website, Support) form the primary card body, while formal publisher credits and license disclaimers reside together as the foundational base.
4. **Missing External Link Affordance:** The About pane includes interactive links to external web destinations (the Wira Digital Indonesia homepage and the GitHub repository & issue tracker), but renders them as plain text without visual affordance. Users cannot distinguish at a glance between in-app navigation and outbound browser actions.
5. **Mouse Dropdown Category Headers Dismiss on Click:** In `crates/settings/ui/main_window.slint`, clicking a category header (e.g., *"Virtual Desktops"*, *"Snap to Half"*, *"Arrange & Move"*) in `PresetDropdownOverlay` closes the dropdown overlay. This happens because category header rows have `item_touch.enabled: false`, causing mouse clicks to fall through the overlay to the underlying full-window backdrop `TouchArea`, which dismisses the dropdown. Category headers should be completely inert and absorb clicks without dismissing the overlay.
6. **Missing Visual Dirty State on Save Changes Button:** In the sticky bottom footer of `main_window.slint`, the `Revert` button accurately reflects `root.is_dirty` (disabled and muted when clean; active when changes exist). However, the `Save Changes` button is statically rendered with vibrant `Palette.accent_primary` and remains always-clickable regardless of whether the configuration is clean or modified, failing to provide visual confirmation of unsaved changes.

## Solution

1. **3-Pillar Positioning Statement (`SPEC-12-01`):**
   Update the primary product description in `about_pane.slint` to encompass all three core product capabilities:
   *"Wira Desk accelerates desktop multitasking with smooth window switching, flexible edge snapping, and driverless mouse navigation."*
   Retain the secondary caption: *"Designed to be invisible, fast, and resource-efficient for all-day multitasking."*

2. **In-Process Updater Clarification (`SPEC-12-01`):**
   Refine the security disclosure copy to eliminate confusion:
   *"No telemetry, no account, no separate background service — update checks run entirely in-process against GitHub Releases, which you can switch off."*
   Mirror this updated phrasing in user-facing documentation where applicable, maintaining compliance with `scripts/verify-public-export.ps1`.
   **Canonical-wording note:** `.scratch/handover-branding-and-distribution-2026-09-11.md` (T2) proposed an earlier draft of this same disclosure — *"No telemetry, no account, no background updater service — one HTTPS request to GitHub when you check for updates, which you can switch off."* That draft predates this spec's owner-authored wording and is **superseded** by the sentence above. Any future ticket drawn from that handover (T1/T2/T12, covering `README.md`/`SECURITY.md`) MUST reuse this spec's wording verbatim rather than the handover's draft, so the About pane and the public docs never state the same fact two different ways.

3. **About Pane Hierarchy Overhaul & External Link Icons (`SPEC-12-01`):**
   Restructure Card 3 in `about_pane.slint` into a coherent visual hierarchy:
   - **Primary Action Rows:**
     - Row 1: Source code & issue tracker on GitHub, featuring an inline vector `Open Link` icon (`↗`).
     - Divider.
     - Row 2: Publisher website (`wiradigital.id`), featuring an inline vector `Open Link` icon (`↗`).
     - Divider.
     - Row 3: Support development button (`Support development`).
   - **Foundational Base:**
     - Divider.
     - Grouped legal and attribution footer:
       *"An open-source utility by Wira Digital Indonesia • Licensed under GPL-3.0"*
       Followed by the in-process updater disclosure.
   - **Vector Icon:** Render the external link indicator using a clean Slint `Path` (diagonal arrow and square bracket) that automatically inherits theme accent/hover colors.
   - **Callback relocation:** The `open_publisher_url()` wiring moves from the old top-of-card attribution row onto the new "Publisher website" action row. The foundational attribution/license footer becomes static, non-interactive text — it MUST NOT retain its own `TouchArea`, or the pane ships two ways to reach the same URL.
   - **"Report an Issue" wording (closed, no change):** The manual test notes flagged that the card reads *"Source code & issue tracker on GitHub"* rather than a separate "Report an Issue" link. `open_source_url()` resolves to the repository root (`https://github.com/wiradigitalid/wira-desk/`), where the Issues tab is one click away, so the combined phrasing already satisfies the intent. No separate issue-tracker link is added by this spec.

4. **Inert Category Headers in Preset Dropdown (`SPEC-12-02`):**
   Update `PresetDropdownOverlay` in `main_window.slint` so that category header items intercept pointer events without triggering selection or dismissal. The header `Rectangle` consumes the click event rather than allowing it to pass through to the backdrop dismiss handler.

5. **Save Changes Button Dirty State Affordance (`SPEC-12-03`):**
   Bind the visual styling and click availability of the `Save Changes` button in `main_window.slint` to `root.is_dirty`:
   - When `root.is_dirty == false` (clean state): Render with muted subtle background (`Palette.bg_subtle`), disabled text color (`Palette.text_tertiary`), and `enabled: false`.
   - When `root.is_dirty == true` (dirty state): Render with vibrant accent background (`Palette.accent_primary`), hover styling (`Palette.accent_hover`), and active click handling.

## User Stories

1. As a user viewing the About pane, I want the product description to clearly explain all three core pillars (switching, snapping, mouse navigation) so I understand the full scope of what Wira Desk does.
2. As a user reviewing update settings, I want the disclosure to clearly state that update checks run in-process without installing a separate Windows Service so I am not confused by the auto-update toggle.
3. As a user clicking links in the About pane, I want to see external link icons so I immediately know which actions will launch my default web browser.
4. As a user viewing the About pane, I want publisher attribution and licensing to appear naturally grouped at the base of the card, with developer links and support actions positioned prominently.
5. As a user navigating the mouse preset dropdown, I want category headers to be inert when clicked, so that accidentally clicking a header does not close the dropdown menu.
6. As a user modifying settings, I want the "Save Changes" button to visually activate only when changes have actually been made, providing clear visual feedback on whether my configuration is dirty or saved.

## Implementation Decisions

- **Seam: Slint UI Layer (`crates/settings/ui/`):** All changes in this spec reside in the declarative Slint presentation layer and its Rust model bindings (`crates/settings/src/main.rs`). The core daemon, hook thread, IPC message ring, and config schema are untouched.
- **Vector Graphics for External Link:** Rather than relying on system font glyph availability (which varies across Windows 10 vs. Windows 11 builds), external link icons will be rendered using a theme-aware Slint `Path` component matching existing titlebar and sidebar vector icons.
- **Inert Event Consumption:** Slint `TouchArea` elements with empty `clicked => {}` blocks absorb click events. Providing an active `TouchArea` on category headers prevents click propagation to the parent/backdrop while keeping `item.is_header` non-selectable.
- **Dirty State Symmetry:** Binding both `Revert` and `Save Changes` to `root.is_dirty` provides intuitive symmetry in the footer bar: both buttons remain visually disabled when the current draft matches the persisted configuration, and both light up when changes occur.

## Testing Decisions

- **UI Snapshot & State Tests:** Add automated unit tests in `crates/settings/src/app.rs` and Slint model verification tests confirming:
  - About pane renders the updated 3-pillar description and revised in-process updater disclosure.
  - About pane external links invoke the registered callbacks with validated HTTPS URLs.
  - Dropdown category header click does not mutate `dropdown_open`.
  - `Save Changes` button enabled and dirty bindings reflect model `is_dirty()` transitions.
- **Public Export Verification:** Run `scripts/verify-public-export.ps1` to ensure new disclosure phrasing passes public release hygiene checks.

## Out of Scope

- Changes to the underlying updater networking logic or GitHub Releases metadata format.
- Adding additional external links or donation rails (handled separately under T11 / `ODR-003`).
- Altering the TOML serialization format or daemon config reload protocol.
- `README.md` / `SECURITY.md` corrections (handover T1/T2) and the README rewrite (T12). Those surfaces
  still carry the stale single-pillar and "no network path" claims this spec fixes only in-app; they
  are a separate corpus SPEC and MUST adopt this spec's wording per the canonical-wording note above.

## Further Notes

- Grounded in manual test checklist results from 2026-09-12 and `.scratch/handover-branding-and-distribution-2026-09-11.md`.
