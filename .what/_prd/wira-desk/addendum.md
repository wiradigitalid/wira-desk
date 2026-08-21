---
type: prd-addendum
initiative: wira-desk
updated: 2026-08-21
---

# PRD Addendum — Wira Desk

## ID Mapping Table

This table survives the retirement of `_bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/`.

| Archive ID | Corpus ID | Notes |
| --- | --- | --- |
| FR-1 … FR-21 | FR-1 … FR-21 | Renumbering unchanged; component ownership mapped in `requirements.yaml` |
| CAP-1 … CAP-11 | CAP-1 … CAP-11 | Unchanged |
| AD-1 … AD-12 | AD-1 … AD-12 | Architectural spine in `.how/_platform/ARCHITECTURE-SPINE.md` |
| WinTick product name | Wira Desk | Settings path migrates from `WinTick` to `WiraDesk` |
| `wintick.exe` | `wiradesk.exe` | Main background daemon container (`daemon`) |
| `wintick-settings.exe` | `wiradesk-settings.exe` | Settings UI container (`settings`) |

## Solution Shape Hints

- **Dual-Thread Actor Model**: Dedicated low-level hook thread (`WH_KEYBOARD_LL`) communicating with a worker thread via a lock-free u8 command ring buffer to ensure hook processing completes well within the OS `LowLevelHooksTimeout` (300 ms).
- **Sterilized Win32 API Boundaries**: Asynchronous window interrogation and stateless Z-order queries avoid cascading hangs when targeting unresponsive ("Not Responding") applications.
- **Pure Win32 Tray Interface**: System tray and context menus implemented directly with raw Win32 APIs rather than heavier GUI frameworks to maintain minimal RAM overhead (<15 MB).
- **Atomic Config Reload**: TOML configuration files written atomically to disk and hot-reloaded via `WM_APP_RELOAD_CONFIG` messages.
- **Rust Toolchain & Footprint**: Standard library (`std`) retained alongside `windows-sys` C-FFI bindings and aggressive profile optimisation (LTO, strip, `opt-level = "z"`) to deliver binaries under 500 KB without compromising concurrency safety.
- **Migration Artifacts**: Scheduled task name and single-instance mutex updated for Wira Desk branding during migration.

## Rejected Alternatives

| Rejected Option | Alternative Chosen | Rationale |
| --- | --- | --- |
| `RegisterHotKey` API | `WH_KEYBOARD_LL` (Low-Level Keyboard Hook) | Operates on a first-come-first-served basis; if a third-party application registers a hotkey first, registration fails. Low-level hooks guarantee deterministic, top-priority input capture. |
| `#![no_std]` Rust Runtime | Rust `std` + `windows-sys` | Removing `std` saves only ~50–100 KB but breaks built-in thread safety and forces raw C-FFI synchronization. Binary size targets (<500 KB) are met using `std` with `windows-sys` and compiler release profiles. |
| `windows` crate (COM abstractions) | `windows-sys` crate (raw Win32 C-FFI) | COM wrappers and heavy metadata in `windows` bloat binary size and compilation times. `windows-sys` provides minimal, zero-cost Win32 bindings. |
| GUI Frameworks for Tray / Settings UI | Pure Win32 API (`Shell_NotifyIcon`) for daemon tray; native lightweight shell | GUI toolkits (e.g. Tauri, Slint, C# WPF/WinUI) bloat baseline RAM and binary footprint, violating lightweight daemon requirements. |
| Internal Z-Order Caching | Stateless Real-time Querying via `GetWindow` / `EnumWindows` | In-memory Z-order caching inevitably falls out of sync with external mouse clicks, OS events, or third-party focus shifts. |
| Skipping Unresponsive Windows | Explicit Timeout Handling & UX Honesty | Silently skipping "Not Responding" windows violates predictable switcher navigation; switcher handles unresponsive targets with non-blocking timeouts. |
| Script-based Runtime (AutoHotkey / Python / C#) | Native Rust Executable | Scripted/managed runtimes suffer from GC micro-stutters, synchronous API cascading hangs, and hook dropout under load. |

## Commercial Redactions

- Marketing strategy, target monetization models, and private cost breakdowns from the internal WinTick PRD remain excluded from the public corpus.
