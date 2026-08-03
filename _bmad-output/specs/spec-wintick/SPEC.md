---
id: SPEC-wintick
companions:
  - conventions.md
  - ../../planning-artifacts/ux-designs/ux-WinTick-2026-07-06/DESIGN.md
  - ../../planning-artifacts/ux-designs/ux-WinTick-2026-07-06/EXPERIENCE.md
  - ../../planning-artifacts/architecture/architecture-WinTick-2026-07-06/ARCHITECTURE-SPINE.md
  - ../../../design-system/project/readme.md
sources:
  - ../../planning-artifacts/mom-2026-07-04-discussion.md
  - ../../planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md
  - ../../planning-artifacts/prds/prd-WinTick-2026-07-06/addendum.md
  - ../../planning-artifacts/mom-2026-07-06-ux-design.md
  - ../../planning-artifacts/mom-2026-07-06-advanced-elicitation.md
  - ../../planning-artifacts/mom-2026-07-07-architecture-coaching.md
  - ../../planning-artifacts/mom-2026-07-09-elevated-administrator-access.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability only — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# WinTick Window Switcher

## Why

Windows lacks a native mechanism to switch between multiple windows of the same active application using a simple keyboard shortcut (comparable to macOS `Cmd + ~`). WinTick solves this by providing a lightweight, low-memory background utility written in Rust. It enables instant same-application window cycling and custom snapping shortcuts across all active windows—including those running with elevated privileges (Administrator rights)—while maintaining a memory footprint below 2MB.

## Capabilities

- **CAP-1**
  - **intent:** User can cycle focus exclusively between all open, non-minimized window instances of the currently active application using an exact keyboard shortcut (defaulting to `Win + Backtick`).
  - **success:** Pressing the exact shortcut instantly shifts focus to the next strictly visible window of the active application. It must actively filter out minimized windows, hidden ghost windows, and system overlays (e.g., rejecting `WS_EX_TOOLWINDOW`), without introducing visible focus latency. Extra modifiers (e.g., Shift) will not trigger the cycle.
- **CAP-2**
  - **intent:** User can trigger keyboard shortcuts to snap and resize the active window (`Ctrl+Win+Panah` for half-screen, and `Ctrl+Win+Enter` for full-screen). Inter-monitor movement is explicitly deferred to native Windows shortcuts (`Win+Shift+Arrow`).
  - **success:** The active window instantly resizes and repositions according to the shortcut pressed, respecting current monitor DPI scaling and screen edge boundaries without inter-monitor translation overhead.
- **CAP-3**
  - **intent:** System loads user preferences and shortcut mappings from a local TOML configuration file stored in a writable user directory (e.g., `%APPDATA%\WinTick`).
  - **success:** Changes to the TOML configuration are parsed and applied to active keyboard hooks and snapping behaviors, avoiding Administrator write-permission conflicts.
- **CAP-4**
  - **intent:** System monitors and controls windows running with elevated (Administrator) privileges (e.g., Task Manager, Command Prompt).
  - **success:** Keyboard hook and window focus transitions work seamlessly on Administrator-level windows, bypassing Windows User Interface Privilege Isolation (UIPI) restrictions.
- **CAP-5**
  - **intent:** System handles configuration and user onboarding via a decoupled settings binary (`wintick-settings.exe`).
  - **success:** The settings GUI binary can be launched on-demand or automatically on first-run by the elevated daemon. It runs elevated (inheriting privileges from the daemon parent) silently without triggering consecutive UAC prompts. It features a First-Run interactive onboarding simulation with an explicit 'Skip Tutorial' option and registers the startup task.
- **CAP-6**
  - **intent:** System alerts the user of critical silent failures (e.g., OS dropping the global keyboard hook) via an unignorable OS notification.
  - **success:** When the daemon detects that the keyboard hook is silently detached, it instantly fires a native Windows Toast Notification to the user, bypassing the flaw of hidden Windows 11 System Tray icons.
- **CAP-7**
  - **intent:** System preserves user spatial layout by restricting window rotation to the exact same physical monitor and virtual desktop.
  - **success:** Rotation only cycles through windows that share the same monitor and virtual desktop as the currently active window.
- **CAP-8**
  - **intent:** System safely bypasses interception when the active window is a virtual machine or remote desktop client.
  - **success:** Known VM/Remote Desktop processes (e.g., `mstsc.exe`, `vmconnect.exe`) receive the raw shortcut input without interference. The bypass list is user-configurable via `config.toml`.
- **CAP-9**
  - **intent:** System robustly recovers from Windows Explorer crashes and handles errors silently.
  - **success:** The Tray Icon reappears automatically after `explorer.exe` restarts. Terminal initialization errors show max 1 popup; runtime errors are logged silently without popups.
