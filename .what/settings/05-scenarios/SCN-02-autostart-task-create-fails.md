---
type: scn
id: SCN-02
component: settings
attaches_to: UC-6
created: '2026-08-21'
updated: '2026-08-21'
---

# SCN-02 — Auto-start task creation fails

## Where it branches

Leaves from **UC-6 (Turn auto-start on boot on or off)** after the preference is saved and the daemon has been signalled — the task is registered by the daemon, so the failure surfaces on its side of the boundary, not in Settings.

## Condition

Group Policy, an endpoint protection product, or a restricted account prevents Task Scheduler from creating an `ONLOGON` task, and `schtasks /Create` exits non-zero.

## Flow

1. User turns Auto-start on in the General pane and clicks Save.
2. `LC-config-writer` validates the draft, writes `config.toml` atomically, and posts `WM_APP_RELOAD_CONFIG`.
3. The daemon reloads configuration, sees `general.auto_start` set, and calls `daemon::autostart::enable`.
4. `schtasks /Create` exits non-zero (access denied, or policy block).
5. The daemon logs a warning carrying the exit code. It does not raise a modal: this is a Tier-2 condition, so the tray icon takes the Red Dot overlay and the reason is in `wiradesk.log` (AD-7).
6. `general.auto_start` stays `true` in `config.toml` — the user's *preference* was recorded, and it is the *task* that failed.
7. The tray menu's Auto-start checkmark reads back from `schtasks /Query`, which still reports no task, so the menu shows auto-start off.

## Outcome

No orphaned task exists. The preference and the observable system state disagree, and the tray menu tells the truth about the system rather than about the preference. The user can retry after the policy or permission changes; nothing has to be undone first.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| `schtasks /Create` non-zero | Tier-2 Red Dot on the tray icon; Auto-start unchecked in the tray menu despite the saved preference | `warn!` with the `schtasks` exit code |
| `schtasks.exe` absent or blocked | As above | `warn!` on spawn failure |

## Why it is not in the UC

The success path never crosses the process boundary twice. This scenario is where the split matters: `settings` owns the preference, the daemon owns the task, and only the daemon can see the failure.

## Notes

Settings cannot revert the toggle on this failure, and MUST NOT be specified as doing so — it has already exited, or is at best unaware, by the time `schtasks` runs. `schtasks /Query` is the authoritative source for the checkmark for exactly this reason (`crates/daemon/src/autostart.rs`).
