---
stepsCompleted: ["step-01", "step-02"]
inputDocuments:
  - _bmad-output/specs/spec-wintick/SPEC.md
  - _bmad-output/planning-artifacts/prds/prd-WinTick-2026-07-06/prd.md
  - _bmad-output/planning-artifacts/architecture/architecture-WinTick-2026-07-06/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-WinTick-2026-07-06/EXPERIENCE.md
  - _bmad-output/planning-artifacts/implementation-readiness-report-2026-07-13.md
  - _bmad-output/implementation-artifacts/2-1-asynchronous-keyboard-hook-foundation.md
  - _bmad-output/implementation-artifacts/sprint-status.yaml
---

# WinTick - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for WinTick, decomposing the requirements from the PRD, UX design, architecture, current implementation evidence, and parallel-delivery constraints into independently verifiable deliverables.

## Requirements Inventory

### Functional Requirements

FR-1: Cycle only windows belonging to the same application as the active window, using executable-name identity rather than PID as the primary identity.
FR-2: Restrict cycling to the same physical monitor and current virtual desktop as the active window.
FR-3: Pass configured shortcuts through untouched when a known VM or Remote Desktop client is active.
FR-4: Preserve UX honesty by allowing a Not Responding window to receive focus instead of silently skipping it.
FR-5: Exclude minimized, hidden ghost, `WS_EX_TOOLWINDOW`, and system-overlay windows from cycling.
FR-6: Trigger only on an exact shortcut match; additional or missing modifiers must not activate the command.
FR-7: Load configurable primary and fallback shortcuts from `%APPDATA%\WinTick\config.toml`, with `Win+Backtick` and `Alt+Backtick` defaults.
FR-8: Run the daemon elevated so it can control elevated windows across UIPI boundaries.
FR-9: Run as a headless, native-Win32, tray-resident background utility.
FR-10: Recreate the tray icon after an `explorer.exe` restart by handling `TaskbarCreated`.
FR-11: Implement the three-tier error protocol: one startup-fatal popup, silent runtime warning plus tray dot, and critical hook-dead tray X plus one toast per episode.
FR-12: Provide a View Logs tray action that opens the diagnostic log.
FR-13: Provide elevated auto-start through Windows Task Scheduler for the active user with an absolute executable path and safe working-directory behavior.
FR-14: Provide DPI-aware keyboard shortcuts for half-screen and maximize window snapping.
FR-15: Provide a DPI-aware overlapping-stack layout for at most three same-application windows.
FR-16: Provide the complete ordered tray menu: Settings, View Logs, Auto-Start, Check for Updates, About, and Exit, including separators.
FR-17: Provide first-run interactive shortcut training with a dummy window and Skip Tutorial action.
FR-18: Provide a shortcut-capture listening mode that captures physical key combinations rather than text.
FR-19: Adapt Settings, About, and onboarding visual surfaces to the native OS light/dark theme.
FR-20: Make all Settings interactions reachable by keyboard navigation.
FR-21: Expose toggle and shortcut-input state to Windows screen readers through an explicit accessibility integration.

### NonFunctional Requirements

NFR1: The daemon should target less than 2 MB static RAM and must remain below the 10 MB hard limit.
NFR2: Daemon idle CPU usage must remain approximately zero.
NFR3: The daemon release binary must remain below 500 KB, with a 250–400 KB target and aggressive release profile.
NFR4: The low-level keyboard hook must run on a dedicated `THREAD_PRIORITY_TIME_CRITICAL` thread and complete each callback in less than 10 ms.
NFR5: Hook-to-Worker communication must use an effective 16-slot, static, lock-free SPSC ring containing only primitive `u8` commands, dropping the newest command without blocking when full.
NFR6: The Hook Thread must reject shortcut repeats less than 50 ms apart without blocking.
NFR7: Window Z-order must be enumerated live for every accepted cycle command; internal Z-order caching is prohibited.
NFR8: Window filtering must use only non-blocking OS APIs; cross-process blocking calls such as `SendMessage` and `GetWindowText` are prohibited.
NFR9: A target that becomes invalid between enumeration and activation must be skipped gracefully without a crash or user-facing runtime error.
NFR10: Perceived end-to-end window-rotation latency targets less than 1 ms and must be measured separately from the hook-callback budget.
NFR11: The hook must exhibit zero dropout over process lifetime; `RegisterHotKey` is prohibited and does not satisfy this requirement.

### Additional Requirements

- AR-01: Preserve the three-crate Cargo workspace: `daemon`, `settings`, and `shared`, with dependency direction from both binaries to `shared`.
- AR-02: Preserve actor/message-passing ownership: Hook Thread owns interception state, Worker owns window operations, Settings owns GUI/config editing, and cross-actor shared mutable state is prohibited.
- AR-03: Use a wake-only Worker notification; command payloads travel only through the SPSC ring.
- AR-04: Use real-time `EnumWindows`, executable-name matching, non-blocking filter APIs, and `IVirtualDesktopManager` for spatial isolation.
- AR-05: Settings writes `config.toml` to completion and explicitly signals `WM_APP_RELOAD_CONFIG`; file watching and polling are prohibited.
- AR-06: VM/RDP bypass evaluation belongs to the Hook Thread and must return through `CallNextHookEx` without input reinjection.
- AR-07: Preserve heartbeat, install-before-unhook recovery, one-toast-per-episode behavior, and clean Hook Thread shutdown.
- AR-08: Keep the daemon on `windows-sys`; any COM exception must be minimal and justified by virtual-desktop integration.
- AR-09: The daemon launches the decoupled settings process through `ShellExecute`; GUI lifecycle must never block the hook path.
- AR-10: User education must explain spatial preservation, why cycling stays on one monitor/virtual desktop, and how the behavior differs from `Alt+Tab`.
- AR-11: Before Settings accessibility implementation begins, architecture must lock an explicit `egui`/Windows accessibility mechanism rather than assuming screen-reader support.
- AR-12: Existing Story 2.1 lifecycle, ring, callback, and Worker-drain contracts remain a frozen upstream boundary for subsequent decomposition.
- PD-01: Every implementation deliverable must expose a concrete internal output and an independently runnable verification gate; it need not be a complete user-facing feature.
- PD-02: Parallel lanes must have no unresolved upstream dependency and must declare module/file ownership plus shared-contract ownership.
- PD-03: A shared contract must be frozen before two or more implementation lanes consume it in parallel.
- PD-04: Any fan-in that depends on two or more upstream lanes must be represented by a dedicated convergence story with explicit entry and exit gates.
- PD-05: Each implementation lane must use its own inter-agent workflow ID and complete Codex → Cursor → Antigravity evidence flow.
- PD-06: A convergence story becomes eligible only after every required upstream review target has been accepted.
- PD-07: Simultaneous production edits in one shared worktree are prohibited unless file ownership is disjoint; otherwise use separate Git worktrees.
- PD-08: Unit, contract, adapter, policy, and harness deliverables should be preferred where they enable isolated testing and shorten the critical path.

### UX Design Requirements

UX-DR1: Window cycling and snapping feedback must remain invisible: no overlay, animation, or transition UI.
UX-DR2: The tray icon must expose Normal, Warning/Logged, and Critical/Dead visual states using the defined alert color semantics.
UX-DR3: Settings must use a native-feeling modular layout, Segoe UI typography, and feature-oriented grouping.
UX-DR4: Shortcut fields must enter an explicit listening state and capture physical key combinations.
UX-DR5: Error and warning microcopy must be neutral, direct, and solution-oriented.
UX-DR6: All interactive Settings controls must be reachable and operable through keyboard navigation.
UX-DR7: Toggles and shortcut controls must expose meaningful state to Windows screen readers.
UX-DR8: Settings, About, and onboarding must adapt to the native OS light/dark theme.
UX-DR9: First run must provide an interactive shortcut simulation with a dummy-window exercise and an explicit Skip Tutorial action.
UX-DR10: The tray context menu must preserve the specified item order and separator grouping.
UX-DR11: Settings must remain a decoupled on-demand process whose startup, rendering, and shutdown cannot delay the daemon hook path.

### Source Reconciliation Decisions

- SR-01: Executable-basename identity from AD-4 and the accepted requirement inventory supersedes older PRD wording that suggested PID or application class as primary same-application identity.
- SR-02: The final SPEC defers inter-monitor movement to native `Win+Shift+Arrow`; older PRD and UX references to a WinTick inter-monitor command are non-binding.
- SR-03: AD-5 explicit `WM_APP_RELOAD_CONFIG` signaling supersedes UX prose that loosely described detecting file changes; polling and file watching remain prohibited.
- SR-04: AD-11's `egui` Settings process supersedes the DESIGN phrase "native Win32 standard dialog"; native-feeling visual and accessibility behavior remains required even though the toolkit is `egui`.
- SR-05: The accepted startup policy may retry Hook installation at most five times; AD-7's Tier-1 rule governs the single terminal popup and exit after retries are exhausted rather than prohibiting bounded initialization retries.

