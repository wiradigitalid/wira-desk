# Chord table — the nine actions, their fields, and their wire values

Projected from `DEC-008`, `DEC-009`, `AD-2`, `.how/settings/05-model/data-model.md`, and
`.what/settings/04-usecases/EXPERIENCE.md`. Nothing here is new.

## The declared sequence

This order is load-bearing three times over: it is the Shortcuts pane's draw order, its keyboard
focus order, and the precedence order that resolves a chord collision (`LBR-ST-14`, `DEC-009`).
There must be exactly one list of it in the code.

| # | Config field | Wire | Shipped default | Group in the pane | Row label |
|---|---|---|---|---|---|
| 1 | `switcher.shortcut` | `1` Cycle | `win+backtick` | Switching | Switch windows of the same application |
| 2 | `switcher.fallback_shortcut` | `1` Cycle | `alt+backtick` | Switching | Fallback switch shortcut |
| 3 | `snapping.snap_half_left` | `2` SnapLeft | `ctrl+alt+left` | Snap & resize | Snap to left half |
| 4 | `snapping.snap_half_right` | `3` SnapRight | `ctrl+alt+right` | Snap & resize | Snap to right half |
| 5 | `snapping.snap_half_top` | `6` SnapTop | `ctrl+alt+up` | Snap & resize | Snap to top half |
| 6 | `snapping.snap_half_bottom` | `7` SnapBottom | `ctrl+alt+down` | Snap & resize | Snap to bottom half |
| 7 | `snapping.snap_maximize` | `4` SnapMaximize | `ctrl+alt+enter` | Snap & resize | Maximize |
| 8 | `layout.move_next_monitor_shortcut` | `8` MoveToNextMonitor | `ctrl+alt+shift+enter` | Move & arrange | Move to next monitor |
| 9 | `layout.stack_shortcut` | `5` OverlappingStack | `ctrl+alt+shift+down` | Move & arrange | Overlapping stack |

Row order and wire order deliberately disagree. Wire values were assigned when each action was
born and may never be renumbered; row order follows the arrow keys and reads for a human. Neither
is derivable from the other, so both are written down.

## What changes per field

| Field | Before this wave | After |
|---|---|---|
| `snapping.snap_half_left` | `ctrl+win+left` | `ctrl+alt+left` |
| `snapping.snap_half_right` | `ctrl+win+right` | `ctrl+alt+right` |
| `snapping.snap_half_top` | did not exist | `ctrl+alt+up` |
| `snapping.snap_half_bottom` | did not exist | `ctrl+alt+down` |
| `snapping.snap_maximize` | `ctrl+win+enter` | `ctrl+alt+enter` |
| `layout.move_next_monitor_shortcut` | did not exist | `ctrl+alt+shift+enter` |
| `layout.stack_shortcut` | `ctrl+win+down` | `ctrl+alt+shift+down` |
| `switcher.shortcut` | `win+backtick` | unchanged |
| `switcher.fallback_shortcut` | `alt+backtick` | unchanged |

The switcher chords are the product's identity and do not move. `DEC-003` already carves out
`alt+backtick` by name.

## The four frozen tests this wave spends

They were pinned so the values could not drift unnoticed, and they are doing their job by making
this change loud. Each must be updated, not deleted.

| Test | File |
|---|---|
| `frozen_snapping_defaults` | `crates/shared/src/config.rs` |
| `frozen_stack_shortcut_default` | `crates/shared/src/config.rs` |
| `arrangement_shortcut_defaults_are_frozen` | `crates/daemon/src/arrangement/mod.rs` |
| fallback literals in `load_shortcuts` | `crates/daemon/src/hook.rs` |
| default assertions in `persistence` tests | `crates/settings/src/persistence.rs` |

## Reserved catalogue addition

| Chord | Kind | Owner text |
|---|---|---|
| `Win + Ctrl + Left` | `ShellOwned` | switch between your virtual desktops |
| `Win + Ctrl + Right` | `ShellOwned` | switch between your virtual desktops |

`ShellOwned` rather than `Immutable` because Windows can be prevented from acting on them — the
refusal can honestly offer an alternative, which is the distinction `DEC-003` draws between the two
kinds. `win+ctrl+d` and `win+ctrl+f4` are already in the catalogue; these two complete that feature.

`ctrl+alt+*` is verified absent from the catalogue, and the escape-hatch guards (`Alt+Tab`,
`Alt+F4`, `Alt+Shift+Tab`) are all conditioned on `alt && !ctrl`, so adding `Ctrl` leaves them
untouched by construction rather than by luck.

## Collision precedence, worked

The only collision this wave creates on a real machine: a `config.toml` that already sets
`layout.stack_shortcut = "ctrl+alt+down"` meets the new `snapping.snap_half_bottom` default of the
same chord.

| Field | Sequence position | Outcome |
|---|---|---|
| `snapping.snap_half_bottom` | 6 | keeps the chord |
| `layout.stack_shortcut` | 9 | **unbound** — unreachable until the user changes one of the two |

The new feature wins and the existing one goes dark. `DEC-009` records this as the least defensible
consequence of the rule rather than hiding it.
