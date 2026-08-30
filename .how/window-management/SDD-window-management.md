---
type: sdd
component: window-management
status: reviewed
created: 2026-08-21
updated: 2026-08-21
realizes: [UC-1, UC-2, UC-3]
binds: [AD-1, AD-2, AD-3, AD-4, AD-5, AD-6, AD-7, AD-8, AD-9, AD-10, AD-12]
reviewed:
  date: '2026-08-30'
  sha: 'd14c964'
  lenses: [structure, prose, edge-case-hunter]
---

# SDD — window-management

## Decision Summary

The `window-management` component is built as an elevated background daemon container (`wiradesk.exe`) decomposed into four isolated logical components: `LC-hook-thread` (input boundary), `LC-worker-thread` (execution and cycling control), `LC-tray-controller` (shell lifecycle and health presentation), and `LC-arrangement-engine` (DPI-aware snap geometry). It executes instant, overlay-free same-application window cycling and screen arrangement with zero cloud telemetry and a static RAM footprint under 2 MB.

The two most expensive architectural decisions reversed from naive desktop utility designs are:
1. **Stateless live Z-order traversal over cached window trees (AD-3):** Maintaining an internal window tree model desynchronizes whenever users click, switch apps via Taskbar/Alt-Tab, or close background windows. Querying live OS state just-in-time via non-blocking kernel APIs on each keypress guarantees the traversal always reflects the desktop as it is, at a cost bounded by the hook budget in NFR-2 rather than by a figure asserted here.
2. **Actor/Message-Passing split between input and execution (AD-1, AD-2):** Windows enforces a strict `LowLevelHooksTimeout` (~300 ms) on `WH_KEYBOARD_LL` callbacks. Performing window enumeration, COM calls, or focus manipulation inside the hook callback risks OS hook unhooking. Decoupling the hook into a dedicated thread communicating exclusively via a lock-free 16-slot ring buffer keeps the hook callback inside the 10 ms budget NFR-2 sets, with no allocation and no blocking call on that thread.

## Structure

The four Logical Components (LCs) operate strictly within the `daemon` container (`wiradesk.exe`).

| LC | type | Responsibility |
| --- | --- | --- |
| `LC-hook-thread` | service | Installs `WH_KEYBOARD_LL`, enforces 50 ms anti-macro throttle, evaluates allocation-free VM/RDP bypass, enqueues `u8` commands to the static ring buffer, and dispatches `WM_APP_COMMAND_READY`. |
| `LC-worker-thread` | service | Drains ring-buffer commands, executes stateless `EnumWindows` traversal, filters by exe name and virtual desktop, activates target windows via `SetForegroundWindow`, suppresses lone Win key-up Start menu pops, and coordinates snap placement. |
| `LC-tray-controller` | service | Hosts the top-level hidden window message loop, owns tray icon lifecycle and context menu, handles `TaskbarCreated` shell recovery, and manages the 3-Tier error state machine (Normal / Warning / Critical) and one-shot toast notification. |
| `LC-arrangement-engine` | service | Queries monitor work areas and DPI metrics, computes half-screen snap and overlapping stack target rectangles, and applies non-blocking window geometry updates via `SetWindowPos`. |

### Dependency & Communication Direction

```text
[OS Keyboard Input] ──> LC-hook-thread
                             │
                      (ring::push u8 + WM_APP_COMMAND_READY)
                             ▼
                        LC-worker-thread ──> LC-arrangement-engine
                             │                      │
                             │                      ▼
                             │                 [Win32 SetWindowPos]
                             ▼
                  [Win32 SetForegroundWindow]

[OS Explorer / Shell] ──(TaskbarCreated)──> LC-tray-controller
[health::heartbeat]   ──(WM_APP_HOOK_CHECK)──> LC-hook-thread
LC-hook-thread        ──(WM_APP_HOOK_DEAD)──> LC-tray-controller
[settings process]    ──(WM_APP_CAPTURE_LEASE)──> LC-tray-controller
LC-tray-controller    ──(WM_APP_HOOK_LEASE)──> LC-hook-thread
LC-hook-thread        ──(observed chord: vk + modifiers)──> [settings process]
Settings (IPC)        ──(WM_APP_RELOAD_CONFIG)──> LC-tray-controller ──(WM_APP_CONFIG_SNAPSHOT)──> LC-hook-thread
```

