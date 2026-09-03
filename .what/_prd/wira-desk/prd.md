---
type: prd
title: Wira Desk
initiative: wira-desk
status: reviewed
created: 2026-07-06
updated: 2026-08-26
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
| 2026-08-26 | Snapping now covers the top and bottom halves of a screen, not only the left and right; moving the active window to another monitor became something Wira Desk does itself instead of leaving to Windows; and every shipped arrangement shortcut moved to the Ctrl+Alt family | The owner asked for vertical halves and a deliberate monitor move. The shortcut family moved because the previous default, `Ctrl+Win+Left/Right`, silently took over Windows' own shortcut for switching virtual desktops — a promise this product had already made not to break. Monitor movement was previously delegated to Windows' `Win+Shift+Arrow`, which discards whatever arrangement the user had just applied, so the two features never composed | v0.4.0 |
| 2026-09-03 | Added Update Checking (CAP-13, FR-24, FR-25): the product already shipped an optional, toggleable HTTPS check for a newer release, disclosed in `PRIVACY.md` but never promised here; §7's Constraints corrected to state the one exception instead of an absolute zero; FR-16's tray menu order corrected to match what ships (an "Update to \<version\>..." item only when one is available, not an always-present "Check for Updates...") | `wdi-reconcile` traced the shipped code against the corpus and found the update-check subsystem — real, deliberate, already privacy-documented — had no promise anywhere in `.what/`, and that FR-16's proof no longer matched the running menu | v0.4.0 |

## 1. Why This Initiative

<!-- New shape wants a delta against the brief's `Why`, not a restatement. No sentence below is
     word-for-word identical to brief.md's `Why`, so none was removed — deciding which paraphrases
     are copies is the owner's call, not this migration's. -->

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

#### UJ-4. Sari lays out a review session across a laptop screen and an external display
- **Persona + context:** Sari, a technical writer reviewing a specification against its implementation on a 14-inch laptop with a larger external monitor to its right. The two displays run at different scaling factors, which is the normal state of her desk rather than an unusual one.
- **Entry state:** Wira Desk is running in the tray. The specification document and a terminal are both on the laptop screen; the external monitor holds a browser.
- **Path:** Sari snaps the specification to the top half of the laptop screen and the terminal to the bottom half, so she can read and run at the same time on a screen too short to make left and right halves useful. She then decides the specification belongs on the larger display, and moves it there with a single shortcut.
- **Climax:** The specification arrives on the external monitor still occupying the top half — the same share of the working area it held on the laptop, not the same number of pixels — so the arrangement she built survives the move. Her browser and terminal stay exactly where they were.
- **Resolution:** Sari works across both screens with a layout she assembled from the keyboard in a few seconds, and rebuilds it the same way whenever she docks.
- **Edge case:** With only the laptop screen attached, the move shortcut does nothing at all — no window jump, no message, no error. On a screen too short to divide, the snap declines rather than producing a window with no height.

## 3. Features

### 3.1 Same-Application Window Cycling

**Capability:** CAP-1 — serves BG-1.

**Description:** Provides instantaneous, overlay-free cycling among visible windows belonging exclusively to the active application process. Triggered via a global keyboard shortcut, the cycling engine identifies the executable identity of the active foreground window and shifts focus to the next window in Z-order. Realizes UJ-1, UJ-2, and UJ-3.

**Realizes:** FR-1, FR-4, FR-5, FR-6

**Out of Scope:**
- Rendering graphical thumbnail previews or on-screen switcher HUD overlays during cycling.

---

### 3.2 Spatial Layout Preservation

**Capability:** CAP-7 — serves BG-1.

**Description:** Restricts window cycling boundaries strictly to the physical display monitor and active virtual desktop of the currently focused window, eliminating unexpected multi-monitor focus jumps. Realizes UJ-1.

**Realizes:** FR-2

---

### 3.3 Virtual Machine and Remote Desktop Passthrough

**Capability:** CAP-8 — serves BG-1.

**Description:** Automatically bypasses shortcut interception when the foreground window is a known virtual machine console or remote desktop client, allowing guest operating systems to receive shortcuts natively.

