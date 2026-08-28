//! Cross-crate constants: file names, paths, and custom Win32 message IDs.
//! Shared cross-crate identifiers for config reload IPC and workspace types.
//! All values here are the single source of truth shared between `daemon` and
//! `settings`.

/// Application subdirectory name under `%APPDATA%`.
pub const APP_DIR_NAME: &str = "WiraDesk";

/// TOML configuration file name.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Append-only log file name.
pub const LOG_FILE_NAME: &str = "wiradesk.log";

/// Named mutex for single-instance locking (daemon).
pub const SINGLE_INSTANCE_MUTEX: &str = "Global\\WiraDeskSingleInstanceMutex";

/// Named mutex for single-instance locking (settings window).
pub const SETTINGS_SINGLE_INSTANCE_MUTEX: &str = "Global\\WiraDeskSettingsSingleInstanceMutex";

/// Target `[[bin]]` name for the `settings` crate — MUST stay aligned with
/// `crates/settings/Cargo.toml` (`default-run` `[[bin]] name`).
pub const SETTINGS_BIN_NAME: &str = "wiradesk-settings";

/// Settings executable file name (daemon builds a path relative to the install folder).
pub const SETTINGS_EXE_NAME: &str = "wiradesk-settings.exe";

/// First-run command-line flag for `wiradesk-settings.exe`.
/// Frozen in `shared` because it is used on **both sides**: the daemon emits it
/// when config is missing, and Settings consumes it. If defined separately in
/// each crate, a typo on one side only shows up as onboarding that never
/// appears — not a compile error.
pub const ONBOARDING_FLAG: &str = "--onboarding";

/// Windows Task Scheduler task name for elevated logon auto-start.
/// Used by `daemon::autostart` for `schtasks /Create|/Query|/Delete /TN`.
pub const TASK_NAME: &str = "WiraDesk";

/// Hidden window class name owned by the daemon (message-only window).
pub const DAEMON_WINDOW_CLASS: &str = "WiraDeskDaemonHiddenWindow";

/// Hidden daemon window title — used by Settings via `FindWindowW` to locate
/// the `WM_APP_RELOAD_CONFIG` target.
pub const DAEMON_WINDOW_TITLE: &str = "WiraDeskDaemon";

// ─────────────────────────────────────────────────────────────────────────
// Win32 custom window messages for daemon IPC. WM_APP = 0x8000.
// ─────────────────────────────────────────────────────────────────────────

/// Base `WM_APP` value from Win32 (`winuser.h`).
pub const WM_APP: u32 = 0x8000;

/// Message: Settings tells the daemon to reload `config.toml`.
pub const WM_APP_RELOAD_CONFIG: u32 = WM_APP + 1;

/// Internal message: Worker thread receives a ready-to-read `u8` command from the ring buffer.
pub const WM_APP_COMMAND_READY: u32 = WM_APP + 2;

/// Internal message: heartbeat monitor reports a dead hook (escalates to Tier 3).
pub const WM_APP_HOOK_DEAD: u32 = WM_APP + 3;

/// Internal message: a runtime warning was logged (triggers Tier 2 red-dot tray state).
pub const WM_APP_LOG_WARNING: u32 = WM_APP + 4;

/// Internal message: heartbeat tick from `health.rs` — asks `wndproc_impl` to
/// verify or refresh the keyboard hook on each heartbeat tick. Separate from
/// `WM_APP_HOOK_DEAD` (which means "transition to Tier 3 Critical").
pub const WM_APP_HOOK_CHECK: u32 = WM_APP + 5;

/// Hook Thread ready — `wParam` = hook thread id (`PostThreadMessageW` target).
pub const WM_APP_HOOK_READY: u32 = WM_APP + 6;

/// Hook Thread failed initialization (fatal policy).
pub const WM_APP_HOOK_INIT_FAILED: u32 = WM_APP + 7;

/// Hook Thread reports successful hook refresh (resets the consecutive fail counter).
pub const WM_APP_HOOK_REFRESH_OK: u32 = WM_APP + 8;

/// Request Hook Thread shutdown (unhook + exit message loop).
pub const WM_APP_HOOK_SHUTDOWN: u32 = WM_APP + 9;