- **CAP-10**
  - **intent:** User can configure the daemon to automatically launch on Windows boot without triggering UAC prompts on reboot.
  - **success:** A Windows Scheduled Task is created or removed via a toggle in the settings GUI. The task runs at logon for the specific active `%USERNAME%` with highest privileges, launching the daemon elevated and silently on boot.
- **CAP-11**
  - **intent:** User can access runtime diagnostic logs from the System Tray.
  - **success:** The tray context menu includes a "View Logs" option that opens the silent log file location for user self-diagnosis.

## Constraints

- **Rust Programming Language:** The utility must be built in Rust to satisfy strict constraints: memory usage < 2MB RAM and idle CPU utilization near 0%.
- **Binary Decoupling (Separation of Concerns):** The application must be split into two binaries: a headless Win32 daemon (`wintick.exe`) running under 2MB RAM, and an on-demand UI binary (`wintick-settings.exe`) for settings.
- **Administrator Elevation & Hardening:** The compiled daemon binary must run with Administrator privileges (via manifest `requireAdministrator` and Task Scheduler) to bypass UIPI. It must be installed in `%ProgramFiles%\WinTick` protected by admin-only ACLs to prevent Privilege Escalation. It must call `SetDllDirectoryW(L"")` on start, and the Scheduled Task `Start in` parameter must be empty or direct to the secure install path to mitigate DLL Hijacking.
- **APPDATA Alignment:** The Scheduled Task must be registered to run under the specific active `%USERNAME%` account with highest privileges (not SYSTEM), ensuring the `%APPDATA%` path matches for both daemon and settings GUI.
- **Logon Hook Retry Loop:** The daemon must perform up to 5 hook initialization attempts (1-second delay) on startup if `SetWindowsHookExW` returns `NULL`, mitigating logon race conditions with the Desktop Window Manager (DWM).
- **Single Instance Lock:** The daemon must check for and register a named mutex `Global\WinTickSingleInstanceMutex` on startup and exit immediately if another instance is detected, preventing conflicts in fast user switching sessions.
- **Asynchronous Hook Architecture:** The keyboard hook implementation must run on an independent thread with `THREAD_PRIORITY_TIME_CRITICAL` priority, decoupling key-press interception from the main window focus/activation logic. The hook callback must execute in under 10ms to prevent the OS from timing out (`LowLevelHooksTimeout`).
- **Anti-Macro Throttle:** The hook thread must silently reject rapid consecutive shortcut inputs (e.g., <50ms interval) to protect the buffer from artificial macro spam.
- **Zero-Allocation Ring Buffer:** Cross-thread communication must strictly use a statically sized lock-free ring buffer (max 16 slots) containing only primitive `u8` types. Heap-allocated objects are explicitly forbidden to ensure zero memory fragmentation and static RAM <2MB.
- **Stateless Window Ordering:** The application is prohibited from caching the window Z-order internally. It must query the native OS Z-order dynamically in real-time during keypress to prevent desynchronization.
- **Kernel-API Sterilization:** The `EnumWindows` callback in the worker thread must absolutely not use cross-process blocking APIs (e.g., `SendMessage`, `GetWindowText`). It must rely exclusively on non-blocking Kernel APIs (e.g., `IsWindowVisible`, `GetWindowThreadProcessId`) to prevent cascading hangs.
- **Graceful Fail on Invalid Target:** If a target window is closed or becomes invalid during the asynchronous delay between hook signal and worker execution, the worker thread must silently skip it and advance to the next valid window in Z-order without crashing or raising an error.
- **Compiler Profiling:** The daemon must use `windows-sys` crate (not `windows`) to avoid COM abstraction bloat. Release profile must apply `lto=true`, `opt-level="z"`, `strip=true`, `panic="abort"`. Target binary size: <500KB.
- **UX Honesty:** The application must strictly cycle through windows in their raw state. If a window is "Not Responding" (hung), it must still receive focus during the cycle without any automated bypass/skip features.

## Non-goals

- **Visual Switcher Overlay:** The utility will not display an on-screen preview or menu overlay (like the Alt-Tab window switcher) during cycling. Focus transition must be direct and instant.
- **Cloud Configuration Sync:** Configuration is strictly local and file-based. No cloud sync, remote updates, or network telemetry features are included.
- **Overlapping Stack Layout (P2 — Deferred):** The overlapping-stack window arrangement for small monitors is a secondary feature (P2/SHOULD) and is not required for the initial release. Architecture should accommodate it without blocking core functionality.

## Success signal

- The compiled Rust executable runs as a background process from the system tray with < 2MB RAM usage, successfully hook-cycling same-application windows (including elevated windows) via `Win + Backtick` and snapping them instantly without any keyboard hook dropouts.

