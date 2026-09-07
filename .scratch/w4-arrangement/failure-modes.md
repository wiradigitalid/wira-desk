# Failure modes — what happens when the other side is slow, absent, or lying

Projected from `.how/window-management/SDD-window-management.md` § Failure Behaviour, `UC-7`,
`SCN-03`, and `DEC-009`. Nothing here is new.

"Returns an error" is not an answer anywhere in this table.

## Monitor enumeration

| Condition | What the system does | What the user sees | What is logged |
|---|---|---|---|
| Enumeration callback runs long on a many-display machine | Nothing special. It is off the hook thread, so the input path is unaffected | Nothing | Nothing |
| No monitor reported, or the foreground window resolves to no monitor (`MONITOR_DEFAULTTONULL`) | Plans nothing. The chord was already consumed | Nothing moves. No popup | One Tier-2 warning naming the failing query |
| **Exactly one monitor attached** | Returns an **empty plan**, which is a success | Nothing moves, nothing is shown | **Nothing.** This is a successful no-op, not a failure, and logging it would train the user to ignore the log |
| A monitor is reported and unplugged before placement | Attempts the placement; Windows refuses or clamps it | Window lands on an attached monitor rather than off-screen; the next press moves it again | One Tier-2 warning |
| Destination work area is empty or unrepresentable | Reports a planning failure, emits no placement | Window stays exactly where it was | One Tier-2 warning |

## Placement

| Condition | What the system does | What the user sees | What is logged |
|---|---|---|---|
| `SetWindowPos` refused — target privilege or style lock | Logs and continues; no crash | Window remains at its current position and size | One Tier-2 warning |
| Foreground window belongs to Wira Desk itself | Resolves no target. Chord stays consumed rather than passed back to Windows | Nothing moves. No popup | Tier-2 diagnostic (`LBR-WM-6`, `DEC-006`) |
| Window enforces a minimum size larger than the planned half | Positions it flush to the edge, honouring the application's minimum | A window wider than half, aligned to the edge | Nothing |
| Window is maximized | **Restores to normal first**, then places | The expected result, reached by restore-then-place | Nothing |
| Work area too small to divide on the requested axis | Refuses as a planning failure rather than emitting a zero-extent placement | Nothing moves | One Tier-2 warning |
| Source and destination scaling differ | Places anyway; the frame lands a few pixels off | Correct monitor, correct share, edge slightly off | **Nothing.** Nothing failed; `DEC-007` accepts the imprecision |

## Duplicate chord configuration

The asymmetry is deliberate and `DEC-009` states why: at startup there is no last-known-good
configuration to keep, and at reload there is.

| Path | What the system does | What the user sees | What is logged |
|---|---|---|---|
| **Startup** (`load_shortcuts`) | Earlier field in the declared sequence keeps the chord; later field is left **unbound**. Every unrelated setting still takes effect | The later action's chord does nothing. Tray goes to its Warning state. No popup | **Exactly one** Tier-2 warning naming **both** fields and the chord. Never one per field, and never silence |
| **Reload** (`config::validate`) | Refuses the **whole** candidate. No actor receives a snapshot | Previous configuration stays in force | One Tier-2 warning saying the reload was skipped and current settings were kept |

An unbound action matches nothing, and **no other action fires in its place**. From the keyboard,
an unbound action and a broken feature are indistinguishable — `DEC-009` accepts that silence and
names showing it in Settings as the route out, which this wave does not build.

A duplicate arriving on the reload path means the file was hand-edited: the settings process refuses
to save one, so it cannot have produced it.

## Ring buffer and hook, unchanged by this wave

Listed so a reader does not assume the new commands changed them.

| Condition | Behaviour |
|---|---|
| Ring buffer full (>16 unread commands) | Keypress dropped silently, no freeze; drop count recorded |
| Chord pressed within the 50 ms throttle | Dropped on the hook thread |
| Modifier superset of a configured chord | Passes through via `CallNextHookEx`; never triggers an action (`LBR-WM-1`) |