// Debug verification seams — posted to the hidden window or Hook Thread
// during elevated runtime verification; values remain safe in release (handlers are compile-out).
/// Toggle forced hook refresh failure (Hook Thread).
pub const WM_APP_DEBUG_TOGGLE_HOOK_FAIL: u32 = WM_APP + 20;
/// Force one Tier-2 warning (`log::warn`) to verify the red-dot tray state.
pub const WM_APP_DEBUG_TRIGGER_WARN: u32 = WM_APP + 21;
/// Force one `WM_APP_HOOK_CHECK` tick to the Hook Thread (without waiting for heartbeat).
pub const WM_APP_DEBUG_HOOK_CHECK: u32 = WM_APP + 22;
/// Write QPC callback statistics to trace (`HOOK_LATENCY: ...`).
pub const WM_APP_DEBUG_DUMP_HOOK_LATENCY: u32 = WM_APP + 23;
/// Simulate a shortcut (`wParam` 0=primary, 1=extra modifier) on the Hook Thread.
pub const WM_APP_DEBUG_SIMULATE_SHORTCUT: u32 = WM_APP + 24;
/// Write cycle latency distribution and reconciliation counters to the debug trace.
/// Separate from `WM_APP_DEBUG_DUMP_HOOK_LATENCY`: end-to-end Worker distribution
/// must be reported independently of hook callback timing.
pub const WM_APP_DEBUG_DUMP_CYCLE_METRICS: u32 = WM_APP + 25;
/// Reset all cycle latency samples and reconciliation counters.
pub const WM_APP_DEBUG_RESET_CYCLE_METRICS: u32 = WM_APP + 26;
/// Run `wParam` consecutive cycles through the Worker path.
/// Deliberately does NOT use `WM_APP_DEBUG_SIMULATE_SHORTCUT`: that seam drains
/// the ring and resets throttle on every call, so at high volume it drops commands
/// before they are drained and produces false dropouts. This seam measures
/// "Worker command receipt → activation completion", so it drives exactly that path.
pub const WM_APP_DEBUG_CYCLE_BURST: u32 = WM_APP + 27;
/// Run **one** command (`wParam` = `u8` `Command` value) through the full Worker
/// path using the actual foreground window.
/// Unlike `WM_APP_DEBUG_CYCLE_BURST`, which only repeats `Cycle` for measurement,
/// this seam is used by scenario harnesses to prove the *success* path — that a
/// candidate is actually accepted, focus actually moves, and windows are actually
/// placed. Until something runs it, that entire path is only proven "does not crash".
pub const WM_APP_DEBUG_RUN_COMMAND: u32 = WM_APP + 28;
/// Toggle acceptance of `LLKHF_INJECTED`-flagged input by the hook (Hook Thread).
/// The hook permanently rejects injected input on the normal path, and that is
/// **required**: Wira Desk itself injects `VK_NONAME` to suppress the Start Menu,
/// so accepting injected input would make the hook consume its own injection.
/// As a result, the entire harness can only drive the Worker via `PostMessageW`,
/// which bypasses the hook — and because Windows grants foreground to the process
/// that received the last input event, the daemon never obtains it, so
/// `SetForegroundWindow` is always denied and every cycle ends `Exhausted`. Every
/// number ever recorded therefore measures a cycle where focus did not move.
/// This seam opens that path **only in `debug_assertions` builds** so the harness
/// can send real `SendInput` shortcuts and drive the full hook → ring → Worker →
/// activation chain. Safe against Wira Desk's own injection because `VK_NONAME`
/// does not match any shortcut.
pub const WM_APP_DEBUG_TOGGLE_ACCEPT_INJECTED: u32 = WM_APP + 29;

/// Deliver an owned Hook configuration snapshot to the Hook thread.
/// `lParam` carries `Box::into_raw` of `daemon::config::HookSnapshot`; the Hook
/// Thread reconstructs it with `Box::from_raw` so **ownership fully transfers**
/// and the old snapshot is dropped there. This satisfies AC "each owning actor
/// receives an owned immutable configuration snapshot through explicit
/// control-plane message passing": no shared state, no lock, and Hook-owned
/// shortcuts are never mutated concurrently by the Worker.
/// Not a cross-process pointer — sender and receiver are two threads in the
/// same daemon process. If `PostThreadMessageW` fails, the sender reclaims its
/// `Box` so nothing leaks.
pub const WM_APP_CONFIG_SNAPSHOT: u32 = WM_APP + 30;

