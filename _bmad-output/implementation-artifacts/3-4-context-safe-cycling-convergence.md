---
baseline_commit: 2474934
workflow_id: story-3-4-context-safe-convergence
---

# Story 3.4: Context-Safe Cycling Convergence

Status: in-progress

## Story

As a multi-context desktop user,
I want spatial isolation and VM/RDP passthrough to operate as one reliable cycling pipeline,
so that WinTick changes focus only when the active context permits it.

## Acceptance Criteria

### AC-3.4-001 — Integration without contract change — **DONE**
Frozen spatial and bypass adapters integrated with their public contracts intact. One correction (`Intercept` → `ContinueWinTickMatching`) was returned to its owning story, Story 3.1, rather than aliased at the integration point.

### AC-3.4-002 — Spatial gate over Epic 2 rules — **code complete, unverified at runtime**
Epic 2 ordering and eligibility preserved; only spatially accepted candidates activate; focus unchanged when none qualifies.

### AC-3.4-003 — Monitor and desktop isolation — **code complete, unverified**

### AC-3.4-004 — Whole-chord passthrough — **code complete, unverified**
A bypass context passes the entire chord through `CallNextHookEx` with no ring publication, Worker wake, throttle advancement, or swallow state; coherent through releases even if focus changes mid-chord; no reinjection.

### AC-3.4-005 — Epic 2 contracts unchanged for non-bypass — **code complete**

### AC-3.4-006 — Hung and excluded windows — **inherited from Epic 2**

### AC-3.4-007 — Adapter failure handling — **code complete**

### AC-3.4-008 — Elevated convergence matrix — **NOT DONE**

### AC-3.4-009 — User-facing documentation — **NOT DONE**

## Dev Notes

### The Latch Is the Hard Part

AC-3.4-004 requires passthrough to stay coherent "even if foreground focus
changes during the chord". Evaluating bypass on every key event would break
exactly that: the key-down could pass through while the matching key-up got
swallowed, leaving the guest session with a stuck modifier.

So `HookRuntime::bypass_latched` is set the moment a *matched* chord is found to
be in a bypass context, and every subsequent event returns `PassToNext`
immediately — before any swallow, throttle, or ring logic. The latch clears only
when `ModifierState::any()` reports every modifier released, i.e. the chord is
genuinely over.

Bypass is evaluated **after** `match_shortcut` succeeds, not before. A
non-matching keystroke never pays for an identity query, which keeps the common
path at its Story 2.1 cost.

### Failing Closed Through Missing COM

`VirtualDesktopManager::create()` returns `Option`. `context_allows()` treats
`None` as "no candidate qualifies", so a machine without working COM cycles
nothing rather than cycling unguarded. `missing_com_leaves_focus_unchanged`
pins it.

COM is created per command and dropped at the end of `execute_cycle()`, which
keeps its lifetime on the Worker thread and avoids holding an interface across
commands.

### Filtering Never Reorders

The spatial gate is a `.filter()` over the Epic 2 candidate list; the attempt
order still comes from `cycling::cycle_order`.
`epic_two_ordering_survives_the_spatial_gate` asserts the surviving attempts
remain in Z-order with the rejected candidate simply absent.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 5 integration tests in `worker.rs` using the Story 3.1 fake adapters — fully
  desktop-free.
- Static gates PASS in debug and release.
- **Status `in-progress`.** AC-3.4-008 (elevated convergence matrix) and
  AC-3.4-009 (user-facing documentation) are not done, and the whole pipeline
  depends on Story 3.2's unexecuted COM path.
- The hook-side latch has **never been exercised against real key events**.

### File List
- `crates/daemon/src/hook.rs` (modified — bypass policy, collector, latch, `ModifierState::any`)
- `crates/daemon/src/worker.rs` (modified — spatial gate, `run_context_safe_cycle`)
