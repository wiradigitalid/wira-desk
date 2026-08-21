---
type: prd
title: Wira Desk
initiative: wira-desk
status: reviewed
created: 2026-07-06
updated: 2026-08-21
provenance: >-
  Harvested from _bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md
  via wdi-product intent update (brownfield).
---

# PRD: Wira Desk

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-07-06 | Initial PRD baseline created under WinTick working title | Initial product conception and requirements definition | v1.0.0 |
| 2026-07-10 | Finalized core architecture constraints, snapping scope (P2), and UX honesty protocol | Elicitation review rounds (A1–A6) and performance budget locking | v1.0.0 |
| 2026-08-21 | Rebranded to Wira Desk and structured into WDI Method corpus format with explicit proof-of-done criteria | Migration from BMAD planning output to WDI repository standard | v1.0.0 |

## 0. Document Purpose

This Product Requirements Document (PRD) defines the user promises, functional capabilities, and cross-cutting quality constraints for Wira Desk. It serves product management, architects, developers, and quality assurance engineers as the canonical specification of what the product delivers to end users. Terminology in this document is anchored in the Product Glossary; functional requirements are nested under user-facing capabilities and linked to business goals; and all implementation assumptions are recorded and indexed for verification.

## 1. Vision

Wira Desk brings the seamless same-application window cycling experience of macOS (`Cmd + \``) to the Windows desktop ecosystem (`Win + \``). Windows natively lacks any mechanism to cycle strictly among windows belonging to the current foreground application, forcing users into noisy `Alt + Tab` switchers or taskbar hunting that breaks focus, disrupts spatial memory, and creates severe friction across multi-monitor workstations.

Operating as an ultra-lightweight, invisible background tray utility written in Rust, Wira Desk delivers instant, overlay-free window cycling while strictly respecting physical monitor and virtual desktop boundaries. It runs elevated to guarantee seamless operation across administrator and standard windows (UIPI bypass), pairs with an optional DPI-aware keyboard snapping engine, and maintains a strict memory budget under 2 MB idle RAM without telemetry or cloud dependencies.

## 2. Target User

### 2.1 Jobs To Be Done

- **Functional:** When working with multiple windows of the same application (e.g. 5 browser windows, 3 code editor workspaces, or multiple document drafts), I want to cycle directly between them with a single muscle-memory keystroke without seeing windows from other applications.
- **Contextual / Spatial:** When using multi-monitor or virtual desktop configurations, I want my focus switches to remain isolated to the screen I am actively looking at, so my peripheral workspace layout is never disrupted.
- **Emotional:** I want window switching to feel completely immediate, frictionless, and invisible—free from sluggish graphical overlays, taskbar searching, or operating system permission blocks when switching to elevated administrative tools.
- **Reliability:** When an application window freezes or hangs, I want the system to handle the situation transparently without freezing the switcher utility or losing global shortcut responsiveness.

### 2.2 Non-Users (v1)

- Users seeking a full visual task switcher replacement (e.g., Mac Mission Control or Windows Task View with rich thumbnails).
- Users demanding automated, complex tiling window manager layouts (e.g., i3, bspwm, or Komorebi style auto-tiling).
- Users requiring cloud synchronization of settings across multiple machines.

### 2.3 Key User Journeys

#### UJ-1. Rian cycles document windows on a multi-monitor workstation without spatial disruption
- **Persona + context:** Rian, a full-stack software engineer working with three Word specifications open on his left monitor and two Word drafts on his right monitor.
- **Entry state:** Authenticated on Windows desktop; Wira Desk daemon is running silently in the system tray; keyboard focus is active on a Word document on the right monitor.
- **Path:** Rian presses the global cycling shortcut (`Win + \``). The daemon captures the keypress via its low-level keyboard hook, identifies Word as the active process, filters only top-level visible Word windows located on the right physical display, and shifts focus directly to the next Word window in Z-order.
- **Climax:** Window focus transfers instantaneously to the adjacent Word draft on the right display. The left monitor remains entirely untouched, avoiding visual whiplash.
- **Resolution:** Rian continues editing without losing his mental map of where documents reside.
- **Edge case:** If only one window of the active application exists on the current monitor, pressing the shortcut produces no focus jump or screen flicker.