### FR Coverage Map

FR-1: Epic 2 - Cycle only windows belonging to the active application.
FR-2: Epic 3 - Preserve physical-monitor and virtual-desktop context.
FR-3: Epic 3 - Pass shortcuts through to VM and Remote Desktop clients.
FR-4: Epic 2 - Preserve UX honesty for Not Responding windows.
FR-5: Epic 2 - Exclude minimized, ghost, tool, and overlay windows.
FR-6: Epic 2 - Require exact shortcut matching.
FR-7: Epic 5 - Configure primary and fallback shortcuts.
FR-8: Epic 1 - Operate with Administrator elevation and UIPI reach.
FR-9: Epic 1 - Run as a headless tray-resident daemon.
FR-10: Epic 1 - Recover the tray icon after Explorer restarts.
FR-11: Epic 1 - Apply the three-tier error protocol.
FR-12: Epic 1 - Open diagnostic logs through the tray.
FR-13: Epic 1 - Manage elevated auto-start through Task Scheduler.
FR-14: Epic 4 - Provide DPI-aware window snapping.
FR-15: Epic 4 - Provide DPI-aware overlapping-stack layout.
FR-16: Epic 1 - Provide the complete ordered tray context menu.
FR-17: Epic 5 - Provide interactive first-run shortcut training.
FR-18: Epic 5 - Capture shortcuts through physical-key listening mode.
FR-19: Epic 5 - Adapt visual surfaces to the OS theme.
FR-20: Epic 5 - Support complete keyboard navigation.
FR-21: Epic 5 - Expose control state to Windows screen readers.

## Epic List

### Epic 1: Reliable Background Presence & Recovery

Users have a quiet, elevated, tray-resident utility that starts safely, survives Explorer restarts, exposes diagnostics and lifecycle controls, and communicates failures without runtime popup spam.

**FRs covered:** FR-8, FR-9, FR-10, FR-11, FR-12, FR-13, FR-16

**Implementation notes:** This epic represents the already-delivered daemon, tray, auto-start, and health foundation. Subsequent epics must preserve its lifecycle and error-state contracts.

### Epic 2: Instant Same-Application Cycling

Users can switch instantly and predictably among visible windows of the active application with an exact shortcut, while minimized or ghost windows are excluded and hung windows remain honestly visible.

**FRs covered:** FR-1, FR-4, FR-5, FR-6

**Implementation notes:** This epic owns the critical Hook-to-Worker and live-window-candidate contract. Its accepted contracts enable sibling delivery lanes without requiring later epics.

### Epic 3: Context-Safe Cycling

Users remain within the intended monitor and virtual desktop while cycling, and shortcuts pass through untouched when their active context is a VM or Remote Desktop client.

**FRs covered:** FR-2, FR-3

**Implementation notes:** Spatial policy and VM/RDP bypass are separable implementation lanes. This epic consumes only accepted contracts from Epics 1 and 2 and does not require window-arrangement or Settings functionality.

### Epic 4: Fast Window Arrangement

Users can arrange the active window through DPI-aware snapping and can organize up to three related windows in a usable overlapping stack.

**FRs covered:** FR-14, FR-15

**Implementation notes:** Snap policy and stack-layout mathematics can be verified independently. This epic builds on accepted command and window-candidate contracts from Epic 2 and can proceed as a sibling of Epic 3.

### Epic 5: Personalized & Accessible Experience

Users can configure shortcuts through a native-feeling accessible Settings application and learn the product through a first-run interactive experience without burdening the background daemon.

**FRs covered:** FR-7, FR-17, FR-18, FR-19, FR-20, FR-21

**Implementation notes:** Settings shell, accessibility, shortcut capture, and onboarding can progress as isolated UI lanes after the shared config schema and accessibility mechanism are frozen. Daemon live reload is an explicit later convergence point rather than an implicit cross-lane dependency.

## Epic 1: Reliable Background Presence & Recovery

Users have a quiet, elevated, tray-resident utility that starts safely, survives Explorer restarts, exposes diagnostics and lifecycle controls, and communicates failures without runtime popup spam.

### Story 1.1: Three-Crate Workspace and Hardened Build Pipeline

As a project maintainer,
I want daemon, Settings, and shared contracts separated in a hardened Cargo workspace,
So that each runtime can evolve and be verified without bloating or destabilizing the background daemon.

**Acceptance Criteria:**

**Given** a clean checkout with the supported Rust toolchain
**When** the workspace is built
**Then** it contains independently buildable `daemon`, `settings`, and `shared` crates
**And** both binaries depend on shared contracts through the `shared` crate rather than duplicated definitions.

**Given** `build.ps1` is invoked in development or production mode
**When** the build completes
**Then** it produces the appropriate daemon and Settings executables
**And** the production daemon uses `lto=true`, `opt-level="z"`, `strip=true`, and `panic="abort"`
**And** its target range is 250–400 KB
**And** it remains below the 500 KB hard limit
**And** a result above 400 KB is recorded as an explicit target miss even when the hard limit still passes.

**Given** the daemon starts from its protected installation location
**When** startup hardening runs
**Then** its manifest requests Administrator elevation
**And** `SetDllDirectoryW(L"")` is applied before runtime integrations initialize.

**Given** Story 1.1 is verified independently
**When** workspace checks and development and production builds are run
**Then** workspace compilation succeeds without requiring tray, hook, cycling, layout, or Settings feature completion.

### Story 1.2: Elevated Single-Instance Daemon

As a power user,
I want WinTick to establish one elevated and resilient daemon instance,
So that it can control elevated windows without duplicate hooks or competing background processes.

**Acceptance Criteria:**

**Given** the user launches `wintick.exe`
**When** Windows starts the process
**Then** the executable requests Administrator elevation through its manifest
**And** the running process has a High integrity level suitable for UIPI-protected window operations.

**Given** no WinTick daemon is running
**When** startup acquires `Global\WinTickSingleInstanceMutex`
**Then** the process becomes the sole daemon instance.

**Given** another daemon already owns the mutex or the current process cannot access an existing global mutex
**When** a second daemon starts
**Then** the second process exits cleanly without installing another hook or tray icon.

**Given** hook initialization temporarily fails during startup
**When** the initialization policy executes
**Then** it performs at most five attempts separated by one-second delays
**And** terminal failure produces exactly one fatal message before process exit.

**Given** Story 1.2 is verified independently after Story 1.1
**When** integrity, single-instance, retry, and elevated-window smoke gates run
**Then** they pass without requiring tray menu, health monitoring, window cycling, layout, or Settings functionality.

### Story 1.3: Native Tray Presence and Explorer Recovery

As a desktop user,
I want WinTick to remain unobtrusive in the System Tray and recover after Explorer restarts,
So that the utility stays available without leaving background windows or stale icons.

**Acceptance Criteria:**

**Given** the elevated single-instance daemon starts successfully
**When** its hidden-window message loop becomes operational
**Then** it registers a native Win32 tray icon without displaying a console or application window
**And** normal operation is represented by the standard tray state.

**Given** the tray icon is registered
**When** WinTick negotiates its notification behavior
**Then** it uses the supported notification-icon version and preserves the expected callback semantics.

**Given** `explorer.exe` restarts and removes notification icons
**When** the daemon receives the registered `TaskbarCreated` broadcast
**Then** it recreates exactly one WinTick tray icon
**And** the daemon continues running without reinstalling unrelated runtime components.

**Given** the daemon shuts down normally
**When** its hidden window is destroyed
**Then** the tray icon and owned native resources are removed
**And** no stale or ghost icon remains after Explorer refreshes the tray.

**Given** Story 1.3 is verified after Stories 1.1 and 1.2
**When** tray registration, simulated Explorer restart, duplicate prevention, and cleanup gates run
**Then** they pass without requiring menu actions, error-state transitions, window cycling, layout, or Settings functionality.

### Story 1.4: Tray Controls, Diagnostics, and Auto-Start

As an advanced user,
I want lifecycle controls, diagnostics, and auto-start management available from the tray,
So that I can manage WinTick without editing files or using command-line tools.

**Acceptance Criteria:**

