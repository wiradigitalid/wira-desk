# Data model — settings

Persistent entities are stored in `_platform` entity `app-config` (`%APPDATA%\WiraDesk\config.toml`). This dictionary describes the settings-owned slice of that schema.

## Entity relationship

```mermaid
erDiagram
    user-shortcut-preference ||--|| app-config : "stored in"
    onboarding-completion ||--|| app-config : "stored in"
    auto-start-preference ||--|| app-config : "stored in"
    auto-start-preference ||--o| scheduled-task : "registers"
```

## user-shortcut-preference

| Column | Type | Nullable | Meaning |
| --- | --- | --- | --- |
| cycling_primary | string | no | Canonical chord, e.g. `Win+Oem3` |
| cycling_fallback | string | yes | Optional `Alt+Oem3` fallback |
| snap_left | string | no | Half-left snap binding |
| snap_right | string | no | Half-right snap binding |
| snap_top | string | no | Half-top snap binding (FR-22) |
| snap_bottom | string | no | Half-bottom snap binding (FR-22) |
| snap_maximize | string | no | Maximize binding |
| move_next_monitor | string | no | Next-monitor move binding (FR-23) |
| snap_percent_left | string | no | Custom-percentage left-edge snap binding (FR-26) |
| snap_percent_right | string | no | Custom-percentage right-edge snap binding (FR-26) |
| snap_percent_top | string | no | Custom-percentage top-edge snap binding (FR-26) |
| snap_percent_bottom | string | no | Custom-percentage bottom-edge snap binding (FR-26) |
| snap_stack | string | no | Overlapping stack binding. Default `ctrl+alt+shift+s` (`DEC-011`; was `ctrl+alt+shift+down` until the arrow tier above was freed for the four `snap_percent_*` rows) — placed **after** them in this declared sequence on purpose: on an install still holding the retired default, the percent-snap row must resolve the chord first, per `DEC-011`'s cost and the `DEC-009` mechanism it relies on |
| snap_third_left | string | no | Left-third snap binding (FR-27) |
| snap_third_middle | string | no | Middle-third snap binding (FR-27) |
| snap_third_right | string | no | Right-third snap binding (FR-27) |
| `<action>`_enabled | bool | no | Per-action enabled/disabled flag, one per row above, independent of its chord string. `[MISSING]` — planned by this pass (FR-28). Disabling never clears the chord column; re-enabling reads the same stored chord back. Distinct from `unbound`, which this table never records — that state is `window-management`'s runtime derivation, not a persisted value (`BR-9`). **Must default to `true` on every existing install's config, not Rust's derived `bool` default of `false`** — `shared::Config`'s structs already carry `#[serde(default)]` at the container level, so a missing field is filled from that struct's own hand-written `Default` impl rather than the primitive default, and every one of these sixteen fields' `Default` arm must set it to `true` explicitly. Getting this wrong on any one of them silently disables that action for every config.toml written before this field existed. |

### Dictionary

- The **row order above is the declared sequence** `LBR-ST-14` names: the Shortcuts pane is drawn from it, keyboard focus follows it, and a chord collision is resolved in favour of whichever row comes first. There is no second list.
- Grouping the rows under headings in the pane is a presentation concern and belongs to `.how/settings/01-ux/DESIGN.md`. It must not reorder them relative to this sequence.
- Every value is a **canonical** chord string. Two rows holding the same canonical string is the collision condition `BR-6` governs; this component refuses to save it at all.

Schema source: `shared::Config` in `crates/shared/src/config.rs`.

## arrangement-percentage-preference

The percentage each `snap_percent_*` chord snaps to — a value, not a chord, so it is not part of the
declared sequence above and cannot collide with anything (FR-26).

| Column | Type | Nullable | Meaning |
| --- | --- | --- | --- |
| percent_left | u8 | no | Percentage of work-area width, left edge. Default `50` |
| percent_right | u8 | no | Percentage of work-area width, right edge. Default `50` |
| percent_top | u8 | no | Percentage of work-area height, top edge. Default `50` |
| percent_bottom | u8 | no | Percentage of work-area height, bottom edge. Default `50` |

## onboarding-completion

| Column | Type | Nullable | Meaning |
| --- | --- | --- | --- |
| tutorial_completed | bool | no | User finished interactive practice |
| skipped | bool | no | User chose Skip Tutorial |

## auto-start-preference

| Column | Type | Nullable | Meaning |
| --- | --- | --- | --- |
| enabled | bool | no | Whether logon task is registered |
| task_name | string | no | `WiraDesk` (`shared::TASK_NAME`) |

## scheduled-task (external)

Not stored in TOML; created by `schtasks` when `enabled` is true.

| Column | Type | Meaning |
| --- | --- | --- |
| trigger | ONLOGON | Runs at user logon |
| run_level | HIGHEST | Matches daemon elevation (AD-13) |
| run_as | `%USERNAME%` | Aligns APPDATA paths (BR-4) |