**Realizes:** FR-3

---

### 3.4 Elevated Window Management and UIPI Bypass

**Capability:** CAP-4 — serves BG-1.

**Description:** Executes the core daemon with Administrator privileges to navigate User Interface Privilege Isolation (UIPI) boundaries, enabling reliable focus switching across elevated and standard application windows. Realizes UJ-3.

**Realizes:** FR-8

---

### 3.5 DPI-Aware Window Snapping and Overlapping Stack

**Capability:** CAP-2 — serves BG-3.

**Description:** Provides optional keyboard-driven window arrangement shortcuts that snap the active window to either half of the screen — left, right, top, or bottom — maximize it, and arrange several windows as an overlapping stack, all with per-monitor DPI awareness.

**Realizes:** FR-14, FR-15, FR-22

---

### 3.6 Local Configuration Persistence

**Capability:** CAP-3 — serves BG-2.

**Description:** Manages user preferences, custom shortcut bindings, and passthrough lists using a local TOML configuration file stored in the user's roaming AppData directory.

**Realizes:** FR-7

---

### 3.7 Native System Tray Lifecycle and Error Protocol

**Capability:** CAP-9 & CAP-6 — serves BG-2.

**Description:** Runs as an invisible, native Win32 system tray daemon with automatic recovery following Explorer shell restarts and a structured three-tier error handling protocol.

**Realizes:** FR-9, FR-10, FR-11

---

### 3.8 Diagnostic Log Access

**Capability:** CAP-11 — serves BG-2.

**Description:** Provides quick access to local silent runtime diagnostic logs directly from the system tray context menu.

**Realizes:** FR-12

---

### 3.9 Silent Auto-Start at Logon

**Capability:** CAP-10 — serves BG-2.

**Description:** Configures seamless, elevated background auto-start upon user logon via the Windows Task Scheduler, avoiding repeated UAC elevation prompts.

**Realizes:** FR-13

---

### 3.10 Onboarding, Settings UI, and Accessibility

**Capability:** CAP-5 — serves BG-2.

**Description:** Delivers an on-demand, standalone settings and interactive onboarding application (`wiradesk-settings.exe`) with native theme adaptation and full keyboard/screen-reader accessibility.

**Realizes:** FR-16, FR-17, FR-18, FR-19, FR-20, FR-21

---

### 3.11 Deliberate Movement Between Monitors

**Capability:** CAP-12 — serves BG-3.

**Description:** Moves the active window to another physical monitor on purpose, from the keyboard, keeping the arrangement the user has already built rather than replacing it. Realizes UJ-4.

Windows already moves windows between monitors with `Win + Shift + Arrow`, and Wira Desk deliberately left that job to Windows until now. It is taken back here because the Windows shortcut re-decides the window's state on arrival, discarding a snap the user applied a moment earlier — so the two features never combined into one layout. Realizes DEC-007.

**Realizes:** FR-23

---

### 3.12 Update Checking

**Capability:** CAP-13 — serves BG-2.

**Description:** Tells the user when a newer release exists, without touching the network for anything else. The daemon checks on its own schedule and surfaces a found update in the tray menu; the settings application lets the user check on demand and, on confirmation, fetches and verifies the installer before offering to run it. Both paths share one property: nothing about how the product is used ever leaves the machine.

**Realizes:** FR-24, FR-25

---

## 4. MVP Scope

### 4.1 In Scope
- Low-level keyboard hook capturing `Win + \`` and fallback `Alt + \``.
- Same-application process window filtering with spatial preservation (per monitor and virtual desktop).
- UIPI bypass via elevated daemon execution.
- UX honesty surfacing unresponsive ("Not Responding") windows.
- Three-tier error handling protocol with tray recovery.
- DPI-aware keyboard snapping to any half of the screen (`Ctrl + Alt + Arrows`) and maximize (`Ctrl + Alt + Enter`), plus the overlapping stack layout.
- Moving the active window to the next physical monitor from the keyboard, keeping its share of the working area.
- Separate settings binary with first-run onboarding, physical key listening, and UI Automation accessibility.
- Silent auto-start via Windows Task Scheduler.
- Local TOML configuration parsing and logging.