**Given** the WinTick tray icon is active
**When** the user opens its context menu
**Then** the menu presents `Settings...`, `View Logs`, `Auto-Start`, `Check for Updates...`, `About`, and `Exit` in the specified order and separator groups
**And** it appears at the tray callback anchor under DPI scaling
**And** the first click outside dismisses it.

**Given** the context menu is open
**When** the user selects `Settings...`
**Then** the decoupled Settings executable is launched through `ShellExecute`
**And** daemon message processing is not blocked by the Settings process lifetime.

**Given** the user selects `View Logs`
**When** the diagnostic file exists or must be initialized
**Then** `%APPDATA%\WinTick\wintick.log` is opened through the OS-associated text viewer without displaying a daemon runtime popup.

**Given** Auto-Start is disabled
**When** the user enables it
**Then** WinTick creates a Task Scheduler entry for the active user with `ONLOGON`, highest privileges, an absolute quoted executable path, and no unsafe working directory.

**Given** the scheduled task already exists
**When** the menu opens or the user disables Auto-Start
**Then** the check state reflects the actual Task Scheduler state
**And** disabling it removes the scheduled task cleanly.

**Given** the user selects `Exit`
**When** the shutdown action is dispatched
**Then** the hidden window is destroyed through the normal cleanup path
**And** tray resources are released without leaving a ghost icon.

**Given** Story 1.4 is verified after Story 1.3
**When** menu-construction tests and elevated runtime checks for DPI anchoring, View Logs, Settings launch, Task Scheduler arguments/state, dismissal, and Exit run
**Then** they pass without requiring error-state escalation, cycling, layout, or completed Settings UI functionality.

### Story 1.5: Three-Tier Error Protocol and Background-Service Convergence

As a reliability-conscious user,
I want WinTick to report startup failures, runtime warnings, and hook failures according to their severity,
So that I know when the background service needs attention without repeated popup interruptions.

**Acceptance Criteria:**

**Given** an unrecoverable failure occurs during daemon startup
**When** all applicable startup retries are exhausted
**Then** WinTick displays exactly one fatal `MessageBox`
**And** exits without leaving a hook, tray icon, or background process.

**Given** a recoverable runtime warning occurs after startup
**When** the warning is reported
**Then** WinTick appends diagnostic detail to `%APPDATA%\WinTick\wintick.log`
**And** changes the tray state to Warning with a red-dot indicator
**And** does not display a runtime popup.

**Given** the keyboard hook is healthy
**When** the ten-second health interval elapses
**Then** WinTick refreshes the hook using install-before-unhook behavior
**And** avoids an interval in which no hook is installed.

**Given** three consecutive hook-refresh attempts fail
**When** the third failure is confirmed
**Then** the tray state becomes Critical with a red-X indicator
**And** exactly one toast notification is displayed for that failure episode
**And** repeated checks during the same episode do not produce additional toasts.

**Given** the hook recovers after a Critical episode
**When** a refresh succeeds
**Then** the consecutive-failure counter and per-episode toast guard are reset
**And** the tray returns to Warning when a runtime warning remains latched
**And** otherwise returns to Normal.

**Given** the tray is already in the Critical state
**When** a runtime warning is recorded
**Then** the warning is logged and latched
**And** it does not downgrade or visually replace the Critical state.

**Given** Stories 1.2 through 1.4 are complete
**When** the Epic 1 convergence suite runs in an elevated environment
**Then** it verifies single-instance enforcement, tray recovery, tray controls, auto-start, all three error tiers, hook recovery, severity precedence, and clean shutdown
**And** it passes without requiring cycling, window arrangement, or completed Settings UI functionality.

## Epic 2: Instant Same-Application Cycling

Users can switch instantly and predictably among visible windows of the active application with an exact shortcut, while minimized or ghost windows are excluded and hung windows remain honestly visible.

### Story 2.1: Asynchronous Keyboard Hook Foundation

As a WinTick user,
I want keyboard shortcuts intercepted by a dedicated, time-critical Hook Thread,
So that Windows never drops the hook because window-processing work delayed the callback.

**Acceptance Criteria:**

**Given** the elevated daemon is starting
**When** the keyboard subsystem initializes
**Then** a dedicated Hook Thread creates its message queue, applies `THREAD_PRIORITY_TIME_CRITICAL`, installs `WH_KEYBOARD_LL`, and exclusively owns the `HHOOK` lifecycle
**And** initialization uses at most five attempts separated by one second
**And** terminal failure produces exactly one fatal popup before exit
**And** only one production hook remains active.

**Given** primary and fallback shortcuts have been parsed from shared configuration
**When** physical keyboard events reach the callback
**Then** only an exact modifier and main-key match produces `Command::Cycle`
**And** missing or additional modifiers do not match
**And** injected, unmatched, or negative-code events pass immediately to `CallNextHookEx`
**And** modifier state is Hook Thread-owned without `GetAsyncKeyState`.

**Given** the hook is active
**When** its callback handles an event
**Then** it performs no allocation, logging, I/O, locking, sleeping, configuration parsing, enumeration, focus operation, or blocking cross-process call
**And** elevated runtime evidence measures the callback below 10 ms
**And** debug instrumentation is absent from release builds.

**Given** the Hook-to-Worker ring is empty
**When** commands are published and consumed
**Then** all 16 slots store usable FIFO `u8` commands
**And** a seventeenth push fails immediately without blocking, overwriting data, or advancing the producer cursor.

**Given** a command has entered the ring
**When** the Hook Thread wakes the Worker
**Then** `WM_APP_COMMAND_READY` carries zero-valued `wParam` and `lParam`
**And** the existing hidden-window/main thread drains the ring to empty
**And** this foundation performs placeholder dispatch without enumerating or focusing windows.

**Given** an exact shortcut repeats
**When** its interval is below 50 ms
**Then** the new command is rejected before publication
**And** an interval of exactly 50 ms is accepted
**And** matched events rejected by throttle or full capacity remain swallowed
**And** the matching main-key release and required Win-key release are swallowed without blocking modifier events.

**Given** Epic 1 hook-health behavior already exists
**When** ownership moves to the dedicated Hook Thread
**Then** the ten-second heartbeat, install-before-unhook refresh, keep-old-on-failure behavior, three-failure Critical escalation, warning precedence, recovery reset, and one toast per episode remain intact
**And** shutdown cannot reinstall the hook after final unhook
**And** the Hook Thread exits and is joined without leaving a live hook.

### Story 2.2: Deterministic Cycling Candidate Contract and Harness

As a WinTick user,
I want every cycling component to interpret window candidates and failures through one deterministic contract,
So that parallel implementation cannot change which window is considered next.

**Acceptance Criteria:**

**Given** the frozen Story 2.1 Worker-drain boundary
**When** the cycling contract is defined
**Then** it represents an ordered candidate, captured window facts, active-window identity, eligibility decision, selection result, and activation result without shared mutable state
**And** these types remain internal to the daemon Worker domain rather than expanding the cross-crate `shared` API.

**Given** a process path is available for a window
**When** application identity is normalized
**Then** identity is the case-insensitive executable basename
**And** PID is used only to query the process and never as the primary same-application identity
**And** an inaccessible or vanished process produces an unavailable identity without a crash or runtime popup.

**Given** the Worker begins processing one accepted `Command::Cycle`
**When** the contract captures the active context
**Then** `GetForegroundWindow` is sampled once at the start of that command
**And** candidate order represents one fresh top-to-bottom Z-order snapshot
**And** activation may attempt each eligible target at most once before terminating.