#### UJ-2. Maya encounters an unresponsive application window during cycling
- **Persona + context:** Maya, a UI/UX designer with eight Google Chrome windows open across research topics, where one tab process has frozen into a "Not Responding" state.
- **Entry state:** Active on a responsive Chrome window; the frozen Chrome window is next in the Z-order sequence on the active monitor.
- **Path:** Maya presses `Win + \``. The daemon's worker thread enumerates same-application windows using non-blocking kernel APIs and brings the frozen Chrome window to the foreground.
- **Climax:** The unresponsive window surfaces immediately, showing the native Windows "(Not Responding)" title state without freezing or hanging Wira Desk.
- **Resolution:** Maya instantly realizes the window is hung, presses `Win + \`` once more, and the cycle immediately advances to the next healthy Chrome window without delay.
- **Edge case:** If the frozen window is terminated via Task Manager while cycling, the next shortcut press seamlessly shifts focus to the subsequent valid window in Z-order without errors.

#### UJ-3. Budi switches between standard and elevated command terminals
- **Persona + context:** Budi, a systems administrator troubleshooting server scripts with a standard Command Prompt and an elevated Administrator Command Prompt open side-by-side.
- **Entry state:** Active focus is currently on the standard user-level Command Prompt.
- **Path:** Budi presses `Win + \``. The elevated Wira Desk daemon captures the shortcut and executes a focus transition targeting the Administrator Command Prompt.
- **Climax:** Focus transfers directly to the elevated console without being blocked or ignored by Windows User Interface Privilege Isolation (UIPI).
- **Resolution:** Budi executes administrative commands and switches back and forth between terminal windows effortlessly using the same shortcut.
- **Edge case:** If Wira Desk were launched without elevation, Windows UIPI would reject focus changes to the admin terminal; Wira Desk's elevated architecture guarantees uniform behavior.

## 3. Glossary

- **Ghost window** — A hidden, utility, or surrogate window (such as `WS_EX_TOOLWINDOW` or system shell wrappers) that is excluded from the cycling loop.
- **Hook thread** — A dedicated Windows thread running at `THREAD_PRIORITY_TIME_CRITICAL` executing the `WH_KEYBOARD_LL` low-level keyboard hook; must complete callbacks within 10 ms.
- **LowLevelHooksTimeout** — The Windows OS threshold (~300 ms) after which an unresponsive low-level keyboard hook is silently unhooked by the operating system.
- **Product Component** — A user-named capability boundary in the architecture (`window-management`, `settings`).
- **Ring buffer** — A lock-free, fixed-size 16-slot circular queue of `u8` command bytes bridging the hook thread to the worker thread with zero heap allocation.
- **Spatial preservation** — The core UX guarantee that window cycling and snapping remain strictly confined to the physical monitor and virtual desktop hosting the active window.
- **UIPI** — User Interface Privilege Isolation, the Windows security subsystem preventing lower-integrity processes from controlling or sending messages to higher-integrity windows.
- **UX honesty** — The design principle requiring unresponsive ("Not Responding") windows to be focused rather than hidden, providing transparent feedback to the user.
- **Worker thread** — The background daemon thread executing window enumeration, spatial filtering, and focus manipulation off the critical input path.
- **Z-order** — The front-to-back stacking order of overlapping windows maintained dynamically by the Windows Desktop Window Manager.

## 4. Features

### 4.1 Same-Application Window Cycling

**Capability:** CAP-1 — serves BG-1.

**Description:** Provides instantaneous, overlay-free cycling among visible windows belonging exclusively to the active application process. Triggered via a global keyboard shortcut, the cycling engine identifies the executable identity of the active foreground window and shifts focus to the next window in Z-order. Realizes UJ-1, UJ-2, and UJ-3.