### 4.2 Out of Scope for MVP
- Moving a window to a *named* monitor (primary, secondary) rather than the next one — the next-and-wrap shortcut needs no stable monitor identity, and Windows does not offer one cheaply. See DEC-007.
- Per-virtual-desktop independent snap configuration layouts (deferred to future exploration).
- Visual switcher preview overlays.
- Cloud configuration sync or mobile companion apps.

## 5. Success Metrics

### Primary Metrics
- **SM-1: Focus Transfer Latency** — Perceived end-to-end focus transfer latency occurs in under 1 ms following keypress during standard desktop workloads. Validates FR-1, FR-2, FR-6, NFR-2.
- **SM-2: Hook Stability & Reliability** — Zero unhandled hook dropouts or unhook events across continuous 7-day user sessions. Validates FR-9, FR-10, FR-11, NFR-2, NFR-6.

### Secondary Metrics
- **SM-3: Resource Efficiency** — Idle static RAM consumption of the background daemon remains strictly under 2 MB. Validates FR-9, NFR-1, NFR-5.

### Counter-Metrics (Do Not Optimize)
- **SM-C1: Binary Size vs Feature Integrity** — Do not sacrifice Rust `std` thread safety or essential error handling to artificially drive binary size below 200 KB. Counterbalances NFR-5.
- **SM-C2: Focus Speed vs UX Honesty** — Do not skip unresponsive windows to make cycling appear faster; UX honesty must be preserved. Counterbalances SM-1, validates FR-4.
- **SM-C3: Hook Interception vs System Stability** — Do not hold the low-level hook callback open to perform complex window logic; all enumeration must occur on the worker thread. Counterbalances SM-1, validates NFR-2, NFR-3.

## 6. Cross-Cutting NFRs

| ID | Requirement | Target | Enforced by |
|---|---|---|---|
| **NFR-1** | Daemon Static RAM Footprint | < 2 MB idle (hard ceiling < 10 MB) | `windows-sys` crate, absence of managed runtimes, and aggressive release compilation |
| **NFR-2** | Hook Callback Execution Time | < 10 ms callback duration (sub-millisecond perceived focus change) | Dedicated `TIME_CRITICAL` hook thread, asynchronous worker handoff, and avoiding COM/heavy APIs in hook |
| **NFR-3** | Hot Path Heap Allocations | Zero dynamic heap allocations in hook→worker path | 16-slot lock-free static `u8` ring buffer with Copy primitives |
| **NFR-4** | Non-Blocking Kernel Filtering | Zero synchronous inter-process calls in window enumerator | Strict `EnumWindows` policy using non-blocking kernel reads (`IsWindowVisible`, `GetWindowLong`, `SetWindowPos`) |
| **NFR-5** | Release Executable Binary Size | 250 KB – 400 KB typical (< 500 KB hard ceiling) | Cargo release profile (`opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"`) |
| **NFR-6** | Single Instance & Startup Resilience | Exactly one instance per user session; robust hook retry on logon | Named session mutex (`Local\WiraDeskSingleInstance`) and startup retry loop handling logon races |

## 7. Constraints and Guardrails

- **Delta Beyond Brief:** None beyond the Product Brief (`.what/_product-brief/brief.md`).
- **Platform Constraint:** Strictly targets 64-bit Windows 10 (1809+) and Windows 11 desktop environments; no legacy Windows 7/8 or non-Windows platforms.
- **Elevation Requirement:** Daemon requires Administrator privileges to ensure UIPI bypass across all target windows.
- **Privacy & Telemetry:** Zero telemetry, remote analytics, or cloud syncing; all logs and configurations are strictly local to `%APPDATA%\WiraDesk`. The one exception is the update check (CAP-13): an optional, toggleable HTTPS request carrying no payload beyond the request itself and no identifying data. Nothing else in the product ever touches the network.
- **Architectural Separation:** Dual-binary model (`wiradesk.exe` headless tray daemon vs `wiradesk-settings.exe` on-demand UI) to guarantee UI rendering overhead never degrades input hook responsiveness.