- `LC-hook-thread` writes only `u8` byte commands into the lock-free static ring buffer (`ring.rs`) and signals the worker. It never reads worker state and never performs window enumeration.
- `LC-worker-thread` drains the ring buffer and calls `LC-arrangement-engine` synchronously for snap and stack commands.
- `LC-tray-controller` runs on the main message loop thread, exclusively owning the Win32 tray state and hidden host window.
- Health monitoring runs via a background timer thread (`health.rs`) sending periodic heartbeat ticks (`WM_APP_HOOK_CHECK`) to `LC-hook-thread`.

## Inherited Constraints

The following Architectural Decisions from `ARCHITECTURE-SPINE.md` bind the design and implementation of `window-management`:

| AD | Quoted rule | How it lands here |
| --- | --- | --- |
| **AD-1** | "Each actor (hook thread, worker thread, settings process) owns its state exclusively. Cross-actor communication uses only: lock-free ring buffer (hook→worker), Win32 Window Messages (settings→daemon), TOML file (settings→daemon config), ShellExecute (daemon→settings launch)." | Hook thread, worker thread, and tray message loop never share mutable state or mutexes. All inter-thread communication uses single-direction ring buffers or Win32 message queues. |
| **AD-2** | "The Hook Thread is solely responsible for anti-macro throttle (reject inputs <50ms apart). It translates valid keypresses into a `u8` command enum (`1`=Cycle, `2`=SnapLeft, `3`=SnapRight, `4`=SnapMaximize, `5`=OverlappingStack, `6`=SnapTop, `7`=SnapBottom, `8`=MoveToNextMonitor) before writing to the ring buffer. The Worker Thread never performs input validation — it only executes commands. Wire values are **extended, never renumbered**: a value already assigned keeps its meaning permanently, because a command sitting in the ring buffer carries only the number. Any value outside the assigned set decodes to `Nop` rather than being treated as an error." | `crates/daemon/src/hook.rs` tracks QPC timestamps and discards chords occurring within 50 ms. Only validated chords are translated to `Command` (`u8`) and pushed to the ring buffer. `[MISSING]` — values 6, 7, and 8 are not yet present in `crates/shared/src/commands.rs`; they are planned by this pass. |
| **AD-3** | "On every keypress, the Worker Thread traverses the live Z-Order via `EnumWindows`. No internal Z-Order cache is permitted. The cost of iterating through windows with non-blocking Kernel APIs is accepted." | `crates/daemon/src/cycling/source.rs` performs a fresh `EnumWindows` call on every cycle request using non-blocking APIs (`IsWindowVisible`, `GetWindowLongPtrW`). |
| **AD-4** | "Two windows belong to the "same application" if and only if their owning executable filename matches (e.g., `chrome.exe`). PID comparison is prohibited as the primary identity mechanism. Window Class Name is used only as an exclusion filter to discard ghost windows and internal utility windows (e.g., `WS_EX_TOOLWINDOW`)." | Window grouping inspects process image paths via `QueryFullProcessImageNameW` and extracts the basename. Multi-process architectures (e.g. Chrome/Electron) cycle correctly regardless of differing PIDs. |
| **AD-5** | "The Settings binary writes `config.toml` to completion atomically via temp file rename, then sends a `WM_APP_RELOAD_CONFIG` Win32 message to the Daemon's hidden window. The Daemon reloads config only upon receiving this message — never via polling or file watching." | The daemon hidden window listens for `WM_APP_RELOAD_CONFIG` (0x8001), re-reads `config.toml`, and sends an immutable `HookSnapshot` to `LC-hook-thread` via `WM_APP_CONFIG_SNAPSHOT`. |
| **AD-6** | "Before intercepting any shortcut, the Hook Thread calls `GetForegroundWindow()` and checks the window's class name / process name against the bypass list (loaded from config). If matched, `CallNextHookEx` is called immediately — the key passes through physically to the VM/RDP client with zero latency." | `crates/daemon/src/context/vm_bypass.rs` uses pre-allocated UTF-16 stack buffers on the hook thread to query the foreground window and pass through matching VM/RDP clients without allocation. |
| **AD-7** | "- **Tier 1 (Startup Fatal):** Show exactly 1x `MessageBox`, then exit. No retry.<br/>- **Tier 2 (Runtime Warning):** Write to log file silently. Update tray icon to "unread log" state (red dot overlay).<br/>- **Tier 3 (Runtime Critical — hook dead, fatal but process alive):** Update tray icon to "stopped" state (red X overlay) + fire exactly 1x Windows Toast Notification. Toast is reserved exclusively for this tier." | `crates/daemon/src/tray.rs` and `error.rs` implement the 3-Tier visual protocol. Red dot overlay signals warnings; Red X overlay and single toast signal dead hook. |
| **AD-8** | "The Daemon checks hook handle validity every 10 seconds. If invalid, it attempts re-registration. If re-registration fails repeatedly (`HOOK_CHECK_FAIL_THRESHOLD = 3`), Tier 3 error protocol is triggered." | `crates/daemon/src/health.rs` ticks every 10 s (`HOOK_HEARTBEAT_SECS`). `LC-hook-thread` tracks consecutive refresh failures and posts `WM_APP_HOOK_DEAD` upon reaching threshold 3. |
| **AD-9** | "During Z-Order traversal, each candidate window must pass `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop(hwnd)`. Windows not on the current virtual desktop are skipped. This is an official, documented Microsoft API (`shobjidl_core.h`, Windows 10+)." | `crates/daemon/src/context/virtual_desktop.rs` encapsulates COM apartment initialization and vtable calls on the Worker actor. Any failure fails closed (skips candidate). |
| **AD-14** | "On every command that needs to know what monitors exist, the Worker Thread enumerates them live via `EnumDisplayMonitors` and reads each one's work area and DPI at that moment. No monitor list, work area, or `HMONITOR` may be cached, memoized, or held in a `static` between keypresses. Display-change notifications are not subscribed to, because nothing is stored for them to invalidate." | `crates/daemon/src/context/spatial.rs` will resolve the monitor set per invocation and hand it to `LC-arrangement-engine`, which holds nothing between calls. `[PARTIAL]` — the no-cache property already holds trivially, because `spatial.rs` performs only `MonitorFromWindow` and stores nothing; the enumeration this rule governs is `[MISSING]`. |
| **AD-10** | "The Daemon's message loop listens for the `TaskbarCreated` broadcast message and re-registers the tray icon upon receiving it." | `crates/daemon/src/tray.rs` registers `TaskbarCreated` via `RegisterWindowMessageW` and recreates `NOTIFYICONDATAW` when Explorer crashes and restarts. |

