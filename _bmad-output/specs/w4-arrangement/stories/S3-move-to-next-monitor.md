---
id: "S3"
status: done
component: window-management
commit: 52a0f87
provenance: >-
  As-built record, written by the coordinator at wave close. This story did NOT go through
  the five-step pipeline (plan / build / panel / publish / CI): the owner redirected the wave
  to the main checkout on `main` and the code was written by the coordinator directly, so no
  planning step produced this file at `status: ready-for-dev` and no code-review panel ran.
  It records what landed, not what was planned.
---

# S3 - Move the active window to the next monitor

## What landed

New `arrangement/monitor.rs` with `next_monitor_index` and `plan_move_to_monitor` - pure geometry, proportional mapping, source-clamped before mapping. Live enumeration in `context/spatial.rs` (`EnumDisplayMonitors`, per-monitor work area and DPI, nothing cached). `monitor_dpi` in `arrangement/win32.rs`. Wire value 8, config field in `[layout]`, `execute_monitor_move` on the worker with a restore-if-maximized step. The border clamp now resolves its monitor from the planned rect rather than from the window, per `DEC-010`.

## Verification

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace` - 417 tests, +17. Includes a round trip that must land where it started, a sweep asserting no plan escapes the destination work area, and an explicit assertion that absolute size is *not* preserved.

## Not done, and why

`EnumDisplayMonitors`, `GetMonitorInfoW`, and `GetDpiForMonitor` are not covered by a test - they need a real desktop, and every machine available here has one monitor, so only the single-monitor no-op branch runs in practice. The cross-DPI frame-inset imprecision is accepted rather than fixed (`DEC-007`).
