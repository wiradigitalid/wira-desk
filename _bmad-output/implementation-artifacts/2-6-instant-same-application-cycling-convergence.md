---
baseline_commit: e4a6703
workflow_id: story-2-6-epic-2-convergence
---

# Story 2.6: Instant Same-Application Cycling Convergence

Status: in-progress

## Story

As a WinTick user,
I want the accepted hook, discovery, filtering, selection, and activation capabilities to operate as one instant cycle,
so that one exact shortcut predictably focuses the next real window of my active application.

## Acceptance Criteria

### AC-2.6-001 — Composition — **code complete, unverified**
Worker composes fresh discovery, pure eligibility, stateless selection, and bounded activation behind `Command::Cycle`.

### AC-2.6-002 — Identity and Z-order behavior — **unverified**
Cycling stays within the active executable identity across PIDs; repeated commands follow live Z-order; modifier mismatches do not trigger cycling.

### AC-2.6-003 — Mixed-desktop resilience — **unverified**
Excluded windows never become targets; the real hung window receives a bounded attempt; invalid targets are skipped once without crash, popup, or infinite loop.

### AC-2.6-004 — Visual invisibility — **code complete, unverified**
No overlay, preview, animation, transition surface, restore, or maximize; only the native focus change is visible.

### AC-2.6-005 — NFR10 performance — **harness implemented, NOT EXECUTED**
≥1,000 accepted cycles measured Worker-receipt-to-activation; p50/p95/max recorded separately from hook-callback timing; p95 < 1 ms.

### AC-2.6-006 — 30-minute soak — **harness implemented, NOT EXECUTED**
≥10,000 exact-match events; accepted/throttled/dropped/drained/activated counts reconciled; no unexplained dropout.

### AC-2.6-007 — Frozen extension contract — **DONE, verified**
`u8` values for `Cycle`/`SnapLeft`/`SnapRight`/`SnapMaximize`/`OverlappingStack` stable; snapping defaults unchanged; `layout.stack_shortcut` introduced with default `ctrl+win+down`; `vm_bypass.bypass_processes` defaults retained; backward-compatible `vm_bypass.bypass_classes` introduced with `VMwareUnityWindow`.

### AC-2.6-008 — Release and resource regression — **harness implemented, NOT EXECUTED**
Idle CPU ≈ 0; idle RAM < 2 MB target, < 10 MB hard limit; release binary 250–400 KB, < 500 KB; Epic 1 gates green.

## Dev Notes

### Composition

`worker.rs::execute_cycle()` samples the active context **once** via
`capture_active_context()`, then hands it to `run_cycle` together with
`Win32CandidateSource`, `WindowEligibility`, and `Win32Activator`. The Story 2.2
driver owns the loop, so convergence added no new control flow — it only wired
the four accepted lanes together.

All three terminal outcomes are silent in release builds. The `match` on
`CycleOutcome` is `#[cfg(debug_assertions)]`-gated and writes only to the debug
trace, so AC-2.6-004 holds by construction: there is no user-facing surface in
this path at all.

Sibling-lane code was not reopened.

### Frozen Extension Contract

Seven contract tests in `shared` pin the freeze so Epics 3–5 can consume but not
renumber it: command wire values, snapping defaults, `stack_shortcut` default,
`bypass_processes` defaults, `bypass_classes` default, and two legacy-config
round-trips proving a pre-freeze `config.toml` still loads and silently gains the
new defaults.

`bypass_classes` is additive with `#[serde(default)]`, so backward compatibility
is structural rather than conventional.

### Measurement Seam and Why a New One Was Needed

`crates/daemon/src/metrics.rs` (debug builds only) records cycle latency and the
reconciliation counters. Latency is measured **inside** `execute_cycle()` from
Worker command receipt through activation completion, using the daemon's own
QPC clock — so the reported numbers are not inflated by PowerShell or IPC
overhead. It is emitted under `CYCLE_LATENCY`, deliberately distinct from
`HOOK_LATENCY`, because NFR10 requires the two distributions reported
separately.