**Given** eligibility policy needs deterministic vocabulary
**When** the contract fixtures are frozen
**Then** hidden, iconic, `WS_EX_TOOLWINDOW`, `Ghost`, `Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `Progman`, and `WorkerW` cases have explicit expected decisions
**And** a real application window marked Not Responding remains eligible without a responsiveness probe.

**Given** discovery, eligibility, and activation implementations do not yet exist
**When** the deterministic harness runs
**Then** injected snapshots and fake activation outcomes verify ordering, executable normalization, exclusion decisions, wrap behavior, invalid-target continuation, and one-pass termination
**And** the harness requires neither a global hook nor a live desktop.

**Given** Story 2.2 has passed its contract and harness gate
**When** parallel implementation begins
**Then** live discovery, eligibility policy, and selection/activation use separate workflow IDs and Git worktrees
**And** discovery exclusively owns `cycling/source.rs`
**And** eligibility exclusively owns `cycling/eligibility.rs`
**And** selection and activation exclusively own `cycling/selection.rs` and `cycling/activation.rs`
**And** `cycling/mod.rs`, `worker.rs`, shared-contract close work, and final composition remain reserved for convergence
**And** each lane completes its own Codex-to-Cursor-to-Antigravity evidence chain before convergence eligibility.

### Story 2.3: Live Z-Order and Executable-Identity Discovery

As a multitasking user,
I want WinTick to discover the current windows of my active application at the moment I invoke cycling,
So that stale process or Z-order information never sends me to the wrong application.

**Acceptance Criteria:**

**Given** Story 2.2's accepted candidate contract
**When** the discovery adapter receives a cycle request
**Then** it invokes `EnumWindows` exactly once for that request
**And** preserves the returned top-to-bottom Z-order
**And** does not retain a Z-order or window cache between commands.

**Given** the active window and candidate windows belong to separate processes with the same executable basename
**When** their identities are resolved
**Then** they are classified as the same application using a case-insensitive basename comparison
**And** different PIDs do not prevent grouping
**And** matching PIDs alone cannot make different executable identities equivalent.

**Given** a process closes, denies query access, or changes state during discovery
**When** its metadata cannot be resolved through `GetWindowThreadProcessId`, `OpenProcess`, and `QueryFullProcessImageNameW`
**Then** that candidate is represented as unavailable or omitted according to the frozen contract
**And** discovery continues without a crash, blocking retry, or user-facing runtime error.

**Given** discovery executes in the Worker domain
**When** its Win32 calls are audited
**Then** it uses only non-blocking metadata APIs
**And** it does not call `SendMessage`, `GetWindowText`, focus APIs, eligibility policy, or Hook Thread code.

**Given** Story 2.3 is verified independently after Story 2.2
**When** adapter tests and a helper-window runtime harness execute
**Then** each command produces a fresh ordered snapshot with correct multi-process executable grouping
**And** no eligibility, activation, monitor, virtual-desktop, or UI implementation is required.

### Story 2.4: Eligible-Window Filtering and UX Honesty

As a desktop user,
I want cycling to include only real visible application windows while still exposing a hung window,
So that the cycle remains useful without hiding application failures from me.

**Acceptance Criteria:**

**Given** an injected candidate fact set from the Story 2.2 contract
**When** the pure eligibility policy evaluates it
**Then** visible non-iconic top-level application windows remain eligible
**And** hidden, minimized, `WS_EX_TOOLWINDOW`, ghost-class, and defined shell-overlay windows are excluded.

**Given** the real application window is Not Responding
**When** its eligibility is evaluated
**Then** it remains eligible for activation
**And** no responsiveness probe, automated skip, restore, maximize, or replacement with a ghost surrogate occurs.

**Given** the policy evaluates captured facts
**When** its implementation is audited
**Then** it performs no live enumeration, process query, focus operation, `SendMessage`, or `GetWindowText`
**And** it has no dependency on physical-monitor, virtual-desktop, VM/RDP, arrangement, or Settings behavior.

**Given** Story 2.4 is verified independently after Story 2.2
**When** table-driven policy tests cover every frozen fixture and combinations of exclusion facts
**Then** each decision is deterministic
**And** a synthetic hung-window fact remains eligible
**And** the tests require no sibling-lane implementation.

### Story 2.5: Stateless Selection and Resilient Activation

As a WinTick user,
I want each shortcut to move focus to the next eligible window and recover from windows closing mid-cycle,
So that cycling remains predictable even while applications change.

**Acceptance Criteria:**

**Given** an ordered eligible-candidate sequence containing the active window
**When** next-target selection runs
**Then** it selects the first eligible candidate after the active window
**And** wraps to the beginning at most once
**And** a sequence containing no alternative target results in a no-op.

**Given** the active window is absent from the supplied sequence
**When** selection runs
**Then** it chooses the first eligible non-active candidate deterministically
**And** never loops indefinitely or invents a cached position.

**Given** a chosen target closes or activation fails before focus changes
**When** the activation adapter reports failure
**Then** selection advances to the next untried candidate
**And** each candidate is attempted no more than once
**And** exhaustion ends silently without a crash or runtime popup.

**Given** an eligible target is Not Responding
**When** direct activation is attempted
**Then** WinTick makes the same bounded focus attempt used for responsive windows
**And** it does not probe, skip, restore, maximize, or wait for the target to respond.

**Given** Story 2.5 is verified independently after Story 2.2
**When** pure selection tests, fake-activator tests, and a synthetic hung/closing-window harness run
**Then** next, wrap, no-op, failure continuation, and one-pass termination pass
**And** no live enumeration, eligibility implementation, Hook change, or final Worker composition is required.

### Story 2.6: Instant Same-Application Cycling Convergence

As a WinTick user,
I want the accepted hook, discovery, filtering, selection, and activation capabilities to operate as one instant cycle,
So that one exact shortcut predictably focuses the next real window of my active application.

**Acceptance Criteria:**

**Given** Stories 2.3, 2.4, and 2.5 have accepted review targets against the frozen Story 2.2 contract
**When** Story 2.6 becomes eligible
**Then** the Worker composes fresh discovery, pure eligibility, stateless selection, and bounded activation behind `Command::Cycle`
**And** sibling-lane code is not reopened except for an explicitly traced integration defect
**And** convergence uses its own workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

**Given** primary or fallback shortcuts are pressed with an exact modifier match
**When** the Hook-to-Worker path completes
**Then** cycling stays within the active executable identity across different PIDs
**And** repeated accepted commands follow live Z-order
**And** missing or additional modifiers do not trigger cycling.

**Given** the desktop contains minimized, hidden, tool, ghost, shell-overlay, responsive, hung, and rapidly closing windows
**When** integrated cycling runs
**Then** excluded windows never become targets
**And** the real hung application window receives a bounded focus attempt
**And** invalid targets are skipped once without a crash, popup, or infinite loop.

**Given** a user invokes cycling
**When** focus changes
**Then** no overlay, preview, animation, transition surface, restore, or maximize action is created
**And** the operation remains visually invisible apart from the native focus change.

**Given** an elevated performance harness has completed warm-up
**When** at least 1,000 accepted cycles are measured from Worker command receipt through activation completion
**Then** the end-to-end distribution records p50, p95, and maximum separately from hook-callback timing
**And** p95 remains below 1 ms for NFR10 to pass
**And** any miss prevents NFR10 from being marked satisfied unless an explicit approved requirement change replaces the target.

**Given** an elevated 30-minute soak includes at least 10,000 generated exact-match events, heartbeat refreshes, and normal tray activity
**When** accepted, throttled, capacity-dropped, drained, and activated counts are reconciled
**Then** no unexplained command or hook dropout occurs
**And** intentional throttle and full-ring drops remain distinguishable from failures.

**Given** Epics 3, 4, and 5 will consume shared command and configuration contracts as sibling lanes
**When** the Epic 2 extension contract is frozen
**Then** the existing `u8` values for `Cycle`, `SnapLeft`, `SnapRight`, `SnapMaximize`, and `OverlappingStack` remain stable
**And** snapping defaults remain `Ctrl+Win+Left`, `Ctrl+Win+Right`, and `Ctrl+Win+Enter`
**And** `layout.stack_shortcut` is introduced with the reviewable default `Ctrl+Win+Down`
**And** `vm_bypass.bypass_processes` retains its documented defaults
**And** backward-compatible `vm_bypass.bypass_classes` is introduced with `VMwareUnityWindow` as a documented default
**And** Story 2.6 exclusively owns the necessary `shared` schema and contract-test edits until this freeze is accepted
**And** later sibling lanes may consume but not independently renumber or reinterpret these shared fields.

**Given** the Epic 2 convergence suite completes
**When** release and resource regressions are checked
**Then** daemon idle CPU remains approximately zero
**And** measured idle daemon RAM targets less than 2 MB and remains below the 10 MB hard limit
**And** a result from 2 MB through less than 10 MB is recorded as an explicit target miss
**And** the release binary targets 250–400 KB and remains below 500 KB using the hardened release profile
**And** Epic 1 lifecycle, error-state, recovery, and shutdown gates remain green
**And** Epic 2 closes without requiring monitor, virtual-desktop, VM/RDP, arrangement, or Settings functionality.

## Epic 3: Context-Safe Cycling

Users remain within the intended monitor and virtual desktop while cycling, and shortcuts pass through untouched when their active context is a VM or Remote Desktop client.

### Story 3.1: Context-Safety Contract and Deterministic Harness

As a multi-context desktop user,
I want WinTick's spatial and shortcut-passthrough decisions governed by one stable policy contract,
So that independently delivered context-safety components behave consistently.

**Acceptance Criteria:**

**Given** the accepted Epic 2 candidate, command, and configuration contracts
**When** the context-safety contract is introduced
**Then** it defines immutable inputs and explicit outcomes for spatial eligibility and foreground bypass classification
**And** it introduces no Z-order cache, cross-actor shared mutable state, or second command-payload path.

**Given** an existing configuration contains only `vm_bypass.bypass_processes`
**When** it is loaded through the frozen backward-compatible schema
**Then** the process entries retain their documented defaults
**And** missing `bypass_classes` receives its documented default
**And** process-basename and window-class identifiers remain independently configurable.

**Given** bypass configuration has been loaded before Hook activation
**When** it is prepared for runtime evaluation
**Then** identifiers are normalized outside the callback into Hook Thread-owned immutable policy data
**And** callback evaluation requires no parsing, allocation, file I/O, logging, or lock acquisition.

**Given** deterministic fake monitor, virtual-desktop, and foreground-identity adapters
**When** the contract harness runs
**Then** it covers same and different monitors, current and non-current virtual desktops, process matches, class matches, confirmed non-matches, and adapter failures
**And** spatial uncertainty produces an ineligible decision
**And** foreground-identity uncertainty produces a conservative passthrough decision.

**Given** downstream spatial and VM/RDP lanes begin independently
**When** they compile against the frozen contract
**Then** spatial implementation exclusively owns `context/spatial.rs` and `context/virtual_desktop.rs`
**And** VM/RDP implementation exclusively owns `context/vm_bypass.rs`
**And** `context/mod.rs`, `worker.rs`, `hook.rs`, and final composition remain reserved for convergence
**And** each story uses a distinct workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence before local convergence.

**Given** Story 3.1 is verified independently
**When** schema-compatibility and deterministic context-contract tests run
**Then** they pass without installing a keyboard hook, creating a COM object, enumerating windows, or changing foreground focus.

### Story 3.2: Physical-Monitor and Virtual-Desktop Decision Adapter

As a multi-monitor Windows user,
I want WinTick to reject windows outside my current monitor and virtual desktop,
So that cycling preserves the spatial workspace I am currently using.

**Acceptance Criteria:**

**Given** an active-window origin and an ordered candidate from the accepted Epic 2 contract
**When** the spatial adapter evaluates that candidate
**Then** it resolves the origin's physical monitor once for that cycle operation
**And** accepts the candidate only when its `HMONITOR` equals the origin monitor
**And** `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` confirms membership in the current virtual desktop.

**Given** the user moves a window, switches virtual desktops, or changes focus between commands
**When** another spatial decision is requested
**Then** monitor and virtual-desktop facts are queried live
**And** no monitor, desktop, candidate, or Z-order result is retained as an authoritative cache between cycle commands.

**Given** virtual-desktop integration is initialized
**When** its production adapter is created and destroyed
**Then** COM ownership remains on the Worker actor's thread
**And** initialization, interface lifetime, and release are explicit
**And** raw or otherwise minimal COM integration is limited to the documented `IVirtualDesktopManager` exception.

**Given** monitor lookup, COM initialization, or a virtual-desktop query fails
**When** the adapter cannot prove candidate eligibility
**Then** it fails closed without selecting the candidate
**And** exposes one diagnostic outcome suitable for the Tier-2 path at a safe boundary
**And** never displays a runtime popup or causes a cross-context focus change.

**Given** fake spatial adapters and Windows integration fixtures
**When** isolated spatial tests run
**Then** same-monitor/current-desktop candidates pass
**And** different-monitor, non-current-desktop, and unavailable candidates fail closed
**And** incoming candidate order is preserved without activating a window.

**Given** Story 3.2 is verified independently after Story 3.1
**When** unit, adapter, COM-lifecycle, and Windows spatial-fixture gates run
**Then** they pass without modifying `worker.rs` or `hook.rs`
**And** without requiring VM/RDP bypass, arrangement, Settings, or convergence implementation.

### Story 3.3: VM and Remote Desktop Passthrough Decision Adapter

As a VM or Remote Desktop user,
I want WinTick to recognize when my shortcut belongs to the guest environment,
So that the physical key combination passes through without interference or synthetic reinjection.

**Acceptance Criteria:**

**Given** a foreground window owned by a configured VM or Remote Desktop process
**When** its executable basename is evaluated case-insensitively
**Then** the bypass adapter returns `Passthrough`.

**Given** a foreground window whose configured class identifies a VM or Remote Desktop surface
**When** its class is evaluated case-insensitively
**Then** the bypass adapter returns `Passthrough` even when process identity alone does not match.

**Given** a fully resolved foreground identity matches neither configured process nor class identifiers
**When** bypass evaluation completes
**Then** it returns `ContinueWinTickMatching`
**And** does not alter Epic 2 exact-shortcut, throttle, or swallowing policy.

**Given** process or class identity cannot be resolved safely
**When** the adapter cannot prove a non-bypass application
**Then** it returns `Passthrough` conservatively
**And** exposes a deferred diagnostic signal for handling outside the callback
**And** never requests `SendInput` or another reinjection mechanism.

**Given** the production identity collector runs on the Hook Thread
**When** it inspects a potential shortcut event
**Then** it uses bounded non-blocking Win32 queries and reusable fixed buffers
**And** performs no allocation, logging, config parsing, lock acquisition, sleep, or Worker call.

**Given** default identifiers, configured identifiers, mixed casing, duplicates, non-matches, and identity-query failures
**When** the deterministic bypass harness runs
**Then** every outcome matches the frozen Story 3.1 contract
**And** prepared policy remains immutable for the lifetime of the active Hook configuration.

**Given** Story 3.3 is verified independently after Story 3.1
**When** pure policy, identity-adapter, allocation-guard, and timing gates run
**Then** they pass without modifying `hook.rs` or `worker.rs`
**And** without requiring spatial filtering, arrangement, Settings, or convergence implementation.

### Story 3.4: Context-Safe Cycling Convergence

As a multi-context desktop user,
I want spatial isolation and VM/RDP passthrough to operate as one reliable cycling pipeline,
So that WinTick changes focus only when the active context permits it.

**Acceptance Criteria:**

**Given** the accepted Epic 2 convergence contract and accepted reviews for Stories 3.2 and 3.3
**When** Story 3.4 becomes eligible
**Then** the frozen spatial and bypass adapters are integrated without changing their public contracts
**And** required contract corrections are returned through their owning workflow before integration continues
**And** convergence uses its own workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

**Given** an accepted cycle command outside a bypass context
**When** the Worker evaluates live same-application candidates
**Then** it preserves Epic 2 ordering and eligibility rules
**And** activates only a candidate accepted by the spatial adapter
**And** leaves focus unchanged when no context-eligible candidate exists.

**Given** same-application windows exist across multiple physical monitors and virtual desktops
**When** the user repeatedly invokes the exact shortcut
**Then** only windows on the origin monitor and current virtual desktop participate
**And** windows outside either boundary never receive focus.

**Given** a configured VM or Remote Desktop process or class is foreground when a shortcut chord begins
**When** physical key-down and key-up events are processed
**Then** the entire chord passes through `CallNextHookEx` without ring publication, Worker wake-up, throttle advancement, or WinTick swallow state
**And** passthrough remains coherent through the corresponding releases even if foreground focus changes during the chord
**And** no input is synthetically reinjected.

**Given** the foreground application is confirmed not to be bypassed
**When** the same shortcut is pressed
**Then** Epic 2 exact-match, anti-macro, ring, selection, activation, invalid-target, and key-swallowing contracts remain unchanged.

**Given** a same-context candidate is Not Responding
**When** cycling reaches it
**Then** it remains eligible for a bounded foreground attempt
**And** minimized, ghost, tool, hidden, and overlay windows remain excluded.

**Given** a spatial or identity adapter reports an operational failure
**When** convergence handles the outcome
**Then** no cross-context activation or partial shortcut interception occurs
**And** the condition reaches the Tier-2 logging and tray-warning path outside the callback
**And** no runtime popup is displayed.

**Given** the Epic 3 elevated convergence matrix runs
**When** it exercises different applications, monitors, virtual desktops, VM/RDP identities, hung windows, disappearing targets, exact and extra modifiers, and adapter failures
**Then** every context-safety and passthrough case passes without overlay, animation, or transition UI
**And** hook-callback timing remains below 10 ms
**And** rotation latency is measured separately against the less-than-1-ms target
**And** release size, idle CPU, memory, and zero-dropout regressions remain within their established gates.

**Given** Epic 3 convergence is accepted
**When** its user-facing documentation is finalized
**Then** spatial preservation is described as intentionally limited to one physical monitor and the current virtual desktop
**And** its distinction from global `Alt+Tab` behavior is explicit.

**Given** Epic 3, Epic 4, or Epic 5 convergence diffs touch the same daemon hotspot
**When** their accepted worktrees are prepared for canonical merge
**Then** a cross-epic release/evidence gate serializes those hotspot merges
**And** reruns combined qualification after each merge
**And** no additional epic is created solely for that release gate.

## Epic 4: Fast Window Arrangement

Users can arrange the active window through DPI-aware snapping and can organize up to three related windows in a usable overlapping stack.

### Story 4.1: Arrangement Contract and Deterministic Harness

As a desktop organizer,
I want snapping and stack commands to use one stable arrangement contract,
So that window-layout capabilities can be delivered independently without destabilizing keyboard interception or cycling.

**Acceptance Criteria:**

**Given** the accepted Epic 2 command contract
**When** arrangement commands are consumed
**Then** `SnapLeft`, `SnapRight`, `SnapMaximize`, and `OverlappingStack` retain their frozen primitive `u8` values
**And** unknown values continue to decode as `Nop`
**And** no command payload is added to the wake-only Worker notification.

**Given** default or partial configuration is loaded
**When** arrangement shortcuts are resolved
**Then** half-left defaults to `Ctrl+Win+Left`, half-right to `Ctrl+Win+Right`, maximize to `Ctrl+Win+Enter`, and stack to the frozen `Ctrl+Win+Down`
**And** stack remains disabled by default
**And** the default stack width remains 50 percent.

**Given** the older PRD mentions inter-monitor movement
**When** the final SPEC and architecture scope are applied
**Then** WinTick implements no inter-monitor arrangement command
**And** movement between monitors remains delegated to native `Win+Shift+Arrow`.

**Given** an arrangement shortcut is malformed or collides with another accepted command
**When** validation runs outside the Hook callback
**Then** the invalid mapping produces a Tier-2 warning
**And** the last-known-good mapping or documented default remains active
**And** the Hook Thread performs no parsing, allocation, logging, or blocking operation.

**Given** an arrangement policy receives monitor geometry
**When** it constructs a placement plan
**Then** rectangles use signed physical-pixel coordinates and half-open edges
**And** the contract explicitly carries monitor work area and DPI context
**And** checked arithmetic prevents overflow, negative width, and negative height.

**Given** the deterministic arrangement harness supplies a command, active window, ordered candidates, work area, and DPI
**When** planner and adapter lanes are tested
**Then** it captures placement plans and platform calls without invoking User32
**And** existing cycling behavior remains unchanged
**And** snap exclusively owns `arrangement/snap.rs`
**And** stack exclusively owns `arrangement/stack.rs`
**And** the platform adapter exclusively owns `arrangement/win32.rs`
**And** `arrangement/mod.rs`, `hook.rs`, `worker.rs`, and final composition remain reserved for convergence
**And** every arrangement lane uses its own workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence before local convergence.

### Story 4.2: DPI-Aware Snap Planning

As a desktop organizer,
I want the active window snapped precisely to the left half, right half, or usable full screen,
So that I can arrange my workspace instantly on monitors with different sizes and DPI settings.

**Acceptance Criteria:**

**Given** a valid monitor work area
**When** the Snap Left policy runs
**Then** it returns the left half of the work area
**And** the rectangle neither covers reserved work area nor leaves the monitor work area.

**Given** the same work area
**When** the Snap Right policy runs
**Then** it returns the complementary right half
**And** an odd pixel width is divided deterministically without a gap or overlap.

**Given** a valid monitor work area
**When** the Snap Maximize policy runs
**Then** it returns the complete usable work-area rectangle
**And** it does not use full monitor bounds when they include a taskbar or reserved application bar.

**Given** work-area coordinates originate from a Per-Monitor-V2-aware process
**When** fixtures at 96, 120, 144, and 192 DPI or with negative monitor coordinates are evaluated
**Then** the planner preserves supplied physical-pixel coordinates
**And** does not apply DPI scaling a second time.

**Given** work-area geometry is empty, inverted, or cannot be represented safely
**When** any snap policy is requested
**Then** it returns a deterministic planning failure without a partial placement.

**Given** Story 4.2 is verified independently after Story 4.1
**When** pure geometry and property-based boundary fixtures run
**Then** all snap policies pass without User32, stack layout, Epic 3, Settings, or final composition.

### Story 4.3: Overlapping-Stack Planning

As a user with a small monitor,
I want up to three related windows arranged in a clickable overlapping stack,
So that I can reach each window directly without using Alt+Tab.

**Acceptance Criteria:**

**Given** overlapping stack is disabled
**When** the stack command is evaluated
**Then** the planner returns a successful no-op
**And** produces no placement.

**Given** an ordered list of accepted same-application candidates
**When** stack planning runs
**Then** it preserves the accepted live order
**And** selects no more than the first three candidates
**And** zero candidates produce a successful no-op.

**Given** the default stack-width policy
**When** one, two, or three candidates are planned
**Then** each window uses 50 percent of the target work-area width
**And** one window is centered
**And** two windows are anchored left and right
**And** three windows are anchored left, center, and right
**And** every window retains a visible clickable horizontal edge.

**Given** a valid configured width percentage
**When** stack positions are calculated
**Then** horizontal anchors are distributed deterministically across the available travel range
**And** all rectangles remain inside the monitor work area
**And** zero or greater-than-100 percent produces a deterministic policy error.

**Given** work areas have odd dimensions, negative origins, or 96, 120, 144, or 192 DPI
**When** the isolated stack-policy suite runs
**Then** one-, two-, three-, and greater-than-three candidate fixtures produce deterministic results
**And** no User32, snap policy, Epic 3, Settings, or final composition is required.

**Given** Story 4.1 is accepted
**When** Story 4.3 begins in its own worktree and workflow
**Then** it consumes only the frozen arrangement contract
**And** it does not depend on another parallel-lane implementation.

### Story 4.4: Non-Blocking Win32 Arrangement Adapter

As a WinTick user,
I want layout plans applied to the correct monitor without freezing or stealing focus,
So that arrangement remains reliable even when another application is slow or closes unexpectedly.

**Acceptance Criteria:**

**Given** an accepted arrangement command and active window
**When** platform context is requested
**Then** the adapter resolves the foreground window, its nearest monitor, current work area, and window DPI for that command
**And** monitor or DPI state is not cached between commands
**And** Per-Monitor-V2 awareness is verified before placement evidence is accepted.

**Given** Win32 monitor coordinates are converted into the arrangement contract
**When** the target monitor uses non-default DPI or negative coordinates
**Then** physical-pixel coordinates are preserved without logical-pixel double scaling.

**Given** a valid placement plan
**When** the adapter applies it
**Then** each target is revalidated immediately before placement
**And** `SetWindowPos` uses non-activating, Z-order-preserving, asynchronous flags where required
**And** no overlay or transition surface replaces the current foreground window.

**Given** one target closes or becomes invalid after planning
**When** a multi-window plan is applied
**Then** that target is skipped
**And** remaining valid placements continue
**And** no runtime popup, crash, or blocking cross-process call occurs.

**Given** the adapter is inspected and tested
**When** fake-FFI contract tests and an elevated helper-window runtime test run
**Then** call arguments, work-area selection, flags, partial failure, and call ordering are verified
**And** `SendMessage`, `GetWindowText`, internal geometry caching, and Epic 3 virtual-desktop integration are absent.

**Given** Story 4.1 is accepted
**When** Story 4.4 begins in its own worktree and workflow
**Then** it consumes only the frozen arrangement contract
**And** it does not depend on Story 4.2 or Story 4.3 implementation.

### Story 4.5: Window-Arrangement Integration and Convergence

As a WinTick user,
I want arrangement shortcuts to produce correct snapping and stacking through the existing background service,
So that the complete feature works instantly without weakening cycling or hook reliability.

**Acceptance Criteria:**

**Given** Stories 4.2, 4.3, and 4.4 have accepted review targets against Story 4.1
**When** convergence begins
**Then** their frozen contract and evidence versions match
**And** no upstream lane has an unresolved Critical or Major finding
**And** convergence uses its own workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

**Given** an exact configured snapping shortcut is pressed
**When** the Hook-to-Worker pipeline handles it
**Then** the corresponding primitive command is published through the existing SPSC ring
**And** the Worker resolves fresh platform context, invokes the snap planner, and applies the plan
**And** callback time, allocation, throttle, and swallowing contracts remain intact.

**Given** stack layout is enabled and its exact shortcut is pressed
**When** the Worker handles `OverlappingStack`
**Then** it obtains live same-application candidates through the Epic 2 contract
**And** retains only candidates on the active target monitor without requiring Epic 3
**And** plans and applies no more than three placements.

**Given** stack layout is disabled
**When** its exact shortcut is accepted
**Then** no window is moved
**And** the daemon remains responsive without overlay, animation, popup, or error-state escalation.

**Given** DPI, taskbar work area, or target validity changes between commands
**When** another arrangement command runs
**Then** fresh geometry is used
**And** invalid targets are skipped without blocking or crashing
**And** actionable runtime failure follows the Tier-2 path without downgrading Critical tray state.

**Given** the Epic 4 convergence suite runs
**When** policy, fake-adapter, elevated helper-window, and end-to-end shortcut tests execute
**Then** half-left, half-right, maximize, stack counts zero through greater than three, negative coordinates, mixed DPI, disabled stack, invalid targets, and invisible feedback pass
**And** cycling, heartbeat, Explorer recovery, shutdown, RAM, CPU, binary-size, and zero-dropout gates remain satisfied
**And** neither Epic 3 nor completed Settings functionality is required.

**Given** Epic 3, Epic 4, or Epic 5 convergence diffs touch the same daemon hotspot
**When** their accepted worktrees are prepared for canonical merge
**Then** the cross-epic release/evidence gate serializes those merges and reruns combined qualification
**And** no simultaneous merge or unreviewed manual conflict resolution is permitted.

## Epic 5: Personalized & Accessible Experience

Users can configure shortcuts through a native-feeling accessible Settings application and learn the product through a first-run interactive experience without burdening the background daemon.

### Story 5.1: Shared Settings, Persistence, and IPC Contract

As a WinTick user,
I want my preferences represented and saved through one stable shared contract,
So that Settings and the background daemon cannot interpret or apply them differently.

**Acceptance Criteria:**

**Given** WinTick starts without an existing configuration file
**When** the accepted shared configuration contract creates its default state
**Then** the primary shortcut is `Win+Backtick`
**And** the fallback shortcut is `Alt+Backtick`
**And** every switcher, snapping, layout, general, and VM-bypass field has its frozen deterministic default.

**Given** a complete or partial valid `config.toml`
**When** it is parsed and serialized through the shared contract
**Then** every defined field round-trips without loss
**And** missing fields receive their documented defaults
**And** shortcut values use one canonical representation shared by Settings and the Hook actor.

**Given** a shortcut value is submitted for persistence
**When** it contains an unsupported token, no main key, or more than one main key
**Then** shared validation rejects it before active configuration is replaced
**And** the previous valid value remains available.

**Given** Settings saves a valid configuration
**When** persistence succeeds
**Then** it writes a temporary file and atomically replaces `%APPDATA%\WinTick\config.toml`
**And** failure before replacement leaves the previous file intact
**And** reload intent is emitted only after the completed file is visible.

**Given** Settings needs the daemon to reload configuration
**When** it creates the IPC intent
**Then** it uses the frozen `WM_APP_RELOAD_CONFIG` identifier with no cross-process configuration pointer
**And** configuration data travels through the completed TOML file
**And** no file watcher or polling contract is introduced.

**Given** no configuration exists at daemon startup
**When** first-run state is evaluated
**Then** the frozen launch contract selects `wintick-settings.exe --onboarding`
**And** either tutorial completion or Skip Tutorial creates a valid configuration so onboarding is not repeated unintentionally.

**Given** Story 5.1 is verified independently
**When** defaults, validation, canonical formatting, atomic persistence, IPC, and first-run contract tests run
**Then** they pass without rendered UI, a live daemon hook, Epic 3 behavior, or Epic 4 behavior.

**Given** the Settings contract lane and the independent accessibility/theme contract lane begin in parallel after Epic 2
**When** file ownership is assigned
**Then** Story 5.1 exclusively owns Settings-side contract and persistence modules plus their tests
**And** it consumes the frozen shared schema without independently reinterpreting Epic 2 fields
**And** it does not edit accessibility, theme-probe, or dependency-version files
**And** it uses a dedicated workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

### Story 5.2: Accessible Native-Theme Runtime Contract

As a Windows user who relies on native accessibility behavior,
I want WinTick controls to expose meaningful keyboard and screen-reader semantics,
So that I can use Settings without depending on a mouse or visual-only state.

**Acceptance Criteria:**

**Given** AR-11 blocks Settings implementation until its accessibility mechanism is explicit
**When** this contract gate completes
**Then** `egui` and `eframe` 0.35.x with the supported AccessKit-backed Windows adapter are selected and frozen
**And** Cargo metadata, `Cargo.lock`, and the architecture SSOT identify the same version and mechanism
**And** the existing 0.28 workspace dependency is not silently retained as a divergent runtime.

**Given** an isolated probe renders a toggle and shortcut control
**When** Windows UI Automation inspects the surface
**Then** each exposes a stable role, accessible name, current value, enabled state, and checked or listening state as applicable
**And** Listening mode is not communicated through visual text alone.

**Given** the probe has keyboard focus
**When** the user presses `Tab`, `Shift+Tab`, `Space`, `Enter`, and `Escape`
**Then** focus order is deterministic
**And** controls can be reached and operated without a mouse
**And** focus remains visible in both supported themes.

**Given** Windows uses Light or Dark mode
**When** the probe starts or the OS theme changes while it is open
**Then** the surface selects the matching native-feeling mode
**And** text, focus indicators, controls, and error states retain readable contrast
**And** Segoe UI or the documented Windows fallback typography is used.

**Given** About currently uses a native Win32 `MessageBox`
**When** its Light and Dark behavior is probed
**Then** the native surface is retained only if it satisfies the accepted theme and accessibility contract
**And** otherwise the contract requires an accessible themed About surface in the Settings process.

**Given** Settings and daemon are separate executables
**When** accessibility and theme probes run
**Then** they require no daemon UI framework, Hook Thread access, or shared mutable state
**And** opening and closing the probes cannot affect daemon RAM or callback timing.

**Given** automated accessibility-tree checks and Windows UI Automation smoke tests execute
**When** any required role, state, keyboard operation, or theme behavior cannot be demonstrated
**Then** Story 5.2 is not accepted
**And** dependent UI stories remain ineligible until corrected or an explicit architecture fallback is adopted.

**Given** Stories 5.1 and 5.2 begin as parallel contract lanes after Epic 2
**When** file ownership is assigned
**Then** Story 5.2 exclusively owns Settings dependency metadata, `Cargo.lock`, and accessibility/theme probe modules and tests
**And** it does not edit Settings persistence or shared configuration-contract modules
**And** it uses a dedicated workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

### Story 5.3: Modular Settings Shell and Safe Editing

As a WinTick user,
I want a native-feeling Settings window that organizes and saves my preferences safely,
So that I can personalize WinTick without manually editing TOML.

**Acceptance Criteria:**

**Given** Stories 5.1 and 5.2 are accepted
**When** `wintick-settings.exe` starts in normal Settings mode
**Then** it presents feature-oriented groups for Core Switcher, Window Snapping, Stack Layout, and applicable advanced settings
**And** it uses the accepted theme, typography, and accessibility contract
**And** remains decoupled from the daemon.

**Given** a valid configuration exists
**When** the Settings shell initializes
**Then** all defined values are represented in a local draft
**And** no change reaches the daemon before Save
**And** fields owned by other epics round-trip without requiring their runtime capability.

**Given** no configuration exists
**When** normal Settings mode opens
**Then** the shell displays shared defaults
**And** it creates a valid configuration through the same save path used for an existing file.

**Given** the user navigates only with the keyboard
**When** `Tab`, `Shift+Tab`, `Space`, and `Enter` are used
**Then** every interactive control and action is reachable in logical order
**And** each toggle exposes its label and checked state through the accepted accessibility adapter.

**Given** the draft passes shared validation
**When** the user activates Save
**Then** configuration is persisted atomically
**And** exactly one reload intent is emitted after successful replacement
**And** success is confirmed without restarting the daemon.

**Given** validation or persistence fails
**When** Save is attempted
**Then** the original file and active configuration remain unchanged
**And** neutral, direct, solution-oriented microcopy identifies corrective action
**And** no daemon runtime popup is requested.

**Given** Story 5.3 is verified independently
**When** model, persistence-adapter, keyboard-order, theme, and accessibility tests run with fake shortcut controls and a fake daemon-intent sink
**Then** they pass without onboarding, a live hook, or Epic 3 or Epic 4 behavior.

### Story 5.4: Accessible Physical Shortcut Capturer

As a power user,
I want to configure shortcuts by pressing the actual key combination,
So that primary and fallback shortcuts are recorded accurately without typing configuration syntax.

**Acceptance Criteria:**

**Given** Stories 5.1 and 5.2 are accepted and a shortcut control has focus
**When** the user clicks it or activates it with the keyboard
**Then** the control enters an explicit Listening state
**And** Windows UI Automation exposes the state and identity of the shortcut being edited.

**Given** the control is listening
**When** physical modifier and main-key events arrive
**Then** the component derives the combination from key events rather than text input
**And** records the exact modifier set plus one supported main key
**And** produces the shared canonical shortcut format.

**Given** only modifier keys have been pressed
**When** no main key has arrived
**Then** the component remains in Listening state
**And** does not save a modifier-only shortcut.

**Given** the user presses `Escape` while listening
**When** capture is cancelled
**Then** the previous shortcut remains unchanged
**And** focus returns to the shortcut control
**And** cancellation is announced through accessibility state.

**Given** an unsupported physical combination is entered
**When** shared validation rejects it
**Then** the previous value remains intact
**And** neutral corrective microcopy and an accessible error description are exposed.

**Given** a valid shortcut is captured
**When** capture completes
**Then** visible and UI Automation values describe the same exact combination
**And** additional modifiers are not silently discarded.

**Given** Story 5.4 is verified independently
**When** synthetic physical-key sequences, cancellation, invalid input, keyboard-only activation, and accessibility-tree assertions run
**Then** they pass without the Settings shell, live global hook, daemon process, onboarding, or other implementation lanes.

### Story 5.5: Interactive First-Run Shortcut Training

As a new WinTick user,
I want an accessible interactive exercise for the default cycling shortcut,
So that I understand WinTick before it disappears into the background.

**Acceptance Criteria:**

**Given** Stories 5.1 and 5.2 are accepted
**When** Settings launches with `--onboarding`
**Then** it presents an interactive dummy-window exercise for `Win+Backtick`
**And** installs no second global keyboard hook
**And** handles practice input only within the onboarding process.

**Given** the dummy exercise is active
**When** the user presses the expected physical shortcut
**Then** simulated foreground state advances visibly between dummy windows
**And** success is announced accessibly
**And** no real desktop window is moved, focused, or enumerated.

**Given** guidance is presented
**When** the user learns the behavior
**Then** it explains same-application cycling, same-monitor and same-virtual-desktop preservation, and the difference from `Alt+Tab`
**And** microcopy is neutral, direct, and solution-oriented.

**Given** an experienced user activates Skip Tutorial by mouse or keyboard
**When** the skip action completes
**Then** onboarding closes through the normal completion path
**And** a valid default configuration is persisted
**And** the next daemon start does not reopen onboarding solely because it was skipped.

**Given** onboarding offers Start with Windows
**When** the user accepts or declines
**Then** `general.auto_start` reflects the explicit choice in the completed configuration
**And** auto-start is never enabled silently
**And** the lane verifies the reload intent through a fake daemon sink.

**Given** the user relies on keyboard navigation, a screen reader, or either OS theme
**When** onboarding is exercised
**Then** dummy windows, completion, Skip Tutorial, and auto-start consent follow accepted theme and accessibility contracts.

**Given** Story 5.5 is verified independently
**When** tutorial-state, practice, completion, skip, consent, theme, and accessibility tests run
**Then** they pass without a live daemon bridge, Settings shell, Epic 3, or Epic 4 functionality.

### Story 5.6: Daemon Settings Bridge and First-Run Orchestration

As a WinTick user,
I want Settings changes and first-run onboarding coordinated without interrupting the background service,
So that personalization becomes active safely without a daemon restart.

**Acceptance Criteria:**

**Given** Stories 5.1 and 5.2 are accepted and configuration does not exist
**When** the daemon evaluates first-run state
**Then** it launches `wintick-settings.exe --onboarding` through `ShellExecute` at most once for that daemon startup
**And** continues using safe defaults
**And** never waits for the Settings process to exit.

**Given** Settings atomically replaces a valid configuration
**When** the daemon receives `WM_APP_RELOAD_CONFIG`
**Then** it reads and validates the completed file exactly in response to that message
**And** uses no watcher, polling loop, or repeated idle wake-up.

**Given** the candidate configuration is valid
**When** reload is accepted
**Then** each owning actor receives an owned immutable configuration snapshot through explicit control-plane message passing
**And** Hook-owned shortcut state is never mutated concurrently by the Worker
**And** the Hook-to-Worker ring continues carrying only frozen `u8` commands
**And** no cross-process configuration pointer is used.

**Given** the file is missing, unreadable, or invalid
**When** reload is attempted
**Then** every actor retains its last-known-good configuration
**And** one Tier-2 warning is logged and latched
**And** no runtime popup, crash, or partial update occurs.

**Given** a reloaded configuration requests an auto-start state
**When** the daemon applies it
**Then** it delegates to Story 1.4's accepted Task Scheduler implementation
**And** duplicate enable or disable requests converge to the requested state.

**Given** the bridge runs while the Story 2 hook contract is active
**When** reload and first-run decisions are tested
**Then** exact matching, throttle, ring capacity, Hook ownership, heartbeat recovery, and shutdown remain unchanged.

**Given** Story 5.6 is verified independently
**When** fake-filesystem, fake-launcher, fake-actor-sink, invalid-reload, auto-start, and idle-wake tests run
**Then** they pass without rendered Settings UI, onboarding implementation, Epic 3, or Epic 4 behavior.

### Story 5.7: Personalized and Accessible Experience Convergence

As a WinTick user,
I want onboarding, Settings, shortcut capture, accessibility, and live reload to work as one dependable experience,
So that I can learn and personalize WinTick without compromising background reliability.

**Acceptance Criteria:**

**Given** Stories 5.3, 5.4, 5.5, and 5.6 have accepted review targets against Stories 5.1 and 5.2
**When** convergence begins
**Then** their frozen contracts and modules are integrated without reopening accepted internal behavior
**And** Story 5.3 has exclusively owned Settings shell/editor modules
**And** Story 5.4 has exclusively owned shortcut-capture modules
**And** Story 5.5 has exclusively owned onboarding modules
**And** Story 5.6 has exclusively owned daemon settings-bridge and first-run modules
**And** each upstream story has used a distinct workflow ID and worktree and completed Codex-to-Cursor-to-Antigravity evidence
**And** shared entry points, `main.rs`, daemon Hook/Worker wiring, tray/About wiring, and end-to-end tests are owned exclusively by Story 5.7
**And** convergence uses its own workflow ID and worktree and completes Codex-to-Cursor-to-Antigravity evidence.

**Given** WinTick launches on a clean user profile
**When** no configuration exists
**Then** the daemon continues with safe defaults and launches onboarding without blocking
**And** completing or skipping onboarding creates valid configuration
**And** a subsequent daemon start does not reopen onboarding.

**Given** the user opens Settings from the accepted tray menu
**When** the primary or fallback shortcut is captured and saved
**Then** the file is atomically replaced
**And** the daemon receives exactly one explicit reload signal
**And** the exact new shortcut becomes active without daemon restart
**And** all other defined fields survive the round-trip.

**Given** a new configuration fails validation during reload
**When** the user exercises the previously active shortcut
**Then** the last-known-good shortcut still works
**And** failure appears as a Tier-2 log and tray warning without a popup.

**Given** Windows switches between Light and Dark mode
**When** Settings, About, and onboarding open or remain active
**Then** every surface follows the native-theme contract
**And** no feature introduces a cycling or arrangement overlay.

**Given** keyboard-only and screen-reader verification is performed
**When** complete Settings and onboarding flows are traversed
**Then** every action is reachable in logical `Tab` and `Shift+Tab` order
**And** toggles expose checked state
**And** shortcut controls expose value, Listening, validation, and focus state
**And** completion and Skip Tutorial are operable and announced.

**Given** Settings is repeatedly opened, saved, reloaded, and closed while shortcut input is generated
**When** elevated runtime and performance checks run
**Then** the daemon remains within RAM, idle-CPU, callback-time, ring, heartbeat, and zero-dropout contracts
**And** Settings lifetime never blocks the daemon message loop or Hook Thread.

**Given** Epic 3 and Epic 4 are not yet integrated
**When** the Epic 5 convergence suite runs
**Then** it passes using accepted Epic 1 and Epic 2 capabilities
**And** does not require VM/RDP bypass, spatial filtering, snapping, or stack runtime behavior.

**Given** Epic 3, Epic 4, or Epic 5 convergence diffs touch the same daemon entry point or actor file
**When** their accepted worktrees are prepared for canonical merge
**Then** the cross-epic release/evidence gate serializes those merges and reruns combined qualification
**And** preserves every accepted upstream review target and SSOT fingerprint.