/// Lease level carried in `WM_APP_CAPTURE_LEASE` / `WM_APP_HOOK_LEASE`'s `wParam`.
/// A single ordered level rather than two independent booleans, per `DEC-004`:
/// one value that is last-write-wins cannot contradict itself the way two
/// separately-armed flags could.
pub const CAPTURE_LEASE_NONE: usize = 0;
/// Settings reports what the hook observes and suppresses its own action, but
/// never swallows — the Shortcuts pane is visible and Settings holds the
/// foreground window.
pub const CAPTURE_LEASE_OBSERVE: usize = 1;
/// Everything `CAPTURE_LEASE_OBSERVE` does, plus swallowing the chord from
/// Windows — a shortcut field is actually listening and Settings holds the
/// foreground window.
pub const CAPTURE_LEASE_RECORD: usize = 2;

/// Settings process requests a temporary shortcut capture lease on the daemon,
/// or releases it.
/// `wParam` = lease level (`CAPTURE_LEASE_NONE` / `_OBSERVE` / `_RECORD`);
/// `lParam` = Settings process id (`std::process::id()`, as `isize`).
///
/// `lParam` carries a **process id**, never a window handle. The daemon's
/// comparison (`hook.rs`'s `capture_lease_pid` guard) is against a process id
/// it reads from `GetForegroundWindow`, so a process id is the value that
/// comparison actually needs — a window handle would have to be converted to
/// one on the daemon side, and `DEF-3` is exactly that conversion failing
/// silently. `std::process::id()` cannot fail the way a cross-process handle
/// conversion can.
pub const WM_APP_CAPTURE_LEASE: u32 = WM_APP + 31;

/// Internal daemon message: host window notifies Hook Thread of updated capture
/// lease settings. Same shape as `WM_APP_CAPTURE_LEASE`, forwarded unchanged —
/// `wParam` = lease level; `lParam` = Settings process id.
pub const WM_APP_HOOK_LEASE: u32 = WM_APP + 32;

/// Daemon sends a chord its hook actually observed back to Settings, while the
/// observe or record lease is armed (`DEC-004`).
/// `wParam` = Win32 Virtual Key code (`vkCode` as `u32`); `lParam` = packed
/// modifier bits (1=Ctrl, 2=Win, 4=Alt, 8=Shift). Posted to the Settings
/// process's dedicated receiver window (`SETTINGS_HOOK_WINDOW_CLASS` /
/// `_TITLE`), which the daemon locates with a plain `FindWindowW` the same way
/// Settings already locates the daemon's own hidden window — never through a
/// handle carried across the process boundary.
pub const WM_APP_RECORDED_CHORD: u32 = WM_APP + 33;

/// The daily update check has new state to show.
///
/// Carries nothing: both parameters are zero, and the result lives in
/// `updatecheck`'s own state. A version string does not fit in a `WPARAM`, and packing one
/// into a heap pointer for the receiver to free is a leak waiting for a dropped message.
pub const WM_APP_UPDATE_STATE: u32 = WM_APP + 34;

/// Window class for the Settings process's hidden receiver window, which exists
/// only to receive `WM_APP_RECORDED_CHORD` from the daemon on the observe/record
/// lease (`DEC-004`). Kept separate from the visible Slint-owned window so the
/// daemon can find it with `FindWindowW` without touching winit's window handle.
pub const SETTINGS_HOOK_WINDOW_CLASS: &str = "WiraDeskSettingsHookWindow";

/// Title of the Settings process's hidden receiver window. See
/// [`SETTINGS_HOOK_WINDOW_CLASS`].
pub const SETTINGS_HOOK_WINDOW_TITLE: &str = "WiraDeskSettingsHookReceiver";

// ─────────────────────────────────────────────────────────────────────────
// Timing and sizing constants.
// ─────────────────────────────────────────────────────────────────────────

/// Cross-thread ring buffer capacity (static 16 slots).
pub const RING_BUFFER_CAPACITY: usize = 16;

/// Anti-macro throttle threshold in milliseconds (drop input < 50ms).
pub const ANTI_MACRO_THROTTLE_MS: u64 = 50;

/// Hook validity heartbeat interval in seconds.
pub const HOOK_HEARTBEAT_SECS: u64 = 10;

/// Maximum hook install attempts during daemon startup.
pub const HOOK_RETRY_MAX: u32 = 5;

/// Delay between hook install retries in seconds.
pub const HOOK_RETRY_DELAY_SECS: u64 = 1;

/// Number of consecutive failed heartbeat ticks before escalating to Tier 3
/// Critical tray state. Unlike `HOOK_RETRY_MAX`, this counts runtime refresh
/// failures per heartbeat (10 seconds between ticks), not blocking startup retries.
pub const HOOK_CHECK_FAIL_THRESHOLD: u32 = 3;