**Functional Requirements:**

#### FR-1: Same-Application Identity Cycling
The system can cycle focus strictly among windows sharing the same executable identity (process path and name) as the current foreground window upon detecting the cycling shortcut. Realizes UJ-1.

**Proof of done:** Pressing the cycling shortcut while an application with multiple windows is focused advances focus sequentially only among windows of that exact application, never switching to unrelated applications.

**Consequences (testable):**
- Pressing `Win + \`` with three Chrome windows and two Word windows open cycles focus only through the three Chrome windows if Chrome is active.
- Window cycling operates dynamically in real time without caching Z-order state between keystrokes.

#### FR-4: UX Honesty for Unresponsive Windows
The system can bring "Not Responding" application windows to the foreground during a cycle rather than silently skipping them. Realizes UJ-2.

**Proof of done:** Pressing the cycling shortcut when an unresponsive same-app window is next in Z-order brings that hung window directly to the front so the user sees its frozen state.

**Consequences (testable):**
- An application window marked as "Not Responding" by Windows OS receives focus when reached in the cycling sequence.
- Cycling past the unresponsive window on the next shortcut press completes without delay or hang.

#### FR-5: Minimized and Ghost Window Exclusion
The system can filter out minimized windows, hidden system windows, tool windows (`WS_EX_TOOLWINDOW`), and ghost overlays during window enumeration. Realizes UJ-1.

**Proof of done:** Minimized windows and background utility windows are omitted from the cycling sequence, allowing focus to shift only among visibly rendered desktop windows.

**Consequences (testable):**
- A minimized same-application window remains minimized in the taskbar and is not restored during cycling.
- System tray background helper windows and tooltips are ignored by the enumeration filter.

#### FR-6: Exact Shortcut Matching
The system can intercept the cycling action only when the exact configured key combination is pressed, ignoring key events with extraneous modifier keys. Realizes UJ-1.

**Proof of done:** Pressing an unconfigured combination such as `Win + Shift + \`` when only `Win + \`` is registered passes the keystroke to the operating system without triggering window cycling.

**Consequences (testable):**
- Shortcut recognition evaluates exact modifier state masks (Ctrl, Alt, Shift, Win).
- Extra modifier combinations are passed transparently to downstream window hooks.

**Out of Scope:**
- Rendering graphical thumbnail previews or on-screen switcher HUD overlays during cycling.

---

### 4.2 Spatial Layout Preservation

**Capability:** CAP-7 — serves BG-1.

**Description:** Restricts window cycling boundaries strictly to the physical display monitor and active virtual desktop of the currently focused window, eliminating unexpected multi-monitor focus jumps. Realizes UJ-1.

**Functional Requirements:**

#### FR-2: Physical Monitor and Virtual Desktop Boundary Locking
The system can restrict same-application cycling to windows positioned on the same physical monitor and virtual desktop as the active foreground window. Realizes UJ-1.

**Proof of done:** Pressing the cycling shortcut repeatedly on a multi-monitor setup cycles only through the windows on the active monitor without moving focus or cursor to any other display.

**Consequences (testable):**
- Windows of the same application residing on secondary monitors are excluded from the active cycling list.
- Windows residing on other Windows Virtual Desktops are excluded from the active cycling list.

---

### 4.3 Virtual Machine and Remote Desktop Passthrough

**Capability:** CAP-8 — serves BG-1.

**Description:** Automatically bypasses shortcut interception when the foreground window is a known virtual machine console or remote desktop client, allowing guest operating systems to receive shortcuts natively.

**Functional Requirements:**

#### FR-3: VM and Remote Desktop Shortcut Passthrough
The system can detect if the active foreground window belongs to a configured virtual machine or remote desktop client and pass cycling shortcuts through to the guest OS without interception.

**Proof of done:** Pressing `Win + \`` while focused inside a Remote Desktop (`mstsc.exe`) or VMware session transmits the raw keystroke directly to the remote session without cycling host windows.