## Failure Behaviour

Failure modes across all internal and external Win32 / IPC boundaries:

| Boundary | Slow | Absent | Lying | What the user sees | What is logged |
| --- | --- | --- | --- | --- | --- |
| **Win32 Hook Subsystem** (`SetWindowsHookExW`, `UnhookWindowsHookEx`) | Hook callback takes >300 ms; OS unhooks callback silently. | Initial hook registration fails (`NULL` handle returned after 5 retries). | Returns non-null handle but OS stops delivering key events. | **Startup:** Tier-1 modal `MessageBox` once, process exits.<br/>**Runtime:** Tier-3 Red X tray icon + exactly 1x toast notification; shortcuts stop intercepting. | `error!` on startup failure; `warn!` on heartbeat refresh attempt; `error!` when escalating to Tier 3. |
| **Explorer.exe Tray & Shell** (`Shell_NotifyIconW`, `TaskbarCreated`) | Taskbar takes several seconds to redraw after explorer restart. | Explorer process crashes or terminates; taskbar window missing. | `Shell_NotifyIconW` returns `FALSE` despite valid handle. | Tray icon disappears while Explorer is down; automatically restores once Explorer restarts and broadcasts `TaskbarCreated`. | `info!` on initial tray creation and on successful re-registration after `TaskbarCreated`. |
| **Window Enumeration & Focus** (`EnumWindows`, `SetForegroundWindow`) | Hung window process takes long to respond to shell messages. | Target window closes between enumeration and activation. | Window has `WS_VISIBLE` but zero dimensions or transparent alpha (ghost window). | Focus does not shift to closed/ghost window; cycling gracefully skips to the next valid window in Z-order. | `debug!` recording skip reason (ineligible class, closed hwnd, or activation refusal). |
| **In-Process Static Ring Buffer** (`ring::push` / `ring::pop`) | Worker thread delayed by OS scheduling. | Ring buffer uninitialized (impossible by static layout). | Buffer full due to extreme key spam (>16 unread commands). | Keypress dropped silently with zero freeze or hang. System remains fully responsive. | `trace!` drop count incremented; debug metrics record drained and dropped counts. |
| **Virtual Desktop Manager COM** (`IVirtualDesktopManager`) | COM apartment query delays worker execution. | COM subsystem disabled, uninitialized, or returns `REGDB_E_CLASSNOTREG`. | Returns `S_OK` with stale membership for animating desktop transition. | Candidate window is treated as ineligible (fails closed); cycling remains strictly within proven current desktop. | `debug!` note on COM initialization failure; candidate marked `VirtualDesktopUnavailable`. |
| **Foreground Process Identity** (`OpenProcess`, `QueryFullProcessImageNameW`) | Querying protected system process takes >5 ms. | Process terminates before handle query or access denied. | Returns empty executable image name string. | Fails open: VM/RDP bypass treats unresolved window as non-bypassable or passes through safely without crash. | Atomic failure counter incremented; `debug!` diagnostic logged at worker boundary. |
| **Config Reload IPC** (`WM_APP_RELOAD_CONFIG`) | Settings process slow to write TOML file. | `config.toml` missing on disk during reload signal. | TOML file contains malformed syntax or invalid key names. | Daemon falls back safely to default in-memory configuration; cycling continues uninterrupted. | `warn!` logging parse failure and fallback to defaults; tray icon shows Tier-2 Red Dot. |
| **Monitor Enumeration** (`EnumDisplayMonitors`, `GetMonitorInfoW`, `GetDpiForMonitor`) | Enumeration callback runs long on a machine with many displays; it is off the hook thread, so the input path is unaffected. | No monitor is reported, or the foreground window resolves to no monitor (`MONITOR_DEFAULTTONULL`). | A monitor is reported and then unplugged before placement, so its work area describes a display that is gone. | **Absent:** nothing moves; the chord was consumed. **Lying:** Windows refuses or clamps the `SetWindowPos`, so the window lands on an attached monitor rather than off-screen, and the next press moves it again. Never a popup. | Tier-2 `warn!` naming the failing query. A single attached monitor is **not** logged — it is a successful no-op, not a failure. |
| **Duplicate Chord Configuration** (`load_shortcuts`, `config::validate`) | No slow path: a comparison over nine values at load. | Not applicable — a chord is present or it is not. | Two fields name the same chord, so the configuration claims two actions are reachable by one keypress. | **At startup:** the later action is unbound and unreachable; every other setting is honoured. **On reload:** the whole candidate is refused and the previous configuration stays in force. Tray icon goes to its Warning state either way; no popup. | Exactly one Tier-2 `warn!` naming **both** fields and the chord. Never one warning per field, and never silence — silence is the failure `DEC-009` exists to stop. |
| **Capture Lease IPC** (`WM_APP_CAPTURE_LEASE` → `WM_APP_HOOK_LEASE`) | No slow path: one integer comparison, forwarded off the callback thread. | Settings dies without disarming; the lease names a process id that no longer exists. | The lease names a process id Windows has since recycled onto an unrelated process. | Nothing: the lease is inert unless the named process also holds the foreground window, and a dead holder is reaped on the existing heartbeat. Under recycling the keyboard could reach Wira Desk while an unrelated process holds a lease — the residual risk `OQ-17` carries. | `debug!` trace recording the lease level and the process id as **received**, alongside what was sent, so a derived-value failure cannot be read as a silent sender (`DEF-3`). |

