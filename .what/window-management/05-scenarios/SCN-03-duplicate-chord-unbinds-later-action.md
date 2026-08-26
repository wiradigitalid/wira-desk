---
type: scenario
id: SCN-03
component: window-management
relates_to: [UC-2, UC-7]
created: '2026-08-26'
---

# SCN-03 — A configuration file binds two actions to one chord

## Why this scenario exists

It is the upgrade path, not an edge case. `snapping.snap_half_bottom` arrives with the shipped default
`ctrl+alt+down`. Any `config.toml` written before that field existed, which already sets
`layout.stack_shortcut = "ctrl+alt+down"`, becomes a colliding configuration on the first launch after
upgrade — with no user action involved, and with no route through the settings process, which would have
refused to save such a pair.

Two answers, chosen by whether a last-known-good configuration exists. `BR-6` states the rule; `DEC-009`
states why it is not one uniform answer.

## Flow — at startup

| Step | What happens |
| --- | --- |
| 1 | Daemon starts and reads `config.toml`. Missing fields take their shipped defaults, so a legacy file gains `snap_half_bottom` without being rewritten. |
| 2 | System parses every configured chord and finds two fields resolving to the same chord. |
| 3 | System keeps the chord for whichever field comes first in the fixed precedence order — the declaration order of the fields. |
| 4 | System leaves the later field **unbound**. Its action has no chord that reaches it, and no other action fires in its place. |
| 5 | System emits exactly **one** Tier-2 warning naming both fields and the chord, and raises the tray icon to its Warning state. |
| 6 | Daemon continues starting. Every other setting in the file — switcher chord, VM bypass list, auto-start — is untouched. |

**On the collision this upgrade actually creates**, `snap_half_bottom` precedes `stack_shortcut`, so snapping to
the bottom half wins and the overlapping stack goes dark. That is the least defensible consequence of the rule
and `DEC-009` records it as such rather than hiding it.

## Flow — on an explicit reload

| Step | What happens |
| --- | --- |
| 1 | Settings posts `WM_APP_RELOAD_CONFIG` after an atomic write, or a user has hand-edited the file and something triggers the reload. |
| 2 | System validates the candidate configuration and finds a duplicate chord. |
| 3 | System refuses the **whole** candidate. No actor receives a snapshot; every actor stays on its last-known-good configuration. |
| 4 | System emits one Tier-2 warning saying the reload was skipped and current settings were kept. |

Nothing is partially applied, because a half-applied reload leaves the user unable to tell which half took
effect — the reason the all-or-nothing reject contract exists. A duplicate arriving here means the file was
hand-edited, since the settings process refuses to write one.

## What the user sees, and what they do not

They see the tray icon's Warning dot, and — only if they open the log — a line naming both fields. From the
keyboard, an unbound action and a broken feature are indistinguishable. `DEC-009` accepts this silence and
names the route out of it: the settings process showing the collision on the fields that collide, which is
`DEC-001`'s shape already built and merely not reached from a daemon-side detection.

## Business Rules

- `BR-6` (One chord, one action, answered differently on each side)
- `LBR-WM-1` (Exact shortcut matching only)
