# Product Brief: Wira Desk

## Why

Wira Desk is a lightweight, background Windows productivity utility that brings the intuitive same-application window cycling behavior of macOS (`Command + ~`) to Windows 11 and 10 via a dedicated keyboard shortcut (`Win + backtick`). Windows natively lacks any mechanism to cycle strictly among windows belonging to the current foreground process, forcing users into multi-app `Alt + Tab` switchers or taskbar hunting that breaks focus and disrupts spatial workflows across multi-monitor setups. Existing third-party commercial utilities have been abandoned since 2022, leaving power users without a reliable, active solution.

Built in Rust with pure Win32 bindings (`windows-sys`), Wira Desk runs as an elevated tray daemon with an uncompromising resource budget of under 2 MB idle RAM and near-zero CPU usage. It performs instantaneous, overlay-free window cycling while preserving monitor boundaries and spatial context. Companion settings and onboarding operations are isolated into a separate process, ensuring the core input interception loop remains lightweight, rock-solid, and responsive.

Wira Desk establishes itself as the essential, invisible window navigation companion for power users on Windows—delivering the effortless elegance of macOS same-application switching while honoring the speed, stability, and resource constraints demanded by modern high-performance desktop environments.

## The Problem

macOS users migrating to Windows, along with long-time Windows power users, developers, and designers, routinely manage multiple windows within the same application—such as multiple browser profiles, code editor workspaces, or terminal sessions. On Windows, switching between these windows requires `Alt + Tab` or `Win + Tab`, which intermingles every running application across the desktop and triggers distracting visual overlays.

This creates significant workflow friction:
- **Spatial Disorientation:** Standard switching tools pull focus across different monitors or trigger full-desktop switcher overlays, breaking mental context and spatial memory.
- **Loss of Muscle Memory:** Users accustomed to macOS same-app switching face continuous cognitive load when navigating multiple instances of the same tool.
- **Abandoned Ecosystem:** Prior alternatives like NeoSmart EasySwitch have been unmaintained since 2022, leaving compatibility and stability gaps on modern Windows builds.

## The Solution

Wira Desk installs a low-level, global keyboard hook (`WH_KEYBOARD_LL`) to capture `Win + backtick` (with configurable fallback to `Alt + backtick`) and move focus directly to the next window belonging to the same process. Focus moves immediately without rendering visual switcher overlays, HUDs, or window preview thumbnails.

Key capabilities include:
- **Same-App Window Cycling:** Identifies the active process from the foreground window and cycles through its top-level windows on the same monitor and virtual desktop.
- **DPI-Aware Window Snapping:** Provides optional keyboard shortcuts (`Ctrl + Win + Left/Right`, `Ctrl + Win + Enter`) to snap and manage windows using native monitor pixel geometry.
- **Process Isolation:** Runs the input hook and window-cycling engine in a dedicated daemon while hosting onboarding, configuration, and diagnostics in a companion settings process.
- **Tray Status and Error Visibility:** Provides lightweight tray feedback with a three-tier error protocol (silent logging, tray indicator warning, and actionable dialog for unrecoverable hook dropouts).

## What Makes This Different

- **Pure Focus Switching Without Visual Noise:** Unlike `Alt + Tab` replacements or overlay switchers, Wira Desk displays no graphical interface during cycling. Focus shifts instantly beneath the user's fingers.
- **Spatial Preservation:** Window navigation stays bound to the current monitor by default, avoiding unexpected focus jumps across multi-display workspaces.
- **Extreme Resource Discipline:** Written in Rust using raw `windows-sys` C-FFI rather than heavy COM runtimes or managed frameworks (such as Electron, .NET, or Python), keeping memory overhead under 2 MB.
- **UX Honesty for Unresponsive Windows:** Rather than silently skipping hung windows and masking application state, Wira Desk surfaces "Not Responding" windows directly to ensure predictable OS behavior.
- **Full UIPI Compatibility:** Runs with required administrator privileges to seamlessly cycle into elevated windows (e.g., administrator terminals, Task Manager) without OS permission blocks.

## Who This Serves

| Role | Need | Tier |
|---|---|---|
| Power User / macOS Migrant | Instant same-application window cycling with spatial preservation and zero overlay latency | **primary** |
| Developer / Designer | Reliable multi-window management across varied DPI multi-monitor setups with optional window snapping | secondary |
| System Administrator | Seamless focus transitions into elevated tools and utilities without UIPI restrictions | secondary |
| Enterprise IT Administrator | Zero-telemetry, offline-first utility that adheres to strict workstation resource budgets | secondary |

Shared goal across all roles: Window focus transitions feel immediate, predictable, and invisible until a diagnostic intervention is explicitly requested.

## Goals

Goals — see `.control/registry/goals.yaml` → `goals:`.

## Success Criteria

- Perceived focus transition occurs immediately upon keypress during normal desktop workloads.
- Core daemon idle memory footprint remains strictly under 2 MB static RAM in production profiling.
- Global keyboard hook maintains attachment without dropouts or unhandled timeouts across continuous multi-day sessions.
- Users can inspect and diagnose operational issues directly through local tray logs and error alerts.

## Scope

### Scope In

- Global low-level keyboard hook intercepting primary (`Win + backtick`) and fallback (`Alt + backtick`) shortcuts.
- Dynamic same-process window discovery and immediate focus activation without visual switcher chrome.
- Optional DPI-aware window snapping shortcuts (`Ctrl + Win + Arrow/Enter`) and overlapping stack layouts.
- System tray lifecycle management, auto-start task integration, and local TOML configuration parsing.
- Isolated companion settings and onboarding interface with baseline accessibility compliance.
- Three-tier error and diagnostic handling for hook lifecycle recovery.

### Scope Out

- Graphical window previews, thumbnail matrices, or Alt+Tab-style visual switchers.
- Cloud synchronization, background telemetry, or remote telemetry collection.
- Per-virtual-desktop isolated snap configurations.
- Direct distribution pipelines within public continuous integration (local verified builds only).
- Automatic tiling window management (e.g. Komorebi/i3).

## Constraints

- **Windows 10 / 11 Desktop Only:** Exclusively targets modern Win32 desktop environments; no cross-platform Linux or macOS runtime support.
- **Elevated Execution Required:** Must run with elevated administrator privileges to interact with elevated target windows across User Interface Privilege Isolation (UIPI) boundaries.
- **Low-Level Keyboard Hook (`WH_KEYBOARD_LL`):** Core input interception relies on a low-level OS keyboard hook rather than `RegisterHotKey`, requiring transparent user communication regarding system hook usage.
- **No Background Telemetry:** All configuration, logging, and error tracing must remain entirely local to the host machine.
