# Product Brief Addendum: Technical Constraints & Architecture Notes

## Technical Constraints

- **Language & Runtime:** Implemented strictly in Rust targeting the MSVC toolchain (`x86_64-pc-windows-msvc`). Retains Rust standard library (`std`) for safe cross-thread primitives while forbidding heavy runtime frameworks, runtimes with garbage collection (C#/.NET, Python), and GUI web views (Electron/Tauri) within the core daemon.
- **Win32 API Bindings:** Uses `windows-sys` for direct, lightweight C-FFI bindings to Win32 APIs instead of full COM wrapper crates (e.g., `windows`), preventing binary bloat and unnecessary COM runtime overhead.
- **Elevation & UIPI:** Manifest requires `requireAdministrator` execution level. This is mandatory to bypass User Interface Privilege Isolation (UIPI) restrictions when interacting with or passing focus to elevated processes (e.g., Administrator PowerShell, Task Manager, elevated IDEs).
- **Input Interception Mechanism:** Global low-level keyboard hook (`WH_KEYBOARD_LL`) deployed via a dedicated hook thread with high responsiveness guarantees to prevent Windows OS timeout hook removal (`LowLevelHooksTimeout`).
- **Configuration Format:** Local TOML configuration loaded from the application directory without external registry reliance or cloud dependencies.

## Sizing & Performance Targets

- **Memory (RAM):**
  - Core Daemon: Strict production budget of `< 2 MB` static working set (aspirational `< 1 MB`, absolute ceiling `< 10 MB`).
  - Settings Process: Independent memory space loaded strictly on demand, exiting completely when closed.
- **Binary Footprint:** Core daemon executable size targeted at `< 500 KB` (estimated 250 KB – 400 KB via release profile optimizations: LTO, `opt-level = "z"`, stripped symbols).
- **CPU Utilization:** Near `0%` during idle; sub-millisecond dispatch on keypress.
- **Focus Transition Latency:** Instantaneous (target `< 10 ms` processing time from hook event to `SetForegroundWindow`).

## Rejected Alternatives

| Alternative | Rationale for Rejection |
|---|---|
| **`RegisterHotKey` API Only** | Operates on a first-come-first-served basis. If another application or OS shortcut claims `Win + backtick` or `Alt + backtick` first, registration fails silently. `WH_KEYBOARD_LL` guarantees deterministic interception and shortcut prioritization. |
| **Visual Switcher Overlay / HUD** | Introducing thumbnail matrices, HUD banners, or Alt+Tab-style window cards introduces rendering latency, GPU compositing overhead, and visual noise that defeats the goal of immediate, muscle-memory-driven switching. |
| **`#![no_std]` Rust Architecture** | While saving 50–100 KB in binary size, dropping `std` removes native safe thread primitives and channels, forcing complex raw C-FFI synchronization. `std` + `windows-sys` satisfies the binary and RAM budget safely. |
| **Heavier Windows Crates / COM Abstractions** | Full `windows` crate features pull large metadata and COM runtime dependencies that bloat binary size and memory footprint. Raw `windows-sys` provides minimal C-FFI interfaces. |
| **Managed Frameworks / Scripting (AutoHotkey, C#, Python)** | Garbage collection pauses introduce micro-stutters during rapid cycling, and synchronous Win32 calls risk cascading hangs when querying unresponsive windows. |
| **Silent Skipping of Hung / Unresponsive Windows** | Masking unresponsive application states violates UX honesty. Wira Desk activates the window as-is, allowing the OS "Not Responding" state and recovery UI to surface predictably. |