**One failure mode both IPC rows above missed, found on hardware 2026-08-28.** Each row asks what
happens when the message is slow, absent, or lying, and each answers as though the only reason a
post fails is that the daemon is gone. There is a second reason: **UIPI discards a `PostMessageW`
sent from a process below this daemon's integrity level**, and the sender cannot tell the two apart
— `PostMessageW` returns 0 either way.

It is reachable in normal use. `wiradesk-settings.exe` inherits the daemon's elevated token when the
tray launches it, and runs at medium integrity when a user starts it from Explorer. The same binary,
two integrity levels. In the second case the capture lease never arms — so shortcut recording misses
every key that only the daemon's hook sees — and a saved configuration never reaches the running
daemon.

The daemon admits both messages with `ChangeWindowMessageFilterEx`, the same mechanism already
applied to `TaskbarCreated` for this same boundary. See `contract-reload-config.md` for why widening
the filter for these two grants no capability that was not already available.

## Robustness Analysis (ABCE)

The Robustness Analysis classifies the technical design for all realized use cases (`UC-1`, `UC-2`, `UC-3`, `UC-7`) and edge-case scenarios into Boundary, Control, Entity, and Behaviour.

### 1. Boundary Objects

- **`B-KbdHook` (OS Keyboard Hook Stream):** Raw `WH_KEYBOARD_LL` input stream delivered by Windows via the low-level hook callback on `LC-hook-thread`.
- **`B-WinMgr` (Win32 Window Manager Interface):** Win32 C-API surface (`EnumWindows`, `IsWindowVisible`, `GetWindowLongPtrW`, `SetForegroundWindow`, `SetWindowPos`, `ShowWindowAsync`).
- **`B-VirtualDesktop` (COM Shell Interface):** Minimal COM vtable wrapper for `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop`.
- **`B-IdentityQuery` (Win32 Process/Class Query):** Allocation-free UTF-16 query adapter (`GetForegroundWindow`, `GetClassNameW`, `GetWindowThreadProcessId`, `OpenProcess`, `QueryFullProcessImageNameW`).
- **`B-DisplaySet` (Win32 Display Configuration):** The attached monitor set and each monitor's work area and DPI, read through `EnumDisplayMonitors`, `GetMonitorInfoW`, and `GetDpiForMonitor`. Read-only, enumerated per invocation, never cached (AD-14). `[MISSING]` — planned by this pass.
- **`B-TrayIcon` (Windows Shell Notification Area):** Shell notification area interface (`Shell_NotifyIconW`, `TrackPopupMenuEx`, Windows Toast API).
- **`B-HiddenWindow` (Win32 IPC Endpoint):** Top-level message-only hidden window (`WiraDeskDaemonHiddenWindow`) receiving `WM_APP_RELOAD_CONFIG`, `TaskbarCreated`, and internal health messages.

