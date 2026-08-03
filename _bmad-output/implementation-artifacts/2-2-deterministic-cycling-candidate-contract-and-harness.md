---
baseline_commit: e55e34e
workflow_id: story-2-2-deterministic-cycling-contract
supersedes: 2-2-stateless-z-order-and-app-specific-matching.md
---

# Story 2.2: Deterministic Cycling Candidate Contract and Harness

Status: review

## Story

As a WinTick user,
I want every cycling component to interpret window candidates and failures through one deterministic contract,
so that parallel implementation cannot change which window is considered next.

## Acceptance Criteria

### AC-2.2-001 — Contract shape and domain boundary

- **Given** the frozen Story 2.1 Worker-drain boundary
- **When** the cycling contract is defined
- **Then** it represents an ordered candidate, captured window facts, active-window identity, eligibility decision, selection result, and activation result without shared mutable state
- **And** these types remain internal to the daemon Worker domain rather than expanding the cross-crate `shared` API

### AC-2.2-002 — Application identity normalization

- **Given** a process path is available for a window
- **When** application identity is normalized
- **Then** identity is the case-insensitive executable basename
- **And** PID is used only to query the process and never as the primary same-application identity
- **And** an inaccessible or vanished process produces an unavailable identity without a crash or runtime popup

### AC-2.2-003 — Single-sample active context and one-pass termination

- **Given** the Worker begins processing one accepted `Command::Cycle`
- **When** the contract captures the active context
- **Then** `GetForegroundWindow` is sampled once at the start of that command
- **And** candidate order represents one fresh top-to-bottom Z-order snapshot
- **And** activation may attempt each eligible target at most once before terminating

### AC-2.2-004 — Frozen eligibility vocabulary

