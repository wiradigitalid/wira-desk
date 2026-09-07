# Live test script — mandate DEC-016 (post-Finish, owner-assisted)

You are running the **live** portion of the smoke test that the previous headless pass could not reach.
The human owner is present at this machine and will click through any UAC elevation prompt when it
appears — do not treat an elevation prompt as a blocker; launch the daemon and wait for it to actually
start (poll for the process/mutex rather than assuming failure).

This is testing only. Do not modify code. Do not commit anything. Work in this worktree, already checked
out to `autopilot/DEC-016` with today's release binaries already built in `target/release/`:
`D:\Developer\wiradigital.id\wira-desk-autopilot`. If the binaries are stale (older than the latest
commit), rebuild with `./build.ps1` first.

Report honestly per item — PASS with what you actually observed, FAIL with what broke, or NOT
VERIFIABLE with the specific reason — never round up.

## 1. `FR-28` — per-action enable/disable, live

1. Launch `wiradesk-settings.exe`.
2. Navigate to the Shortcuts pane (try UI Automation `SelectionItemPattern`/`InvokePattern` on the nav
   item first; if neither is exposed, as the previous pass found, try clicking via `SetCursorPos` +
   `mouse_event` at the item's actual bounding-rectangle coordinates now that a human is present to
   restore foreground focus if `SetForegroundWindow` is refused).
3. Turn off "Snap to left third", click Save.
4. Confirm: every other action's chord is unchanged, the row now reads visibly different (the
   "(disabled)" state, not a collision), and `%APPDATA%\WiraDesk\config.toml` actually persisted the new
   `snap_percent_left_enabled = false` (or the equivalent frozen key name — read the real field name from
   `crates/shared/src/config.rs` rather than guessing it).
5. Turn it back on before moving to the next step, so the daemon test below sees a clean baseline.

## 2. `SPEC-2-01` — commit-on-departure, live

With the Shortcuts pane reachable from step 1:
1. Type a percentage value into a snap-percentage field WITHOUT pressing Enter, click Save. Confirm the
   typed value was actually saved (not the old one) by re-reading `config.toml` or re-opening the pane.
2. Type a value into one row, then click a stepper (`+`/`-`) on a **different** row. Confirm the first
   row's typed value survived (this is the cross-row bug `SPEC-2-01`'s return trip fixed) and the second
   row's stepper computed from ITS just-typed value if one was pending.
3. Type an out-of-range value (e.g. outside 1-99) and try to commit it via Save. Confirm it's refused with
   an actionable message, not silently clamped.

## 3. `FR-25` — Check for Updates, live (no-update-available path)

The current build is version `0.1.5`, which is already the latest published GitHub release — so there is
no real newer version to test the download path against, and none should be manufactured for this test.
1. Navigate to the About pane, click "Check for Updates".
2. Confirm the real HTTPS request goes out (to this repo's actual GitHub releases endpoint — check with
   a network trace or the app's own log if easily available) and the UI reports "up to date" / no update
   available, rather than erroring or hanging.

## 4. `FR-29` — hook registration excludes disabled actions, live (the hard one)

1. Launch the daemon (`wiradesk.exe`). **This requires elevation — the owner will approve the UAC prompt.**
   Wait for it to actually be running (check the process list and/or its single-instance mutex) before
   proceeding; do not give up after the prompt appears, that's expected.
2. Using the Settings GUI (now against the running daemon, not the `$env:WIRADESK_SETTINGS_ALLOW_NO_DAEMON`
   standalone mode used in the prior pass), disable one specific, easy-to-target action — pick one bound to
   a chord unlikely to already be claimed by anything else running, e.g. whatever the shipped default for
   "Snap to left third" is. Save so the daemon reloads config live.
3. Open a plain text editor (e.g. Notepad) as the "other application". Attempt to reproduce the disabled
   action's exact chord as a real keystroke — try, in order, whichever of these actually work in this
   session: `[System.Windows.Forms.SendKeys]::SendWait(...)`, or a P/Invoke `SendInput`/`keybd_event` call.
   If neither can be attempted or both are refused/blocked (report the specific error), say so plainly
   rather than assuming.
4. Confirm from the daemon's own log (`%APPDATA%\WiraDesk\wiradesk.log`) or observable behaviour whether the
   disabled action's chord was silently swallowed (bug, would mean the fix regressed) versus reaching the
   other application normally (correct).
5. Re-enable the action afterward so the machine isn't left with a shortcut disabled from this test.

## Report back

One line per item (1-4): PASS / FAIL / NOT VERIFIABLE, with the concrete evidence or the specific blocker.
Do not summarize away a partial result — if step 4's keystroke injection didn't work but the daemon
elevation itself succeeded, say exactly that.