Percentiles use **nearest-rank**, not interpolation: a reported p95 is always an
observed sample rather than a synthesized value that never occurred.

The existing `WM_APP_DEBUG_SIMULATE_SHORTCUT` seam could **not** be reused for
this. It resets the throttle and drains the ring on every invocation — correct
for Story 2.1's single-shot checks, but at 1,000 iterations it would discard
commands the Worker had not yet drained and report them as dropouts. A new
`WM_APP_DEBUG_CYCLE_BURST` (`WM_APP + 27`) publishes and drains one command per
iteration so the ring never backs up.

**Honest limitation:** the burst seam exercises the Worker path, which is what
NFR10 measures, but it does not exercise the Hook→Worker transport. `ACCEPTED`,
`THROTTLED`, and `DROPPED_FULL` are Hook-side counters and stay at zero under
burst alone; they populate only from real or simulated key events. The soak mode
interleaves `SIMULATE_SHORTCUT` for that reason, but full transport-level
reconciliation still needs real keyboard input.

### Counter Placement in the Hook Callback

`handle_key_event` now distinguishes throttle rejection from capacity rejection
by splitting the previous short-circuit `&&`. The counters are atomic
increments under `#[cfg(debug_assertions)]` — no allocation, no lock, no
logging — so the NFR5/AC-2.1-003 bounded-callback guarantee is preserved and
release builds are untouched.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- **`cargo test -p shared` PASS 18/18** — really executed. This covers AC-2.6-007
  in full; it is the only Epic 2 acceptance criterion with genuine runtime
  evidence.
- Static gates PASS: `cargo clippy --workspace --all-targets` exit 0 with no new
  warnings; `cargo test --workspace --no-run` compiles all three test binaries.
- `verify-story-2-6-convergence.ps1` implements the AC-2.6-005/006/008 gates
  (parse-checked; **never executed** — needs an elevated live desktop).
- **Status held at `in-progress`, not `review`.** The harness exists but has
  produced no numbers. Until it runs, p95, idle RAM, and dropout reconciliation
  are unknown, and the runtime behaviour of AC-2.6-001..004 has never been
  observed. Marking this `review` would misrepresent an unmeasured integration
  as ready.

### Methodology Assumptions (stated, not agreed)

These were chosen to unblock implementation and are the script's defaults. They
are the decisions previously flagged as needing agreement:

| Choice | Default | Rationale |
| --- | --- | --- |
| Warm-up | 200 cycles, discarded | Excludes first-touch page faults and lazy QPC frequency caching |
| Measurement | 1,000 cycles | AC minimum; raise with `-Cycles` |
| Percentile | Nearest-rank | Reported values are observed samples |
| Idle CPU window | 10 s settle + 10 s sample, < 100 ms CPU | "Approximately zero" made falsifiable |
| RAM source | `WorkingSet64` | Directly comparable to Task Manager |
| Soak cadence | 500-cycle batches every 5 s + interleaved shortcut | Reaches 10,000 events well inside 30 min while leaving idle gaps |

Override any of them via parameters if you disagree.

### File List
- `crates/daemon/src/metrics.rs` (new — debug-only measurement seam)
- `crates/daemon/src/worker.rs` (modified — cycle composition + latency/outcome counters)
- `crates/daemon/src/hook.rs` (modified — throttle/capacity counters)
- `crates/daemon/src/tray.rs` (modified — metric message handlers + burst seam)
- `crates/daemon/src/main.rs` (modified — `mod metrics`)
- `crates/shared/src/constants.rs` (modified — `WM_APP + 25/26/27`)
- `crates/shared/src/config.rs` (modified — `stack_shortcut`, `bypass_classes`, 7 contract tests)
- `crates/shared/src/commands.rs` (modified — frozen wire-value test)
- `verify-story-2-6-convergence.ps1` (new — elevated runtime harness)
