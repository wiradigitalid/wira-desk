---
wave: W1
release: "0.1.0"
prd: [wira-desk]
status: done
kind: verification
authored_by: coordinator
---

# SPEC — W1, verification of release 0.1.0

## What this SPEC is, and what it is not

A projection of `.what/` and `.how/` onto wave W1. It states nothing those layers do not already
state; where it looks like it is adding something, that is a defect in this file, not a decision.

It is **not a build contract**, because W1 builds nothing. Release `0.1.0` shipped every behaviour in
scope before the wave was opened, and the workspace suite already covers it. What was missing was the
trace: every row of the traceability matrix broke at `story`, so the catalogue was bound to no
evidence at all. W1 supplies that binding and closes.

`bmad-spec` was not dispatched to author this file. Its job is to distil an intent contract for a
builder, and this wave has no builder — dispatching it would have produced a contract for work that
must not happen. This deviation is deliberate and is reported rather than hidden.

## Scope

| Story | Use case | Component | Behaviour under proof |
| --- | --- | --- | --- |
| S1 | UC-1 | `window-management` | Same-application cycling: identity by executable basename, Z-order traversal, monitor and virtual-desktop confinement, VM/RDP passthrough, exact chord matching, anti-macro throttle |
| S2 | UC-2 | `window-management` | Half-screen snap and maximize against the monitor work area, DPI-invariant, degenerate work areas failing without placement |
| S3 | UC-3 | `window-management` | Tray health escalation to Tier 3, one-shot toast latch, restart-free recovery, latched-warning restoration |
| S4 | UC-4 | `settings` | Shortcut capture, canonicalisation, rejection grammar, atomic save, reload signal |
| S5 | UC-5 | `settings` | First-run launch intent, tutorial progression, skip reaching the same terminal state, config written so onboarding does not repeat |
| S6 | UC-6 | `settings` | Auto-start task arguments: `ONLOGON`, `/RL HIGHEST`, `/RU %USERNAME%`, quoted absolute path, pinned task name |

## Invariants this wave asserts are still held

Quoted from `ARCHITECTURE-SPINE.md`; the wave asserts the shipped code does not contradict them, and
names the tests that show it. It does not restate them as new rules.

- **AD-1** — actors own their state; the only channels are the ring buffer, Win32 messages, the TOML
  file, and `ShellExecute`.
- **AD-2** — the hook thread alone throttles, and translates to a `u8` before the ring buffer.
- **AD-3** — no Z-order cache; live `EnumWindows` on every keypress.
- **AD-4** — same application means same executable basename; PID is never the identity.
- **AD-5** — atomic write, then an explicit reload signal; never a file watcher.
- **AD-6** — bypass is evaluated on the hook thread, before interception.
- **AD-7** — three tiers, one startup modal, one toast per Tier-3 death.
- **AD-8** — a 10-second heartbeat, escalating at three consecutive failures.
- **AD-9** — every candidate must be on the current virtual desktop, failing closed.
- **AD-13** — auto-start is a per-user elevated scheduled task, never a `Run` key, never `SYSTEM`.

## Verification

One command, from the repository root:

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
```

The environment variable is required: the daemon links an elevation manifest that would otherwise
apply to the test harness, which then cannot launch at all.

Each story file lists its proving tests by name. Every name was checked against
`cargo test -- --list` when the wave was opened. **A recorded name that no longer resolves is a
finding to raise, never a rename to absorb quietly** — the trace is the whole product of this wave,
so a silently-renamed test destroys exactly what W1 was opened to create.

The wave MUST NOT close on a red suite. A verification wave whose evidence does not pass asserts the
opposite of what it exists to assert, and closing it would put a green traceability row on top of a
failing test. On red the wave stays open and the failure is diagnosed through
`wdi-systematic-debugging` before anything else — including before any test name here is edited.

## Known coverage gaps, carried openly

Two requirements in scope have **no automated test**, and the stories that carry them say so rather
than implying coverage:

| Gap | Why it is not covered | How it is verified instead |
| --- | --- | --- |
| **FR-10** — tray icon returns after Explorer restarts | The observable effect is a shell broadcast landing in another process; the suite proves the state machine around it, not the `Shell_NotifyIconW(NIM_ADD)` call | Manually: end `explorer.exe` from Task Manager and confirm the icon returns without restarting the daemon |
| **FR-12** — View Logs opens the diagnostic log | Spawns `notepad.exe` on the log path; asserting that means asserting on another process | Manually: tray menu, View Logs, confirm the log file opens and is readable while the daemon runs |

Neither is a defect in the code. Both are recorded in the risk register so they are visible to
whoever next decides whether this product needs a runtime harness.

## Out of scope

- Any production code change. A finding that demands one stops this wave and opens another.
- Any edit to `.what/`, `.how/`, or an `applied` `DEC-`.
- FR-8, FR-9, FR-15, FR-16, FR-19, FR-20, FR-21 — each carries a `no_uc:` justification in
  `requirements.yaml`, so they are exempt from the traceability rows rather than unproven. Several
  are covered by the suite regardless (`app::tests::focus_order_*`, `theme::tests::*`,
  `arrangement::stack::tests::*`); binding them would need a use case they deliberately do not have.
