# Product Brief: Wira Desk

## Why

Wira Desk is a lightweight, background Windows productivity utility that brings intuitive same-application window cycling (macOS-style `Win + backtick`) and driverless, zero-overhead mouse desktop navigation to Windows 11 and 10. Windows natively lacks any mechanism to cycle strictly among windows belonging to the current foreground process, forcing users into multi-app `Alt + Tab` switchers or taskbar hunting that breaks focus and disrupts spatial workflows across multi-monitor setups. Concurrently, standard multi-button productivity mice offer physical auxiliary controls (thumb buttons and tilt wheels) that default to basic browser navigation; unlocking them for desktop multitasking traditionally requires heavy, multi-process vendor companion suites that consume gigabytes of disk space, hundreds of megabytes of RAM, and run persistent background telemetry agents.

Built in Rust with pure Win32 bindings (`windows-sys`), Wira Desk runs as an elevated tray daemon with an uncompromising resource budget of under 5 MB of idle private-bytes RAM and near-zero CPU usage. It performs instantaneous, overlay-free window cycling while preserving monitor boundaries and spatial context, and maps standard mouse inputs directly to virtual desktop switching and desktop navigation without third-party vendor bloat. Companion settings and onboarding operations are isolated into a separate process, ensuring the core input interception loop remains lightweight, rock-solid, and responsive.

Wira Desk establishes itself as the essential, invisible window and desktop navigation companion for power users on Windows—delivering effortless same-application switching and mouse-driven multitasking while honoring the speed, stability, and resource constraints demanded by modern high-performance desktop environments.

## The Problem

macOS users migrating to Windows, along with long-time Windows power users, developers, and designers, routinely manage multiple windows within the same application—such as multiple browser profiles, code editor workspaces, or terminal sessions. On Windows, switching between these windows requires `Alt + Tab` or `Win + Tab`, which intermingles every running application across the desktop and triggers distracting visual overlays. Furthermore, users of multi-button productivity mice lack a lightweight way to repurpose physical thumb buttons and tilt wheels for desktop multitasking.

This creates significant workflow friction:
- **Spatial Disorientation:** Standard switching tools pull focus across different monitors or trigger full-desktop switcher overlays, breaking mental context and spatial memory.
- **Loss of Muscle Memory:** Users accustomed to macOS same-app switching face continuous cognitive load when navigating multiple instances of the same tool.
- **Peripheral Companion Bloat:** Standard productivity mice lack on-board macro memory (EEPROM), forcing users to install vendor companion suites (frequently exceeding 500 MB to 1.5 GB on disk, spawning 3–5 background updater/agent/telemetry processes, and consuming 150 MB to 500+ MB idle RAM) merely to map thumb buttons or tilt wheels to virtual desktop navigation.
- **Abandoned Ecosystem:** Prior alternatives like NeoSmart EasySwitch have been unmaintained since 2022, leaving compatibility and stability gaps on modern Windows builds.

## The Solution

Wira Desk installs low-level global Win32 hooks (`WH_KEYBOARD_LL` and `WH_MOUSE_LL`) to capture primary shortcuts (`Win + backtick`) and auxiliary mouse inputs (thumb buttons and horizontal tilt wheel), executing focus shifts and virtual desktop transitions directly without visual chrome.

Key capabilities include:
- **Same-App Window Cycling:** Identifies the active process from the foreground window and cycles through its top-level windows on the same monitor and virtual desktop.
- **Driverless Mouse Desktop Navigation:** Intercepts standard USB HID thumb buttons (Button 4/5) and tilt wheels (Horizontal Wheel) to navigate virtual desktops, trigger Task View, or cycle windows with zero vendor software running in the background.
- **DPI-Aware Window Snapping:** Provides optional keyboard shortcuts (`Ctrl + Alt + Left/Right`, `Ctrl + Alt + Enter`) to snap and manage windows using native monitor pixel geometry.
- **Process Isolation:** Runs the input hook and window-cycling engine in a dedicated daemon while hosting onboarding, configuration, and diagnostics in a companion settings process.
- **Tray Status and Error Visibility:** Provides lightweight tray feedback with a three-tier error protocol (silent logging, tray indicator warning, and actionable dialog for unrecoverable hook dropouts).

## What Makes This Different