**Consequences (testable):**
- Detection recognizes standard VM and RDP process names (`mstsc.exe`, `vmconnect.exe`, `MobaXterm`, `VMwareUnityWindow`).
- Passthrough list is configurable via `config.toml`.

---

### 4.4 Elevated Window Management and UIPI Bypass

**Capability:** CAP-4 — serves BG-1.

**Description:** Executes the core daemon with Administrator privileges to navigate User Interface Privilege Isolation (UIPI) boundaries, enabling reliable focus switching across elevated and standard application windows. Realizes UJ-3.

**Functional Requirements:**

#### FR-8: Elevated Execution for UIPI Focus Control
The system can execute with elevated Administrator privileges via an embedded application manifest to shift focus seamlessly to elevated target windows. Realizes UJ-3.

**Proof of done:** Pressing the cycling shortcut successfully shifts focus into an Administrator Command Prompt or Task Manager window without operating system denial.

**Consequences (testable):**
- Daemon executable embeds a `requireAdministrator` execution level manifest.
- Focus transitions into high-integrity processes succeed without error dialogs or silent focus loss.

---

### 4.5 DPI-Aware Window Snapping and Overlapping Stack

**Capability:** CAP-2 — serves BG-3.

**Description:** Provides optional keyboard-driven window arrangement shortcuts that snap, maximize, and position the active window with per-monitor DPI awareness, including an overlapping stack layout algorithm for compact displays.

**Functional Requirements:**

#### FR-14: DPI-Aware Window Snapping Shortcuts
The system can snap and resize the active window to half-screen left/right or maximized layouts using dedicated keyboard shortcuts (`Ctrl + Win + Left/Right`, `Ctrl + Win + Enter`), scaled to the target monitor's DPI.

**Proof of done:** Pressing `Ctrl + Win + Left` resizes and aligns the active window to exactly 50% of the working area of the current monitor, taking display scaling into account.

**Consequences (testable):**
- Half-screen snap calculates bounds from `GetDpiForMonitor` and monitor work area (excluding taskbars).
- Maximize shortcut restores or maximizes window state cleanly.

#### FR-15: Overlapping Stack Layout for Compact Monitors
The system can arrange up to three same-application windows in an overlapping 50%-width stack with offset horizontal edges on small screens.

**Proof of done:** Triggering the stack layout command positions up to three windows at 50% screen width each with visible exposed borders allowing mouse selection.

**Consequences (testable):**
- Windows are positioned at Left, Center, and Right horizontal offsets.
- Window geometry calculations adjust proportionally according to monitor DPI.

---

### 4.6 Local Configuration Persistence

**Capability:** CAP-3 — serves BG-2.

**Description:** Manages user preferences, custom shortcut bindings, and passthrough lists using a local TOML configuration file stored in the user's roaming AppData directory.

**Functional Requirements:**

#### FR-7: Configurable Cycling Shortcuts and Fallback
The system can read and apply primary (`Win + \``) and fallback (`Alt + \``) shortcut configurations from `%APPDATA%\WiraDesk\config.toml`.

**Proof of done:** Modifying the shortcut key in `config.toml` updates the active cycling shortcut immediately after configuration reload.

**Consequences (testable):**
- Configuration parses standard key names and modifier flags from TOML format.
- Daemon reloads configuration upon receiving the `WM_APP_RELOAD_CONFIG` message from the settings process.

---

### 4.7 Native System Tray Lifecycle and Error Protocol

**Capability:** CAP-9 & CAP-6 — serves BG-2.

**Description:** Runs as an invisible, native Win32 system tray daemon with automatic recovery following Explorer shell restarts and a structured three-tier error handling protocol.

**Functional Requirements:**

#### FR-9: Pure Win32 Tray-Resident Daemon
The system can maintain its background execution lifecycle exclusively through native Win32 APIs and an `ITaskbarList` tray icon without linking external third-party GUI frameworks.

**Proof of done:** The background daemon runs with an active system tray icon while consuming under 2 MB of static RAM.