### 2. Control Objects

- **`C-HookController` (`LC-hook-thread`):** Owns hook lifecycle, QPC anti-macro throttle check (50 ms), bypass classification, raw byte serialization to the ring buffer, and the capture lease. The lease is three independent decisions rather than one switch — report the chord to Settings, withhold Wira Desk's own action, withhold the keystroke from Windows — and two named combinations of them: **observe** (`yes / yes / no`) while the Shortcuts pane is visible, **record** (`yes / yes / yes`) while a field is listening. Both require the settings process to hold the foreground window, both fire only on a non-modifier key-down carrying at least one modifier, and the comparison sits above `match_shortcut` so it is reached by a chord that is not yet configured. Withholding the keystroke exists only to record; no chord is ever claimed for a Wira Desk action on the strength of a lease (`DEC-004`, LBR-ST-11).
- **`C-WorkerDispatcher` (`LC-worker-thread`):** Dispatches ring-buffer command opcodes on thread wake-up, orchestrates candidate collection, filters candidates, and applies activation or geometry updates.
- **`C-CyclingSelector` (`daemon::cycling`):** Evaluates same-app candidate eligibility (exe name match, style filters), spatial alignment (monitor matching, virtual desktop isolation), and selects the next Z-order target.
- **`C-ArrangementPlanner` (`LC-arrangement-engine`):** Resolves monitor work-area bounds and DPI scale factors to plan coordinate rectangles for half-screen snaps on either axis (left, right, top, bottom), full maximize, overlapping cascade stacks, and a move to the next monitor. The monitor-move plan is the only one that reads two work areas: it expresses the window's rectangle as a share of the source and maps that share onto the destination, so an arrangement survives a difference in size or scaling (LBR-WM-7). It holds no state between invocations, including no monitor list (AD-14). `Win32WindowMover::apply` clamps the frame-inset-compensated rect against the monitor containing the *planned* (destination) rect, not the monitor the window is still on when `apply` runs — resolving from the window would aim the clamp at the monitor being left, and a compensated rect touching a different-DPI monitor is what Windows uses as its own cue to relocate and rescale the window (`DEC-010`).
- **`C-TrayHealthStateMachine` (`LC-tray-controller`):** Manages the 3-Tier error state transitions, updates icon overlays, latches warning states, and throttles toast notifications to exactly one dispatch per critical failure.

