---
baseline_commit: f02cda2
workflow_id: story-3-2-spatial-decision-adapter
---

# Story 3.2: Physical-Monitor and Virtual-Desktop Decision Adapter

Status: in-progress

## Story

As a multi-monitor Windows user,
I want WinTick to reject windows outside my current monitor and virtual desktop,
so that cycling preserves the spatial workspace I am currently using.

## Acceptance Criteria

### AC-3.2-001 — Origin monitor and desktop membership — **code complete, UNVERIFIED**
Resolves the origin's physical monitor once per cycle; accepts a candidate only when its `HMONITOR` equals the origin monitor and `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` confirms membership.

### AC-3.2-002 — Live queries, no cache — **code complete**
Monitor and virtual-desktop facts are queried live; nothing is retained as an authoritative cache between cycle commands.

### AC-3.2-003 — COM ownership — **code complete, UNVERIFIED**
COM ownership stays on the Worker actor's thread; initialization, interface lifetime, and release are explicit; raw COM is limited to the documented `IVirtualDesktopManager` exception.

### AC-3.2-004 — Fail closed — **code complete**
On monitor lookup, COM initialization, or virtual-desktop query failure the adapter fails closed without selecting the candidate, exposes one diagnostic outcome for the Tier-2 path, and never shows a popup or causes a cross-context focus change.

### AC-3.2-005 — Isolated spatial tests — **partially covered**
Fake adapters cover same/different monitor and unavailable cases (in Story 3.1's harness). Windows integration fixtures are **not** implemented.

### AC-3.2-006 — Independent verification — **NOT DONE**
Unit, adapter, COM-lifecycle, and Windows spatial-fixture gates have never been executed.

## Dev Notes

### `MONITOR_DEFAULTTONULL` Is the Whole Point

`MonitorFromWindow` is called with `MONITOR_DEFAULTTONULL`, not
`MONITOR_DEFAULTTONEAREST`. A window intersecting no monitor must report
*unknown* so the frozen contract fails closed. `DEFAULTTONEAREST` would
silently attribute it to a monitor and let cycling jump workspaces — the exact
failure this epic exists to prevent.

### Hand-Rolled COM

`windows-sys` 0.52 ships no `IVirtualDesktopManager` binding, so the vtable is
declared by hand: three `IUnknown` slots (`QueryInterface`, `AddRef`,
`Release`) followed by the three interface slots, in documented order. Getting
that order wrong would call the wrong function pointer, so
`vtable_has_six_slots_in_documented_order` pins the layout size and the CLSID
and IID are asserted nibble-by-nibble — a transcription error there surfaces at
runtime as an opaque `REGDB_E_CLASSNOTREG`.

### Apartment Ownership Is Tracked, Not Assumed

`CoInitializeEx` may return `RPC_E_CHANGED_MODE` when the thread already has an
apartment in another mode. The interface is still usable, but calling
`CoUninitialize` would then unbalance someone else's reference count. The
`owns_apartment` flag records which case occurred and `Drop` only uninitializes
what it initialized.

`VirtualDesktopManager` holds a raw pointer, which makes it `!Send`/`!Sync`
automatically — the compiler, not a comment, enforces "COM ownership remains on
the Worker actor's thread".

### `UnavailableVirtualDesktops`

A null-object adapter returning `None` for everything, so a COM-less
environment degrades to "nothing is eligible" rather than to unguarded cycling.

## Dev Agent Record

### Agent Model Used
claude-opus-5 (Claude Code)

### Completion Notes List
- 8 unit tests across `spatial.rs` and `virtual_desktop.rs`.
- `Win32_System_Com` added to the daemon's `windows-sys` features.
- Static gates PASS in debug and release.
- **Status held at `in-progress`, not `review`.** This is the highest-risk code
  in the project: a hand-declared COM vtable that **has never been executed**.
  The tests that exist cover GUID transcription, vtable shape, and the
  fail-closed path — none of them actually create the COM object. A wrong
  vtable slot, a wrong calling convention, or an apartment mistake would only
  appear at runtime, potentially as a crash.
- Windows spatial integration fixtures (AC-3.2-005) are not implemented; they
  need a live multi-monitor elevated desktop.

### File List
- `crates/daemon/src/context/spatial.rs` (new)
- `crates/daemon/src/context/virtual_desktop.rs` (new)
- `crates/daemon/Cargo.toml` (modified — `Win32_System_Com` feature)
