# Smoke test — mandate `DEC-017`, unattended

You are running the smoke test for this run. **Testing only: do not modify code, do not commit
anything.** Work in this worktree, checked out to `autopilot/DEC-017`.

**Report honestly per item — PASS with what you actually observed, FAIL with what broke, or NOT
VERIFIABLE with the specific reason — never round up.** That convention is inherited from the
previous mandate's live test (`.scratch/live-test-dec-016.md`) and it is the whole value of this
step. A PASS you did not observe is worse than a NOT VERIFIABLE, because it retires a question
nobody will ask again.

## What the coordinator already established about this machine

Read this before planning, because two of it will shape your verdicts:

- **The daemon is running, but from a stale build** — `target/release/wiradesk.exe` is dated
  2026-08-28, three weeks before this run's code. The IPC contract this run touches
  (`WM_APP_CAPTURE_LEASE`, the daemon→settings chord report) did not change in `SPEC-4`, so the
  key-check correlation still exercises the **Settings** side, which is what changed. Say plainly
  in any verdict that relies on it that the daemon half was not this build.
- **This session is not elevated, and the owner is not present to clear a UAC prompt.** So you
  cannot launch a fresh elevated daemon. Do not try to work around elevation; record the affected
  items as NOT VERIFIABLE with that reason.
- `build.ps1` calls plain `cargo build --release`, so it honours the `CARGO_TARGET_DIR` already set
  in your environment. **Do not pass `--target-dir` and do not change it** — one build location is
  the repo owner's explicit instruction, and you hold that lock alone right now.

## Step 0 — build, and launch what you can

1. `./build.ps1` (release). Confirm `wiradesk-settings.exe`'s timestamp is newer than commit
   `444be47` before trusting any observation below — a stale binary silently invalidates everything.
2. Launch `wiradesk-settings.exe`. It carries no elevation manifest by design, so it starts without
   a prompt. If it will not start attached to the stale daemon, `WIRADESK_SETTINGS_ALLOW_NO_DAEMON=1`
   runs it standalone — but note which mode each observation came from, because the key-check items
   need the daemon.

## The Shortcuts pane — `FR-15`, `FR-26`, `FR-28`

1. **Five groups, in order.** Shortcuts pane shows **Switching**, **Snap to half**, **Snap to
   third**, **Snap to custom**, **Resize, move & arrange**, in that order, and no row repeats its
   group's name — under *Snap to custom* a row reads "Snap to left edge", not "Snap left (custom %)".
2. **No sideways scrolling.** At the default window size, scroll the pane top to bottom. No
   horizontal scrollbar appears and nothing is clipped at the right edge. *The automated guard
   measures the window edge, not the pane's content edge, so this step is what covers the gap.*
3. **Description as tooltip.** Hover a row's title: the full description appears, not truncated.
   Move the pointer away: nothing extra remains drawn. Then press **Tab** and confirm the same
   description surfaces on focus alone, with no mouse. **Three things the tests structurally cannot
   see, so look for them:** the tooltip is positioned at `y: root.height + 2px`, outside the row's
   own bounds, so on the last visible row it may be clipped by the scroll area and elsewhere it may
   overdraw the next row's divider; the tooltip box declares a width but no height, so a long
   description may clip; and check the tooltip is readable rather than merely present.
4. **Row alignment.** The enable toggle sits on the same vertical middle as the keycap beside it.
   Check a row with a percentage stepper and a row without, and a row you have toggled **off** (the
   "Disabled" line makes that row taller — 52.5px against 51px — which is expected).
5. **Four panes, no Layout pane.** General, Shortcuts, VM & Exceptions, About. Overlapping Stack's
   width percentage sits inline on its own row, and the value you had is unchanged.
6. **Custom-percentage snap works end to end.** Set a percentage on a snap row, save, press that
   chord: the window snaps to that share of the screen, not to half.

## `DEF-5` — typed digits, which is this run's last fix

7. Click into a percentage field and **type** `68` on the keyboard. The digits must appear as you
   type. Then press Tab or click Save: the value commits. Try **backspace** too — the same fix
   governs it.
8. Type an out-of-range value (`0`, or `120`) and depart the field. It must be **refused with an
   actionable message**, not silently clamped and not silently saved.

## `DEF-9` — the keyboard path, and each of these must start after focus has moved

`SPEC-4-03`'s earlier smoke steps started from a fresh window, which is the one state that already
worked. These deliberately do not.

9. Open Settings, press **Tab once**, then click a keycap and press `Ctrl+Alt+P`. The chord must be
   recorded. Start another capture and press **Escape** — it must cancel.
10. Open Settings, click a **percentage field** (the route that predates this run), then click a
    keycap and press a chord. Same expectation.
11. With nothing capturing, press **Tab** once and then `Ctrl+Alt+Tab`, and read the Key Check
    band. It must report the window as having seen the key — never "another application claimed
    it", which would be a fabricated third party.

## Accessibility — `FR-20`, `FR-21`

12. With a screen reader running, move through a shortcut row. The keycap and the description
    announce **per row** ("Shortcut keycap: Snap to left edge"), not one shared name.
13. **Known open — do not report these as new findings:** the four *Snap to custom* stepper
    controls still share one accessible-name set (`DEF-6`), and no sidebar item or action button is
    keyboard-reachable, so there is no Tab order starting at the navigation tabs (`DEF-7`).

## The three promises with no spec — only if the daemon situation allows

These are daemon-side and this run did not touch them. Attempt them, and expect NOT VERIFIABLE for
the first if elevation blocks you.

14. `FR-8`: press the cycling chord while an elevated window (Task Manager) has focus — focus shifts
    without an OS denial.
15. `FR-9`: the tray icon is present, and the daemon's working set is under the 2 MB its promise
    names.
16. `FR-24`: with the update check enabled, the tray menu offers an update when a newer release
    exists.

## Report back

One verdict per numbered item, in order, each with what you actually observed. Then:

- Which binary each observation came from, and its timestamp.
- Whether Settings ran attached to the stale daemon or standalone, per item where it matters.
- Anything you could not reach, with the specific reason rather than a guess.
- Anything you saw that no item above asked about — that is the most valuable line in this report,
  because everything above is a thing we already suspected.