### 3. Entity Objects

- **`E-HookCommand` (`hook-command`):** Ephemeral command transfer entity represented as a `u8` byte (`0`=Nop, `1`=Cycle, `2`=SnapLeft, `3`=SnapRight, `4`=SnapMaximize, `5`=OverlappingStack, `6`=SnapTop, `7`=SnapBottom, `8`=MoveToNextMonitor). Values 6-8 are `[MISSING]` — planned by this pass. Any byte outside the assigned set decodes to `Nop`.
- **`E-ActiveContext` (`window-focus-state`):** In-memory capture of the origin window state prior to cycling (`HWND`, executable basename, `HMONITOR`, virtual desktop membership).
- **`E-TrayHealthState` (`tray-health-state`):** Current status of the tray icon (`Normal`, `Warning`, `Critical`), `warning_latched` boolean, and `hook_dead_toast_sent` single-shot guard.
- **`E-PlacementPlan` (`arrangement-command`):** Calculated target window bounds (`RECT`), DPI scale, target `HWND`, and placement flags for `SetWindowPos`. An **empty** plan is a successful no-op rather than a failure — a disabled stack and a single-monitor move both land there.
- **`E-MonitorSet`:** The live list of attached monitors with each one's work area and DPI, valid only for the duration of the command that enumerated it. It is deliberately not an entity that persists: an `HMONITOR` is a handle, not an identity (AD-14). `[MISSING]` — planned by this pass.

### 4. Behaviour & Collaborations

#### UC-1: Cycle to Next Same-App Window
1. User presses `Win + \``.
2. `B-KbdHook` invokes `C-HookController`.
3. `C-HookController` checks anti-macro throttle (<50 ms) and calls `B-IdentityQuery`.
4. If foreground window is not bypassed, `C-HookController` enqueues `Command::Cycle` (`1`) to `ring.rs`, posts `WM_APP_COMMAND_READY` to `B-HiddenWindow`, and returns `1` (swallowing keystroke).
5. `C-WorkerDispatcher` wakes up and queries `B-WinMgr` via `EnumWindows`.
6. `C-CyclingSelector` filters candidates by matching executable basename (AD-4), excludes toolwindows and ghost windows, checks `B-VirtualDesktop` membership (AD-9), and verifies monitor bounds.
7. `C-CyclingSelector` identifies the next window in Z-order.
8. `C-WorkerDispatcher` activates the target window via `B-WinMgr` (`SetForegroundWindow`).
9. `C-WorkerDispatcher` invokes `suppress_start_menu()`, injecting unassigned `VK_NONAME` to prevent the Windows Start Menu from popping upon releasing the `Win` key.