**Consequences (testable):**
- Binary links against pure Win32 C-FFI (`windows-sys`).
- No heavy UI runtimes (Electron, .NET, COM GUI frameworks) are loaded into the daemon process.

#### FR-10: Tray Icon Auto-Recovery on Explorer Restart
The system can intercept the `TaskbarCreated` window message and recreate the system tray notification icon whenever `explorer.exe` restarts.

**Proof of done:** Terminating and restarting `explorer.exe` via Task Manager automatically restores the Wira Desk tray icon without restarting the daemon.

**Consequences (testable):**
- Daemon registers `RegisterWindowMessageW("TaskbarCreated")`.
- Icon state is re-added via `Shell_NotifyIconW(NIM_ADD)` upon receiving the broadcast.

#### FR-11: Three-Tier Error Handling Protocol
The system can execute a structured error protocol: Tier 1 (fatal startup displays ≤1 message box and exits), Tier 2 (runtime warning logs silently and adds red dot to tray icon), and Tier 3 (runtime hook death renders red cross on tray icon and delivers exactly one notification toast).

**Proof of done:** If the keyboard hook is terminated by the OS, the tray icon displays a red cross indicator and delivers a single desktop toast alert.

**Consequences (testable):**
- Non-fatal operational errors produce zero intrusive modal popups.
- Hook heartbeat monitor triggers Tier 3 visual indicators upon hook dropout.

---

### 4.8 Diagnostic Log Access

**Capability:** CAP-11 — serves BG-2.

**Description:** Provides quick access to local silent runtime diagnostic logs directly from the system tray context menu.

**Functional Requirements:**

#### FR-12: Diagnostic Log Inspection from Tray Menu
The system can open the local silent log file location in File Explorer when the user selects "View Logs" from the tray context menu.

**Proof of done:** Clicking "View Logs" in the tray menu opens the directory containing Wira Desk diagnostic logs.

**Consequences (testable):**
- Menu action opens `%APPDATA%\WiraDesk\logs\` in Windows Explorer.
- Diagnostic logging writes structured operational events without sensitive keystroke data.

---

### 4.9 Silent Auto-Start at Logon

**Capability:** CAP-10 — serves BG-2.

**Description:** Configures seamless, elevated background auto-start upon user logon via the Windows Task Scheduler, avoiding repeated UAC elevation prompts.

**Functional Requirements:**

#### FR-13: Elevated Logon Auto-Start Scheduled Task
The system can create or delete a Windows scheduled task (`WiraDeskAutoStart`) configured to launch the daemon with highest privileges upon user logon (`ONLOGON`).

**Proof of done:** Toggling the Auto-Start option in the tray menu creates a scheduled task that launches Wira Desk silently upon next reboot without prompting for UAC credentials.

**Consequences (testable):**
- Scheduled task action targets the absolute path of `wiradesk.exe` with an empty working directory to mitigate DLL hijacking.
- Task configuration specifies `/RL HIGHEST` for the active `%USERNAME%`.

---

### 4.10 Onboarding, Settings UI, and Accessibility

**Capability:** CAP-5 — serves BG-2.

**Description:** Delivers an on-demand, standalone settings and interactive onboarding application (`wiradesk-settings.exe`) with native theme adaptation and full keyboard/screen-reader accessibility.

**Functional Requirements:**

#### FR-16: Structured Tray Context Menu
The system can present a right-click tray context menu structured in exact order: Settings..., View Logs, Auto-Start (toggle), [separator], Check for Updates..., About, [separator], Exit.

**Proof of done:** Right-clicking the tray icon displays the context menu matching the exact specified ordering and separator placement.

**Consequences (testable):**
- Menu items correctly reflect current state (e.g. checkmark on Auto-Start when enabled).
- Selecting Exit terminates the background daemon cleanly.

#### FR-17: Interactive First-Run Tutorial Simulation
The system can launch an interactive onboarding simulation upon initial installation with dummy windows to practice the cycling shortcut, including an accessible "Skip Tutorial" option.

**Proof of done:** Launching the application for the first time opens an onboarding window where users can practice cycling through mock windows or click "Skip Tutorial".

**Consequences (testable):**
- First-run flag persists in `config.toml` after tutorial completion or skip.
- Tutorial demonstrates the spatial preservation concept clearly.

#### FR-18: Physical Shortcut Capturing Listening Mode
The system can capture physical keyboard combinations in real time within the settings shortcut input fields instead of accepting standard typed text strings.

**Proof of done:** Clicking into the shortcut input box and pressing `Alt + \`` captures the key combination directly as the new shortcut binding.