- **Pure Focus Switching Without Visual Noise:** Unlike `Alt + Tab` replacements or overlay switchers, Wira Desk displays no graphical interface during cycling. Focus shifts instantly beneath the user's fingers.
- **Zero-Overhead Driverless Peripheral Navigation:** Intercepts standard USB HID mouse messages directly via the OS message stream, eliminating the need for vendor background agents, web-runtime services, or telemetry daemons.
- **Spatial Preservation:** Window navigation stays bound to the current monitor by default, avoiding unexpected focus jumps across multi-display workspaces.
- **Extreme Resource Discipline:** Written in Rust using raw `windows-sys` C-FFI rather than heavy COM runtimes or managed frameworks (such as Electron, .NET, or Python), keeping memory overhead under 5 MB (DEC-027).
- **UX Honesty for Unresponsive Windows:** Rather than silently skipping hung windows and masking application state, Wira Desk surfaces "Not Responding" windows directly to ensure predictable OS behavior.
- **Full UIPI Compatibility:** Runs with required administrator privileges to seamlessly cycle into elevated windows (e.g., administrator terminals, Task Manager) without OS permission blocks.

## Who This Serves

| Role | Need | Tier |
|---|---|---|
| Power User / Desktop Multitasker | Instant same-application window cycling and driverless mouse desktop navigation with zero overlay latency | **primary** |
| Developer / Designer | Reliable multi-window management across varied DPI multi-monitor setups with optional window snapping | secondary |
| Productivity Mouse User | Native desktop and virtual desktop navigation via physical mouse buttons and tilt wheels without vendor suite bloat | secondary |
| System Administrator | Seamless focus transitions into elevated tools and utilities without UIPI restrictions | secondary |
| Enterprise IT Administrator | Zero-telemetry, offline-first utility that adheres to strict workstation resource budgets (<2 MB static RAM) | secondary |

Shared goal across all roles: Window focus transitions and desktop navigation feel immediate, predictable, and invisible until a diagnostic intervention is explicitly requested.

## Goals

Goals — see goals.yaml → goals:

## Success Criteria

Core daemon idle memory footprint remains strictly under 5 MB of private bytes on Windows 10 and 11 across continuous 7-day workstation sessions. Private bytes, not working set: `DEC-027` fixed the metric after `DEF-11` read 29.7 MB working set against 4.0 MB private on the same process, and working set counts shareable pages the daemon does not own.

## Scope

### Scope In

- Global low-level keyboard hook intercepting primary (`Win + backtick`) and fallback (`Alt + backtick`) shortcuts.
- Global low-level mouse hook intercepting standard auxiliary inputs (Thumb Buttons 1 & 2 via `WM_XBUTTONDOWN`, Tilt Wheel via `WM_MOUSEHWHEEL`).
- Dynamic same-process window discovery and immediate focus activation without visual switcher chrome.
- Pre-configured mouse mapping presets for Windows virtual desktop navigation (`Ctrl + Win + Left/Right`), Task View (`Win + Tab`), and Wira Desk window actions.
- Software-level debounce filtering for tilt-wheel burst signals to prevent multiple transition triggers.
- Optional DPI-aware window snapping shortcuts (`Ctrl + Alt + Arrow/Enter`) and overlapping stack layouts.
- System tray lifecycle management, auto-start task integration, and local TOML configuration parsing.
- Isolated companion settings and onboarding interface with baseline accessibility compliance.
- Three-tier error and diagnostic handling for hook lifecycle recovery.

### Scope Out

- Graphical window previews, thumbnail matrices, or Alt+Tab-style visual switchers.
- Cloud synchronization, background telemetry, or remote telemetry collection.
- Proprietary vendor driver emulation or Bluetooth low-energy protocol reverse-engineering.
- Cross-device network clipboard or file transfer protocols (e.g., Logitech Flow equivalents).
- Custom gaming macro scripting or arbitrary multi-key macro sequence recording for mice.
- Per-virtual-desktop isolated snap configurations.
- Direct distribution pipelines within public continuous integration (local verified builds only).
- Automatic tiling window management (e.g. Komorebi/i3).

## Constraints

- **Windows 10 / 11 Desktop Only:** Exclusively targets modern Win32 desktop environments; no cross-platform Linux or macOS runtime support.
- **Elevated Execution Required:** Must run with elevated administrator privileges to interact with elevated target windows across User Interface Privilege Isolation (UIPI) boundaries.
- **Low-Level Input Hooks (`WH_KEYBOARD_LL` and `WH_MOUSE_LL`):** Core input interception relies on low-level OS hooks, requiring responsive message processing to prevent OS hook removal (`LowLevelHooksTimeout`).
- **Standard USB HID Only:** Mouse input capture relies exclusively on standard Win32 mouse messaging (`WM_XBUTTONDOWN`, `WM_MOUSEHWHEEL`); proprietary vendor-specific hardware protocols are not implemented.
- **No Background Telemetry:** All configuration, logging, and error tracing must remain entirely local to the host machine.