#### UC-2: Snap Active Window to Screen Region
1. User presses `Ctrl + Alt + Left` (or `Right` / `Up` / `Down` / `Enter`).
2. `C-HookController` translates the chord to `Command::SnapLeft` (`2`), `SnapRight` (`3`), `SnapMaximize` (`4`), `SnapTop` (`6`), or `SnapBottom` (`7`), pushes to `ring.rs`, and wakes `C-WorkerDispatcher`.
3. `C-WorkerDispatcher` captures the active foreground `HWND` and current monitor work area via `B-WinMgr`.
4. `C-ArrangementPlanner` computes the DPI-scaled target coordinates (`E-PlacementPlan`).
5. `C-WorkerDispatcher` applies geometry via `B-WinMgr` (`SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`).
6. `C-WorkerDispatcher` invokes `suppress_start_menu()`.

`SnapTop` and `SnapBottom` divide the work area horizontally at one boundary computed the same way `SnapLeft`/`SnapRight` divide it vertically, so all four halves inherit one tiling guarantee rather than two implementations of it (LBR-WM-8).

#### UC-7: Move Active Window to the Next Monitor
1. User presses `Ctrl + Alt + Shift + Enter`.
2. `C-HookController` translates the chord to `Command::MoveToNextMonitor` (`8`), pushes to `ring.rs`, and wakes `C-WorkerDispatcher`.
3. `C-WorkerDispatcher` captures the active foreground `HWND`, and resolves its monitor via `B-WinMgr`.
4. `C-WorkerDispatcher` enumerates `B-DisplaySet` and hands `C-ArrangementPlanner` the source monitor, the destination monitor, and the window's current rectangle.
5. `C-ArrangementPlanner` returns an **empty** `E-PlacementPlan` when only one monitor is attached — the command ends here as a successful no-op, with nothing logged.
6. Otherwise `C-ArrangementPlanner` expresses the rectangle as a share of the source work area, maps it onto the destination work area, and returns `E-PlacementPlan`.
7. `C-WorkerDispatcher` applies geometry via `B-WinMgr` (`SetWindowPos` with `SWP_NOACTIVATE | SWP_NOZORDER`), which does not change virtual desktop membership.
8. `C-WorkerDispatcher` invokes `suppress_start_menu()`.

Step 4 is where this use case differs from every other arrangement command: it is the only one whose destination work area is not the one the foreground window sits on, and the only one that reads the display set at all.

#### SCN-03: Duplicate Chord in Configuration
1. `load_shortcuts` parses every configured chord at daemon start and finds two fields resolving to the same one.
2. The field earlier in the fixed precedence order keeps the chord; the later one is left unbound, so `C-HookController` never matches it and no substitute action fires.
3. Exactly one Tier-2 `warn!` names both fields and the chord; `C-TrayHealthStateMachine` latches the Warning state.
4. On the reload path instead, `config::validate` returns `RejectReason::DuplicateShortcut`, no actor receives a snapshot, and every actor stays on its last-known-good configuration (BR-6, DEC-009).

#### UC-3: Tray Recovery After Explorer Restart
1. Windows Explorer process restarts following a crash.
2. Windows Shell broadcasts the registered message string `"TaskbarCreated"` to all top-level windows.
3. `B-HiddenWindow` receives `TaskbarCreated` and routes to `C-TrayHealthStateMachine`.
4. `C-TrayHealthStateMachine` recreates `NOTIFYICONDATAW` with the active icon handle (`Normal`, `Warning`, or `Critical`) and calls `B-TrayIcon` (`Shell_NotifyIconW(NIM_ADD)`).
5. Tray icon is immediately restored to the taskbar notification area.

#### SCN-01: VM / Remote Desktop Passthrough
1. Foreground window is a virtual machine or remote desktop client (e.g. `mstsc.exe` or `VMwareUnityWindow`).
2. User presses `Win + \`` inside the VM session.
3. `C-HookController` calls `B-IdentityQuery` using pre-allocated UTF-16 stack buffers.
4. `B-IdentityQuery` matches the process or window class name against the active `BypassPolicy`.
5. `C-HookController` calls `CallNextHookEx` immediately without enqueuing any command.
6. The key chord passes through physically to the guest operating system with zero latency.