**Consequences (testable):**
- Listening mode processes raw virtual key codes and modifiers.
- Invalid or reserved system combinations (e.g., `Ctrl + Alt + Del`) are flagged with validation warnings.

#### FR-19: Adaptive System Light and Dark Theming
The system can detect Windows system theme changes and adapt the settings UI colors automatically between light and dark modes.

**Proof of done:** Toggling the Windows display theme between Light and Dark immediately updates the background and text palette of the settings window.

**Consequences (testable):**
- UI listens for `WM_SETTINGCHANGE` / theme registry updates.
- High-contrast mode styling is respected when enabled.

#### FR-20: Full Keyboard Navigation Accessibility
The system can support complete keyboard navigation across all interactive settings dialogs and controls via logical Tab order and shortcut keys.

**Proof of done:** A user can configure all settings, test shortcuts, and exit the settings UI using only the keyboard without mouse interaction.

**Consequences (testable):**
- Tab order follows intuitive visual flow across all controls.
- Focus indicators remain clearly visible on active interactive elements.

#### FR-21: Screen Reader Accessibility via UI Automation
The system can expose state, roles, names, and shortcut values across all settings controls to assistive screen readers via Windows UI Automation.

**Proof of done:** Windows Narrator accurately reads aloud the state of the Auto-Start toggle and the captured shortcut keys in the settings window.

**Consequences (testable):**
- Controls implement UI Automation provider interfaces.
- Toggles communicate checked/unchecked state transitions immediately to accessibility listeners.

---

## 5. Cross-Cutting NFRs

| ID | Requirement | Target | Enforced by |
|---|---|---|---|
| **NFR-1** | Daemon Static RAM Footprint | < 2 MB idle (hard ceiling < 10 MB) | `windows-sys` crate, absence of managed runtimes, and aggressive release compilation |
| **NFR-2** | Hook Callback Execution Time | < 10 ms callback duration (sub-millisecond perceived focus change) | Dedicated `TIME_CRITICAL` hook thread, asynchronous worker handoff, and avoiding COM/heavy APIs in hook |
| **NFR-3** | Hot Path Heap Allocations | Zero dynamic heap allocations in hook→worker path | 16-slot lock-free static `u8` ring buffer with Copy primitives |
| **NFR-4** | Non-Blocking Kernel Filtering | Zero synchronous inter-process calls in window enumerator | Strict `EnumWindows` policy using non-blocking kernel reads (`IsWindowVisible`, `GetWindowLong`, `SetWindowPos`) |
| **NFR-5** | Release Executable Binary Size | 250 KB – 400 KB typical (< 500 KB hard ceiling) | Cargo release profile (`opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"`) |
| **NFR-6** | Single Instance & Startup Resilience | Exactly one instance per user session; robust hook retry on logon | Named session mutex (`Local\WiraDeskSingleInstance`) and startup retry loop handling logon races |

## 6. Constraints and Guardrails

- **Delta Beyond Brief:** None beyond the Product Brief (`.what/_product-brief/brief.md`).
- **Platform Constraint:** Strictly targets 64-bit Windows 10 (1809+) and Windows 11 desktop environments; no legacy Windows 7/8 or non-Windows platforms.
- **Elevation Requirement:** Daemon requires Administrator privileges to ensure UIPI bypass across all target windows.
- **Privacy & Telemetry:** Absolute zero telemetry, remote analytics, network connections, or cloud syncing; all logs and configurations are strictly local to `%APPDATA%\WiraDesk`.
- **Architectural Separation:** Dual-binary model (`wiradesk.exe` headless tray daemon vs `wiradesk-settings.exe` on-demand UI) to guarantee UI rendering overhead never degrades input hook responsiveness.

