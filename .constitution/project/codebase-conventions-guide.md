---
status: Accepted
ratified_by: 67f2645     # the last commit that changed `crates/` — the code this file describes
---

# conventions — codebase guide

**Loaded when:** writing or reviewing code.

Filled by the distillation of wave W1 from the code at `67f2645`. `.how/_platform/ARCHITECTURE-SPINE.md` holds the numbered
invariants these rules enforce, and `docs/decisions.md` the reasoning that predates this repository; this file holds the conventions a new
change has to follow. Where the two overlap, `docs/decisions.md` says *why* and this says *what*.

## Naming

| Thing | Convention |
| --- | --- |
| Rust modules and files | `snake_case` |
| TOML config keys | `snake_case` — `snap_half_left`, `bypass_processes` |
| Binaries | `wiradesk.exe`, `wiradesk-settings.exe` |
| Win32 messages | `WM_APP_<VERB>_<NOUN>`, declared as `WM_APP + N` and never as a bare hex literal |
| Tests | A full sentence naming the behaviour, not the function — `halves_tile_the_work_area_without_gap_or_overlap`, not `test_snap` |

The test-naming convention is load-bearing rather than stylistic: the traceability rows in
`waves.yaml` cite tests by name as the evidence a promise is kept, so a name that describes the
behaviour makes the trace readable and a name like `test_snap_2` makes it worthless.

## Threads own their state

The actor rule is the whole architecture, so it is also the whole review checklist. Cross-actor
traffic uses exactly four channels and no fifth: the lock-free ring buffer (hook → worker), Win32
window messages, the TOML file, and `ShellExecute`. A `Mutex` shared between the hook thread and the
worker is not a small deviation — it is the design being abandoned.

Two rules follow, and both have teeth:

- **Nothing allocates on the hook thread.** The bypass check uses reusable `[u16; 256]` and
  `[u16; 260]` stack buffers for exactly this reason. Windows enforces `LowLevelHooksTimeout`, so a
  slow callback is not slow — it is silently unhooked, and the user's shortcuts stop working with no
  error anywhere.
- **`EnumWindows` callbacks use only non-blocking APIs**: `IsWindowVisible`, `GetWindowLongPtrW`,
  `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`, `GetClassNameW`. Never `SendMessage`,
  never `GetWindowText` — either will block on a hung window, which is the exact case the product
  promises to handle honestly.

## Failing closed, and the one place that fails open

Context checks fail **closed**: a window whose monitor or virtual desktop cannot be determined is
treated as ineligible, so cycling never leaves the desktop it started on. The VM/RDP bypass fails
**open**: an unresolved foreground window is not treated as a bypass target, so the shortcut is still
handled by Wira Desk.

The asymmetry is deliberate and should not be "made consistent". Failing closed on a context check
costs the user one skipped window; failing closed on the bypass would swallow a keystroke meant for a
guest OS.

## Errors

Three tiers, and the tier decides the surface. The spine's error-handling invariant owns this:

1. **Startup fatal** — exactly one `MessageBox`, then exit. No retry.
2. **Runtime warning** — the log, silently, plus the tray's red-dot overlay.
3. **Runtime critical** — the red-X overlay plus exactly one toast, latched by
   `hook_dead_toast_sent`.

**Never a runtime popup, and never more than one startup popup.** A latch that stops a notification
repeating is not an optimisation; it is the feature.

## Testing

From `docs/decisions.md`, which owns the reasoning:

- **Never assert on a shared global from a parallel test.** Statics are visible to every test thread;
  this produced a flaky failure once already. Inject the state, or serialise the tests that touch it.
- **Pin a behaviour against the same constant the production path uses**, never a value the test
  recomputes. A test that rebuilds its own copy of a flag set can never disagree with itself, which
  makes it look like a guard while guarding nothing.
- **A test that passes for the wrong reason is worse than a missing test.** "Never jumps to another
  application" passes trivially when the run produced no activation at all. Such an assertion needs a
  precondition that the interesting path was actually reached.
- Percentiles use nearest-rank, not interpolation, so a reported figure is always a sample that was
  really observed.

## Unsafe

Every `unsafe` block carries a `SAFETY:` comment stating the precondition it relies on — enforced by
`undocumented_unsafe_blocks = "deny"`, so it is a compile error rather than a review note. See
`codebase-stack-guide.md` for what a useful one contains.

## What a change must not do

- Cache Z-order, or any window state, between keypresses. Traversal is live on every keypress.
- Use a PID as application identity. Identity is the executable basename, and only that.
- Watch the config file. The daemon reloads on an explicit message and nothing else.
- Register auto-start through `HKCU\...\Run`, or run the task as `SYSTEM`. It is a per-user
  elevated scheduled task.
- Widen `scripts/verify-public-export.ps1` to silence a finding. Fix the content; that file exists
  because widening was tried and shipped a problem for months.
- Edit the `.what/` or `.how/` corpus, or an applied decision record, to make code fit. That
  deviation is reported, and it becomes a decision of its own.
