# 04: Two settings — the visual switcher toggle and the hold delay

**What to build:**
The two configuration fields the feature actually needs, added to the **existing**
`SwitcherConfig`, with pre-save validation and a Settings control for each. Turning the toggle
off returns cycling to pure blind behaviour, which is the point of the switch.

`Config` already has a `switcher` field holding the cycle shortcut and its fallback. A second
`switcher` section cannot exist, and a new top-level section would split one concept across two
places in `config.toml` — so the fields go inside the struct that is already there.

**Blocked by:** 01

**Status:** done

- [x] `SwitcherConfig` gains `visual_enabled: bool` (default `true`) and
      `visual_hold_delay_ms: u32` (default `150`)
- [x] `SwitcherConfig` already carries `#[serde(default)]`, so an existing `config.toml` gains
      both fields on first load — a test round-trips a pre-SPEC-13 `config.toml` and asserts no
      migration is needed and no existing key is disturbed
- [x] Pre-save validation in `persistence.rs` rejects a hold delay outside `100..=500`
- [x] General pane exposes the toggle through `SettingToggleRow`, and the delay through the
      numeric spinner pattern `shortcut_row.slint` already uses for snap percentages — no new
      stepper component
- [x] Staging an edit marks the draft dirty and Save Changes commits it to disk
- [x] `WM_APP_CONFIG_SNAPSHOT` carries both new fields to the hook without dropping the hook,
      and a test asserts a reload with the visual switcher disabled leaves blind cycling
      unchanged
- [x] Grid geometry is deliberately **not** exposed: no `max_rows`, no `card_width`. Both are
      constants sized against the monitor
