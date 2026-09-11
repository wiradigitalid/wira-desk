---
id: SPEC-9-01
component: window-management
satisfies: [UC-13]
blocked_by: []
status: ready-for-agent
tests:
  - log::tests::shared_log_rotation_caps_file_at_1mb
  - util::tests::debug_trace_rotates_at_cap_when_debug_assertions_active
---

# 01: Debug trace log rotation and AppData inventory documentation

**What to build:** Extract a reusable file rotation helper (`rotate_at_cap(path, max_bytes)`) based on the existing rotation logic in `crates/daemon/src/log.rs`. Apply this helper to `append_debug_trace` in `crates/daemon/src/util.rs` so that `wiradesk-debug-trace.log` is capped at 1 MB with a single `.old` backup generation (matching `wiradesk.log`'s bounded ~2 MB footprint). Document the complete `%APPDATA%\WiraDesk` file lifecycle in `CONTRIBUTING.md` / `docs/`.

**Blocked by:** None

## Acceptance Criteria

- [ ] Extracted helper `rotate_at_cap(path: &Path, max_bytes: u64)`:
      - If file size exceeds `max_bytes`, renames `path` to `<path>.old`, overwriting any prior `.old` generation.
      - Handles non-existent files gracefully without errors.
- [ ] `crates/daemon/src/log.rs` refactored to use `rotate_at_cap` with `LOG_MAX_BYTES = 1_000_000`.
- [ ] `crates/daemon/src/util.rs::append_debug_trace` checks `rotate_at_cap` before appending lines in `#[cfg(debug_assertions)]` builds.
- [ ] In production release builds (`build.ps1 -Mode prod`), `append_debug_trace` remains compiled out.
- [ ] Unit tests verify:
      - Appending lines past 1 MB triggers rotation to `.old`.
      - Subsequent writes past 1 MB replace `.old` rather than creating a third generation.
- [ ] Add an "AppData Files Inventory" section in `docs/` describing `config.toml`, `wiradesk.log`, and `wiradesk-debug-trace.log`.
