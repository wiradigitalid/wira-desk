---
id: SPEC-11-03
component: settings
satisfies: []
blocked_by: []
status: done
tests:
  - app::tests::about_pane_renders_publisher_and_links
  - update::tests::open_in_browser_accepts_publisher_and_repo_domains
---

# 03: Settings UI About pane publisher branding, source code, license, and support navigation

**What to build:** In `crates/settings/ui/panes/about_pane.slint` and `crates/settings/src/main.rs`, enhance the About pane with official publisher branding, source code repository link, license statement, and a developer support navigation button. Navigation invokes the extended `open_in_browser` helper with strict URL validation and safe Win32 shell execution (`ShellExecuteW`).

**Blocked by:** none

## Acceptance Criteria

- [ ] In `crates/settings/ui/panes/about_pane.slint`:
      - Add a Card containing:
        - Publisher attribution: *"An open-source utility by Wira Digital Indonesia"* (clickable, opens `https://wiradigital.id`).
        - Source code & issues link: *"Source code & issue tracker on GitHub"* (clickable, opens `https://github.com/wiradigitalid/wira-desk`).
        - License information: *"Licensed under GPL-3.0"* accompanied by the honest offline badge.
      - Add a *"Support development"* button or interactive card that triggers navigation to `https://wiradigital.id/wira-desk`.
      - Follow Slint Fluent design system styling (Typography, Palette tokens, card padding, hover states).
- [ ] In `crates/settings/src/update.rs` & `crates/settings/src/main.rs`:
      - Extend the existing `open_in_browser` helper to perform strict prefix matching over HTTPS:
        - Accepts exactly `https://wiradigital.id`
        - Accepts exactly `https://wiradigital.id/wira-desk`
        - Accepts URLs starting strictly with `https://github.com/wiradigitalid/wira-desk/`
        - Rejects any other host, HTTP scheme, dot-segments, userinfo, or unpinned paths silently.
      - Continue using `ShellExecuteW` with full `SAFETY:` documentation, avoiding shell subprocess spawning (`cmd /C start`).
      - Wire Slint callbacks (`open_publisher_url`, `open_source_url`, `open_support_url`) to call `open_in_browser`.
- [ ] Automated tests in `crates/settings/src/app.rs` and `crates/settings/src/update.rs`:
      - `app::tests::about_pane_renders_publisher_and_links`: UI snapshot verifies cards and buttons render without layout clipping.
      - `update::tests::open_in_browser_accepts_publisher_and_repo_domains`: Unit tests prove allowed pinned targets are accepted and unpinned URLs/schemes are rejected.
- [ ] Release Gate Checklist: Pre-release verification ensures `https://wiradigital.id` and `https://wiradigital.id/wira-desk` return HTTP 200 before public release tag is cut.
