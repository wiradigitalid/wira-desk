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

## Technical how — testable consequences per FR

### FR-1 — Same-Application Identity Cycling
- Pressing `Win + \`` with three Chrome windows and two Word windows open cycles focus only through the three Chrome windows if Chrome is active.
- Window cycling operates dynamically in real time without caching Z-order state between keystrokes.

### FR-4 — UX Honesty for Unresponsive Windows
- An application window marked as "Not Responding" by Windows OS receives focus when reached in the cycling sequence.
- Cycling past the unresponsive window on the next shortcut press completes without delay or hang.

### FR-5 — Minimized and Ghost Window Exclusion
- A minimized same-application window remains minimized in the taskbar and is not restored during cycling.
- System tray background helper windows and tooltips are ignored by the enumeration filter.

### FR-6 — Exact Shortcut Matching
- Shortcut recognition evaluates exact modifier state masks (Ctrl, Alt, Shift, Win).
- Extra modifier combinations are passed transparently to downstream window hooks.

### FR-2 — Physical Monitor and Virtual Desktop Boundary Locking
- Windows of the same application residing on secondary monitors are excluded from the active cycling list.
- Windows residing on other Windows Virtual Desktops are excluded from the active cycling list.

### FR-3 — VM and Remote Desktop Shortcut Passthrough
- Detection recognizes standard VM and RDP process names (`mstsc.exe`, `vmconnect.exe`, `MobaXterm`, `VMwareUnityWindow`).
- Passthrough list is configurable via `config.toml`.

### FR-8 — Elevated Execution for UIPI Focus Control
- Daemon executable embeds a `requireAdministrator` execution level manifest.
- Focus transitions into high-integrity processes succeed without error dialogs or silent focus loss.

### FR-14 — DPI-Aware Window Snapping Shortcuts
- Half-screen snap calculates bounds from `GetDpiForMonitor` and monitor work area (excluding taskbars).
- Maximize shortcut restores or maximizes window state cleanly.
- The shipped default shortcuts are the `Ctrl + Alt` family, and `Win + Ctrl + Left/Right` can no longer be configured for any action because Windows uses it to switch virtual desktops. Realizes DEC-008.

### FR-15 — Overlapping Stack Layout for Compact Monitors
- Windows are positioned at Left, Center, and Right horizontal offsets.
- Window geometry calculations adjust proportionally according to monitor DPI.

### FR-22 — Top and Bottom Half Snapping
- The top and bottom halves together cover the working area exactly, with no overlapping row and no uncovered row between them.
- An odd number of pixels in height is divided the same way every time, so repeating the shortcut never shifts the window by a pixel.
- Snapping is confined to the monitor hosting the active window; no other monitor is touched.

### FR-7 — Configurable Cycling Shortcuts and Fallback
- Configuration parses standard key names and modifier flags from TOML format.
- Daemon reloads configuration upon receiving the `WM_APP_RELOAD_CONFIG` message from the settings process.

### FR-9 — Pure Win32 Tray-Resident Daemon
- Binary links against pure Win32 C-FFI (`windows-sys`).
- No heavy UI runtimes (Electron, .NET, COM GUI frameworks) are loaded into the daemon process.

### FR-10 — Tray Icon Auto-Recovery on Explorer Restart
- Daemon registers `RegisterWindowMessageW("TaskbarCreated")`.
- Icon state is re-added via `Shell_NotifyIconW(NIM_ADD)` upon receiving the broadcast.

### FR-11 — Three-Tier Error Handling Protocol
- Non-fatal operational errors produce zero intrusive modal popups.
- Hook heartbeat monitor triggers Tier 3 visual indicators upon hook dropout.

### FR-12 — Diagnostic Log Inspection from Tray Menu
- Menu action opens `%APPDATA%\WiraDesk\logs\` in Windows Explorer.
- Diagnostic logging writes structured operational events without sensitive keystroke data.

### FR-13 — Elevated Logon Auto-Start Scheduled Task
- Scheduled task action targets the absolute path of `wiradesk.exe` with an empty working directory to mitigate DLL hijacking.
- Task configuration specifies `/RL HIGHEST` for the active `%USERNAME%`.

### FR-16 — Structured Tray Context Menu
- Menu items correctly reflect current state (e.g. checkmark on Auto-Start when enabled).
- Selecting Exit terminates the background daemon cleanly.

### FR-17 — Interactive First-Run Tutorial Simulation
- First-run flag persists in `config.toml` after tutorial completion or skip.
- Tutorial demonstrates the spatial preservation concept clearly.

### FR-18 — Physical Shortcut Capturing Listening Mode
- Listening mode processes raw virtual key codes and modifiers.
- Invalid or reserved system combinations (e.g., `Ctrl + Alt + Del`) are flagged with validation warnings.

### FR-19 — Adaptive System Light and Dark Theming
- UI listens for `WM_SETTINGCHANGE` / theme registry updates.
- High-contrast mode styling is respected when enabled.

### FR-20 — Full Keyboard Navigation Accessibility
- Tab order follows intuitive visual flow across all controls.
- Focus indicators remain clearly visible on active interactive elements.

### FR-21 — Screen Reader Accessibility via UI Automation
- Controls implement UI Automation provider interfaces.
- Toggles communicate checked/unchecked state transitions immediately to accessibility listeners.

### FR-23 — Move the Active Window to the Next Monitor
- Monitors are visited in one fixed order that wraps from the last back to the first, so repeating the shortcut returns the window to where it started.
- With one monitor attached, the shortcut does nothing: no movement, no message, and no error.
- The window remains on the virtual desktop it was on; moving it never changes which desktop shows it.
- The destination placement is derived from proportion, never from copying the window's pixel width and height.
- No other window on either monitor is moved or resized.