- **Given** eligibility policy needs deterministic vocabulary
- **When** the contract fixtures are frozen
- **Then** hidden, iconic, `WS_EX_TOOLWINDOW`, `Ghost`, `Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `Progman`, and `WorkerW` cases have explicit expected decisions
- **And** a real application window marked Not Responding remains eligible without a responsiveness probe

### AC-2.2-005 — Deterministic harness without desktop dependency

- **Given** discovery, eligibility, and activation implementations do not yet exist
- **When** the deterministic harness runs
- **Then** injected snapshots and fake activation outcomes verify ordering, executable normalization, exclusion decisions, wrap behavior, invalid-target continuation, and one-pass termination
- **And** the harness requires neither a global hook nor a live desktop

### AC-2.2-006 — Parallel lane ownership

- **Given** Story 2.2 has passed its contract and harness gate
- **When** parallel implementation begins
- **Then** live discovery, eligibility policy, and selection/activation use separate workflow IDs and Git worktrees
- **And** discovery exclusively owns `cycling/source.rs`
- **And** eligibility exclusively owns `cycling/eligibility.rs`
- **And** selection and activation exclusively own `cycling/selection.rs` and `cycling/activation.rs`
- **And** `cycling/mod.rs`, `worker.rs`, shared-contract close work, and final composition remain reserved for convergence
- **And** each lane completes its own Codex-to-Cursor-to-Antigravity evidence chain before convergence eligibility

## Dev Notes

### Contract Design

The whole contract lives in `crates/daemon/src/cycling/mod.rs` and performs **zero
Win32 calls**. That is what makes AC-2.2-005 achievable: the harness runs on
injected data, so no global hook and no live desktop are required.

Types:

| Type | Role |
| --- | --- |
| `WindowId(isize)` | Opaque handle; a plain integer so fixtures need no Win32 |
| `AppIdentity` | `Executable(String)` (normalized basename) or `Unavailable` |
| `WindowFacts` | Immutable per-window facts from one snapshot |
| `Candidate` | `z_index` (0 = topmost) + `WindowFacts` |
| `ActiveContext` | Foreground `WindowId` + its identity, sampled once |
| `Eligibility` / `ExclusionReason` | Decision vocabulary |
| `SelectionResult` | `Target(WindowId)` or `NoCandidate` |
| `ActivationOutcome` | `Activated` / `InvalidTarget` / `Failed` |
| `CycleOutcome` | `Activated(WindowId)` / `Exhausted` / `NoEligibleTarget` |

Seams for the parallel lanes are three traits: `CandidateSource` (Story 2.3),
`EligibilityPolicy` (Story 2.4), and `Activator` (Story 2.5). `run_cycle` is the
deterministic driver that composes them.

### Two Decisions Worth Recording

**PID is absent from the contract by construction.** AC-2.2-002 says PID must
never be the primary same-application identity. Rather than carry a PID field and
rely on reviewers to never compare it, the field simply does not exist —
`AppIdentity` holds only the normalized basename. Story 2.3 may use a PID
transiently to *query* the process, but it cannot leak into identity.

**Responsiveness is absent for the same reason.** AC-2.2-004 requires a
Not Responding window to stay eligible *without a responsiveness probe*. A
`responding: bool` field would invite exactly the probe the AC forbids, so the
contract omits it. The hung-window case is still covered: Windows creates a
separate `Ghost` class window, which the frozen fixtures exclude, while the real
application window remains eligible.

### `AppIdentity::Unavailable` Never Matches Itself

`same_application()` returns `false` when either side is `Unavailable`, including
`Unavailable` vs `Unavailable` — two unknown processes are not *known* to be the
same application. `PartialEq` is derived for test assertions only; production
same-application checks must use `same_application()`. This is documented on the
method because `==` is the easy wrong call here.

### Wrap Order

`cycle_order()` returns the snapshot rotated to start *after* the active window,
wrapping to the front. When the active window is not in the snapshot the order is
returned unchanged, so the first eligible candidate becomes the next target. The
active window is never re-attempted within a pass, which is what bounds the pass
to "each eligible target at most once".

### Scope and Non-Goals

- No `EnumWindows`, no `GetForegroundWindow`, no `SetForegroundWindow` — those
  belong to Stories 2.3 and 2.5.
- `worker.rs` is **untouched**; its `Command::Cycle` placeholder remains. Wiring
  the contract into the Worker is reserved for Story 2.6 convergence per
  AC-2.2-006.
- No `shared` crate change. The contract is daemon-internal.
- `cycling/source.rs`, `eligibility.rs`, `selection.rs`, `activation.rs` are
  deliberately **not created** — they are owned by the downstream lanes.

### Testing Requirements

22 unit tests in `cycling/mod.rs`, all desktop-free:

- executable normalization (5): case folding, both separators, bare basename,
  empty/trailing-separator degradation, multi-process grouping
- identity semantics (1): `Unavailable` never matches
- frozen vocabulary (2): all 12 fixtures match expected decisions; every named
  shell class has a fixture
- ordering and wrap (4)
- full-pass harness (10): activation, exclusion skipping, invalid-target
  continuation, failed-activation continuation, one-pass termination, no-eligible
  -target, single-window application, foreign-application rejection

`fixtures::expected_decisions()` is the frozen table Story 2.4's production policy
must reproduce; `fixtures::ReferencePolicy` exists only to drive this harness and
is not production code.

## Dev Agent Record

### Agent Model Used

claude-opus-5 (Claude Code)

### Completion Notes List

- Contract, driver, fixtures, and harness implemented in a single new file
  `crates/daemon/src/cycling/mod.rs`; `mod cycling;` added to `main.rs`.
- `#![allow(dead_code)]` is applied at module level because the contract is
  published ahead of its consumers. Same precedent as the Story 1.5
  capability-only `log::warn` seam. Lanes 2.3–2.5 consume it.
- **Static gates PASS**: `cargo build -p daemon` clean; `cargo clippy -p daemon
  --all-targets` exit 0 with no new warnings; `cargo test -p daemon --no-run`
  compiles the test binary, so all 41 tests type-check.
- **Test execution NOT performed.** `cargo test -p daemon` requires an elevated
  session (`os error 740`, `requireAdministrator` manifest) — the same
  pre-existing constraint as Stories 1.2/1.4/1.5. Deferred to the user at their
  request. Expected total after this story: **41 unit tests** (19 baseline + 22).
- Pre-existing clippy warning `collapsible_if` at `hook.rs:574` left untouched:
  it belongs to Story 2.1, whose chain is `CLOSED`.

### File List

- `crates/daemon/src/cycling/mod.rs` (new)
- `crates/daemon/src/main.rs` (modified — `mod cycling;`)

### Change Log

| Date | Change |
| --- | --- |
| 2026-07-26 | Story created from restructured `epics.md` and implemented; status `review` pending elevated test execution. |