#### SCN-02: Hook Heartbeat Failure & Tier-3 Escalation
1. `health::heartbeat` thread ticks every 10 seconds, posting `WM_APP_HOOK_CHECK` to `C-HookController`.
2. `C-HookController` checks hook validity. If the OS unhooked the callback, it attempts re-registration via `SetWindowsHookExW`.
3. If re-registration fails for `HOOK_CHECK_FAIL_THRESHOLD = 3` consecutive checks (30 s), `C-HookController` posts `WM_APP_HOOK_DEAD` to `B-HiddenWindow`.
4. `C-TrayHealthStateMachine` transitions `E-TrayHealthState` to `Critical`.
5. `C-TrayHealthStateMachine` updates `B-TrayIcon` to show the Red X overlay icon.
6. If `hook_dead_toast_sent` is `false`, `C-TrayHealthStateMachine` triggers exactly 1x Windows Toast Notification and sets `hook_dead_toast_sent = true`.

## Evidence

Every architectural claim and boundary behaviour is verified against source code in the repository:

| Claim | Label | Read to decide | Disposition |
| --- | --- | --- | --- |
| Lock-free 16-slot ring buffer command transfer | Verified | `crates/daemon/src/ring.rs`, `crates/shared/src/commands.rs` | Verified: Static 16-slot atomic ring buffer carrying `u8` commands without heap allocation. |
| 50 ms anti-macro throttle on Hook Thread | Verified | `crates/daemon/src/hook.rs` (`ANTI_MACRO_THROTTLE_MS`) | Verified: `ANTI_MACRO_THROTTLE_MS = 50` enforced via QPC timestamps in `hook_callback`. |
| Live stateless `EnumWindows` Z-order traversal | Verified | `crates/daemon/src/worker.rs`, `crates/daemon/src/cycling/source.rs` | Verified: Stateless traversal on every cycle; no internal Z-order caching. |
| Same-application grouping by executable basename | Verified | `crates/daemon/src/cycling/eligibility.rs` | Verified: Compares normalized process image basename from `QueryFullProcessImageNameW`; ignores PID. |
| Allocation-free VM/RDP bypass evaluation | Verified | `crates/daemon/src/context/vm_bypass.rs` | Verified: Uses reusable `[u16; 256]` and `[u16; 260]` stack buffers with zero heap allocation in hook path. |
| `IVirtualDesktopManager` COM isolation on worker | Verified | `crates/daemon/src/context/virtual_desktop.rs` | Verified: Custom vtable layout with `!Send`/`!Sync` adapter bound exclusively to Worker thread. |
| 10 s hook health heartbeat and retry threshold | Verified | `crates/daemon/src/health.rs`, `crates/daemon/src/hook.rs` | Verified: `HOOK_HEARTBEAT_SECS = 10`, `HOOK_CHECK_FAIL_THRESHOLD = 3`, `HOOK_RETRY_MAX = 5`. |
| Explorer crash recovery via `TaskbarCreated` | Verified | `crates/daemon/src/tray.rs` | Verified: Handles `RegisterWindowMessageW("TaskbarCreated")` and invokes `Shell_NotifyIconW(NIM_ADD)`. |
| 3-Tier error protocol & single-shot toast guard | Verified | `crates/daemon/src/tray.rs`, `crates/daemon/src/error.rs` | Verified: Tier 1 `MessageBoxW`, Tier 2 Red Dot tray overlay, Tier 3 Red X + `hook_dead_toast_sent` single-shot toast. |
| Start Menu lone Win key-up suppression | Verified | `crates/daemon/src/worker.rs` (`suppress_start_menu`) | Verified: Injects unassigned `VK_NONAME` on worker thread while Win key is held. |
| UIPI elevation check & DLL search path sterilization | Verified | `crates/daemon/src/main.rs` | Verified: Checks `TokenElevation` at startup and sets `SetDllDirectoryW(L"")`. |

## Open Items

None. All technical mechanisms, invariants (AD-1..10, AD-12), and Win32 failure boundaries are verified against the codebase and ratified.