## 7. Non-Goals (Explicit)

- **No Visual Switcher HUD:** Wira Desk will not render graphical window switcher menus, overlays, or task thumbnails. Switching is strictly invisible.
- **No Cloud Synchronization:** Configuration will not sync across accounts or remote servers.
- **No Independent Virtual Desktop Snap Profiles in v1:** Window snap geometry applies uniformly across all virtual desktops.
- **No Automatic Tiling Window Management:** Wira Desk is a lightweight cycling and snapping utility, not an automated tiling window manager (e.g. Komorebi/i3).

## 8. MVP Scope

### 8.1 In Scope
- Low-level keyboard hook capturing `Win + \`` and fallback `Alt + \``.
- Same-application process window filtering with spatial preservation (per monitor and virtual desktop).
- UIPI bypass via elevated daemon execution.
- UX honesty surfacing unresponsive ("Not Responding") windows.
- Three-tier error handling protocol with tray recovery.
- DPI-aware keyboard snapping (`Ctrl + Win + Arrows/Enter`) and overlapping stack layout.
- Separate settings binary with first-run onboarding, physical key listening, and UI Automation accessibility.
- Silent auto-start via Windows Task Scheduler.
- Local TOML configuration parsing and logging.

### 8.2 Out of Scope for MVP
- Multi-monitor window repositioning shortcuts (delegated to native Windows `Win + Shift + Arrows`).
- Per-virtual-desktop independent snap configuration layouts (deferred to future exploration).
- Visual switcher preview overlays.
- Cloud configuration sync or mobile companion apps.

## 9. Success Metrics

### Primary Metrics
- **SM-1: Focus Transfer Latency** — Perceived end-to-end focus transfer latency occurs in under 1 ms following keypress during standard desktop workloads. Validates FR-1, FR-2, FR-6, NFR-2.
- **SM-2: Hook Stability & Reliability** — Zero unhandled hook dropouts or unhook events across continuous 7-day user sessions. Validates FR-9, FR-10, FR-11, NFR-2, NFR-6.

### Secondary Metrics
- **SM-3: Resource Efficiency** — Idle static RAM consumption of the background daemon remains strictly under 2 MB. Validates FR-9, NFR-1, NFR-5.

### Counter-Metrics (Do Not Optimize)
- **SM-C1: Binary Size vs Feature Integrity** — Do not sacrifice Rust `std` thread safety or essential error handling to artificially drive binary size below 200 KB. Counterbalances NFR-5.
- **SM-C2: Focus Speed vs UX Honesty** — Do not skip unresponsive windows to make cycling appear faster; UX honesty must be preserved. Counterbalances SM-1, validates FR-4.
- **SM-C3: Hook Interception vs System Stability** — Do not hold the low-level hook callback open to perform complex window logic; all enumeration must occur on the worker thread. Counterbalances SM-1, validates NFR-2, NFR-3.

## 10. Open Questions

1. Should future versions support customizable process exclusion lists in the settings UI for games and full-screen graphical applications? (Currently handled via manual TOML editing).
2. Should independent snap layouts per virtual desktop be introduced in v2 following user feedback?

## 11. Assumptions Index

- `[ASSUMPTION: §2.1]` Users are willing to grant initial Administrator elevation to allow seamless window switching across administrative consoles and Task Manager.
- `[ASSUMPTION: §4.3]` Standard virtual machine and remote desktop clients expose predictable process and window class names (`mstsc.exe`, `vmconnect.exe`) suitable for passthrough filtering.
- `[ASSUMPTION: §4.9]` Windows Task Scheduler `ONLOGON` tasks with highest privileges provide a reliable, silent auto-start mechanism across diverse Windows 10 and 11 configurations.
