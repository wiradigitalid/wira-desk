# 01: About pane product pillars, in-process disclosure, and external link icons

**What to build:** Update the Settings About pane layout and copy to reflect Wira Desk's 3 core pillars (window switching, edge snapping, mouse navigation), clarify that update checks run in-process without a separate Windows Service, reorganize Card 3's information hierarchy with prominent community/support links and foundational legal/publisher attribution, and add vector Open Link external icons to web links.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] The primary product description in `about_pane.slint` reads: *"Wira Desk accelerates desktop multitasking with smooth window switching, flexible edge snapping, and driverless mouse navigation."*
- [ ] The secondary description caption remains: *"Designed to be invisible, fast, and resource-efficient for all-day multitasking."*
- [ ] The security/privacy disclosure reads: *"No telemetry, no account, no separate background service — update checks run entirely in-process against GitHub Releases, which you can switch off."*
- [ ] Card 3 is restructured with user actions (GitHub repository, publisher website, support button) placed prominently in the card body, followed by a divider and the grouped attribution/licensing line (*"An open-source utility by Wira Digital Indonesia • Licensed under GPL-3.0"*) and disclosure.
- [ ] The `open_publisher_url()` callback wiring moves from the old attribution row onto the new dedicated "Publisher website" action row; the foundational attribution/licensing line at the bottom is plain static text (no `TouchArea`), so there is exactly one way to reach `wiradigital.id` from this card.
- [ ] No separate "Report an Issue" link is added — "Source code & issue tracker on GitHub" (routing to the repo root, where the Issues tab is reachable) already satisfies that checklist item; this is confirmed, not left open.
- [ ] Outbound web links ("Source code & issue tracker on GitHub" and publisher website) display an inline vector Open Link icon (`↗`) that matches text colors on hover.
- [ ] Web navigation continues to be strictly governed by `update::open_in_browser` with existing HTTPS allowlist enforcement.
- [ ] Automated tests verify the updated copy, layout components, and link callbacks.
- [ ] `scripts/verify-public-export.ps1` passes cleanly with zero hygiene violations.
