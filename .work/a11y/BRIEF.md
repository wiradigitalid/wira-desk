# Task — finish the accessibility (UIA) evidence pass over Wira Desk Settings

You are verifying an already-shipped UI. You MUST NOT change application code.

## Why this exists

`RISK-3` in `.control/registry/risks.yaml`: the accessibility evidence for FR-20 (full keyboard
navigation) and FR-21 (screen-reader exposure via UI Automation) was gathered against the AccessKit
adapter in eframe 0.35. The repo is now on eframe/egui 0.36.1, which replaces that adapter, and no
automated test covers the UIA tree. The pass re-gathers that evidence on 0.36.1.

Part of the pass is already done (results below). Your job is the remaining part.

## Setup — do not rebuild

The binary already exists. Building it again wastes ~6 GB and minutes:

    target/debug/wiradesk-settings.exe

Launch it with `APPDATA` redirected to a scratch directory so nothing touches the real user profile:

    APPDATA=<some scratch dir> WIRADESK_SKIP_MANIFEST=1 ./target/debug/wiradesk-settings.exe &

On first run it shows a 2-step tutorial. Click "Skip Tutorial", then "Finish" to reach the panes.

Two helpers are in this folder — use them, do not reinvent:

- `uia-dump.ps1 -ProcessId <pid>` — full UIA tree with role, name, ToggleState, Value, RangeValue,
  IsKeyboardFocusable, HasKeyboardFocus.
- `focused.ps1 -ProcessId <pid>` — the currently focused element only. Call it after each Tab.

Drive the UI with Orca computer-use (`orca skills get computer-use` first, then
`orca computer get-app-state|click|press-key --app pid:<pid>`). Element indexes go stale after every
state change — re-read state before each click.

## HARD CONSTRAINTS

- MUST NOT press **Save** while the Auto-Start checkbox is enabled. Auto-start registers a real
  Windows logon task, and that is NOT covered by the APPDATA sandbox. Reading and toggling the
  draft is safe; saving an enabled auto-start is not.
- MUST NOT modify any file outside `.work/a11y/`.
- MUST NOT run `git commit`, `git push`, or create a PR. Publishing belongs to the orchestrator.
- MUST NOT rebuild the workspace or edit `Cargo.toml`.
- Kill the app process when finished.

## Already established — do not redo

On eframe 0.36.1 the UIA tree is published and carries role, name, value, and listening state:

- Pane tabs are `Button` with a TogglePattern; the active pane reads `toggle=On`.
- General pane: `CheckBox` name='Start Wira Desk with Windows', `toggle` flips On/Off live.
- Layout pane: `CheckBox` 'Enable overlapping stack layout'; `Slider` and `Spinner` both named
  'Stack width percent', `value='50'`, `range=50[10..100]`.
- Shortcuts pane: each row is a `Text` label plus a focusable `Button` whose name embeds the current
  value, e.g. "Snap to left half. Current shortcut ctrl+win+left."
- Arming a shortcut capture changes that button's name to
  "Listening for a key combination. Press Escape to cancel." and it takes keyboard focus.
- Tab order, **General** pane: General → Shortcuts → Layout → About → Start Wira Desk with Windows
  → Save → Revert → wraps. Matches `focus_order(Pane::General)` in `crates/settings/src/app.rs`.
- Tab order, **Layout** pane: matches the declaration EXCEPT that 'Stack width percent' occupies
  two Tab stops (Slider then Spinner) where `focus_order` declares one — 9 actual stops vs 8.
- `Revert` is disabled while the draft is clean, so Tab correctly skips it. Not a defect.

## YOUR WORK

1. **Walk the Tab order of the Shortcuts pane**, and compare it against `focus_order(Pane::Shortcuts)`
   in `crates/settings/src/app.rs` (read that function; do not guess the expected list). Record the
   actual focused element after every Tab for one full cycle, using `focused.ps1`.
2. **Walk the Tab order of the About pane** the same way, against `focus_order(Pane::About)`.
3. **Confirm Shift+Tab reverses** the order on any one pane. FR-20 names Shift+Tab explicitly.
4. **Check Escape cancels an armed shortcut capture** and that the button's name reverts to the
   "Current shortcut ..." form.
5. For every pane, note any control that is missing a name, has an empty name, exposes no role, or
   shows a duplicate name shared with another focusable control.

## What done means

Write two files in `.work/a11y/`:

- `raw-dumps.txt` — the literal command output you collected, unedited. This is the evidence.
- `REPORT.md` — a short structured report with, for each of the 5 items above: what you ran, what
  you observed, and a verdict of PASS / DEVIATION / FAIL. For every DEVIATION or FAIL state the
  declared expectation, the observed behaviour, and the pane. No prose padding.

Then report completion exactly once:

    orca orchestration send --type worker_done --subject "<status>" \
      --body "<verdict per item, and anything you could not do>" \
      --task-id <TASK_ID> --dispatch-id <DISPATCH_ID> --outcome succeeded \
      --files-modified ".work/a11y/raw-dumps.txt,.work/a11y/REPORT.md" --json

Use `--outcome failed` if you could not complete it; never encode failure only in prose.
