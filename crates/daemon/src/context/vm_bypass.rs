//! VM and Remote Desktop passthrough adapter.
//!
//! Runs on the **Hook Thread**, inside the shortcut path. The callback must not
//! allocate, log, parse config, acquire locks, sleep, or call into the Worker:
//!
//! - Buffers are fixed-size and owned by [`HookIdentityCollector`], reused on
//!   every event. Nothing is heap-allocated per keystroke.
//! - Comparison happens directly between the pre-normalized policy `&str`
//!   entries and the raw UTF-16 buffer, so no `String` is ever built.
//! - Diagnostics are a lock-free counter the Worker drains later, never a log
//!   call from the callback.
//!
//! Reinjection is never used: the adapter only *decides*. `SendInput` and any
//! other synthetic input mechanism are deliberately absent from this file.

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HWND};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId,
};

use std::sync::atomic::{AtomicU64, Ordering};

use super::{BypassDecision, BypassPolicy, BypassReason};

/// Fixed capacity for a window class name (`WNDCLASS` documented maximum).
const CLASS_CAPACITY: usize = 256;

/// Fixed capacity for a process image path. `MAX_PATH` is enough for the
/// `PROCESS_NAME_WIN32` format; a longer path degrades to "unknown", which
/// fails open exactly like any other identity failure.
const PATH_CAPACITY: usize = 260;

/// Deferred diagnostic signal.
/// The callback may not log, so it bumps this counter instead and the Worker
/// reports it later at a safe boundary.
static IDENTITY_FAILURES: AtomicU64 = AtomicU64::new(0);

pub fn identity_failure_count() -> u64 {
    IDENTITY_FAILURES.load(Ordering::Relaxed)
}

pub fn reset_identity_failures() {
    IDENTITY_FAILURES.store(0, Ordering::Relaxed);
}

/// Borrowed, allocation-free view of the foreground identity.
/// Slices point into the collector's reusable buffers, so this cannot outlive
/// the next collection — which is exactly the intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WideIdentity<'a> {
    pub process: Option<&'a [u16]>,
    pub class: Option<&'a [u16]>,
}

/// Hook Thread-owned identity collector with reusable fixed buffers.
pub struct HookIdentityCollector {
    class: [u16; CLASS_CAPACITY],
    class_len: usize,
    path: [u16; PATH_CAPACITY],
    process_start: usize,
    process_len: usize,
}

impl Default for HookIdentityCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl HookIdentityCollector {
    pub fn new() -> Self {
        HookIdentityCollector {
            class: [0; CLASS_CAPACITY],
            class_len: 0,
            path: [0; PATH_CAPACITY],
            process_start: 0,
            process_len: 0,
        }
    }

    /// Collect the current foreground identity into the reusable buffers.
    /// Every Win32 call here is a bounded non-blocking metadata query. Failure
    /// at any step leaves that half of the identity as `None` and bumps the
    /// deferred diagnostic counter.
    pub fn collect(&mut self) -> WideIdentity<'_> {
        self.class_len = 0;
        self.process_len = 0;
        self.process_start = 0;

        // SAFETY: `GetForegroundWindow` takes no arguments and touches no memory we own. A
        // zero return means no window holds the foreground, which is checked below rather
        // than passed on as a handle. Callable from the hook callback: it reads window-manager
        // state without messaging any other thread, so a hung foreground window cannot stall
        // it — the property the whole module depends on.
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd == 0 {
            IDENTITY_FAILURES.fetch_add(1, Ordering::Relaxed);
            return WideIdentity::default();
        }

        self.fill_class(hwnd);
        self.fill_process(hwnd);

        if self.class_len == 0 || self.process_len == 0 {
            IDENTITY_FAILURES.fetch_add(1, Ordering::Relaxed);
        }

        WideIdentity {
            process: (self.process_len > 0)
                .then(|| &self.path[self.process_start..self.process_start + self.process_len]),
            class: (self.class_len > 0).then(|| &self.class[..self.class_len]),
        }
    }

    /// Read the foreground window's process ID directly, without reading process path or class.
    pub fn foreground_pid(&self) -> u32 {
        // SAFETY: `GetForegroundWindow` takes no arguments and touches no memory we own.
        // It reads window-manager state without messaging any other thread.
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd == 0 {
            return 0;
        }
        let mut pid: u32 = 0;
        // SAFETY: `&mut pid` is a live out-parameter of valid u32 size. An unusable hwnd
        // returns 0 and leaves pid at 0.
        let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if thread == 0 {
            0
        } else {
            pid
        }
    }

    fn fill_class(&mut self, hwnd: HWND) {
        // SAFETY: the third argument is the buffer's true capacity in UTF-16 units — `self.class`
        // is declared `[u16; CLASS_CAPACITY]`, so the two cannot drift — and that count is the
        // only thing bounding the write. `GetClassNameW` writes at most that many units
        // including the terminator and returns the length excluding it, so the returned `len`
        // can never exceed the array. `hwnd` needs no validity proof: an unusable handle yields
        // a non-positive length and no write. The class name is read from window-manager state,
        // never by messaging the owning thread, so this stays callback-safe.
        let len = unsafe { GetClassNameW(hwnd, self.class.as_mut_ptr(), CLASS_CAPACITY as i32) };
        if len > 0 {
            self.class_len = (len as usize).min(CLASS_CAPACITY);
        }
    }

    fn fill_process(&mut self, hwnd: HWND) {
        let mut pid: u32 = 0;
        // SAFETY: `&mut pid` is a live out-param of the width the API writes. An unusable
        // `hwnd` returns thread id 0 and leaves `pid` at 0, both of which are checked below.
        let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if thread == 0 || pid == 0 {
            return;
        }

        // SAFETY: takes no pointers; `pid` is a plain integer, and a stale or privileged one
        // makes the call return 0 rather than fault. A non-zero return is an owned handle that
        // must be closed exactly once, which the `CloseHandle` below does on every path out of
        // this function. `PROCESS_QUERY_LIMITED_INFORMATION` is the narrowest right that
        // supports the query, so an elevated daemon asks for no more than it needs.
        let process: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) };
        if process == 0 {
            return;
        }

        let mut size = PATH_CAPACITY as u32;
        // SAFETY: `size` is an in/out parameter, and its *input* value is what bounds the
        // write — it is initialised to `PATH_CAPACITY`, the declared length of `self.path`, on
        // the line above. Claiming a larger capacity than the array holds is the one way this
        // call overflows, so the two must be read together. `process` is a live handle
        // confirmed non-zero above. On return `size` is the length actually written, clamped
        // again below before it indexes the buffer.
        let ok = unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                self.path.as_mut_ptr(),
                &mut size,
            )
        };
        // SAFETY: `process` is non-zero, came from `OpenProcess`, and is closed exactly once —
        // this sits *before* the `ok == FALSE` early return on purpose, so no failure path can
        // leave the handle open. In a daemon that runs for the machine's whole uptime and
        // reaches this on every matched chord, a leak here would be unbounded.
        unsafe {
            CloseHandle(process);
        }

        if ok == FALSE || size == 0 {
            return;
        }

        let total = (size as usize).min(PATH_CAPACITY);
        let (start, len) = basename_range(&self.path[..total]);
        self.process_start = start;
        self.process_len = len;
    }
}

/// Locate the basename inside a UTF-16 path, without allocating.
fn basename_range(path: &[u16]) -> (usize, usize) {
    const BACKSLASH: u16 = b'\\' as u16;
    const SLASH: u16 = b'/' as u16;
    let start = path
        .iter()
        .rposition(|c| *c == BACKSLASH || *c == SLASH)
        .map(|i| i + 1)
        .unwrap_or(0);
    (start, path.len() - start)
}

/// Case-insensitive ASCII comparison between a normalized policy entry and a
/// raw UTF-16 slice. Allocation-free in both directions.
/// `entry` is already lowercase (guaranteed by `BypassPolicy::from_config`), so
/// only the wide side needs folding.
fn eq_ignore_ascii_case_wide(entry: &str, wide: &[u16]) -> bool {
    if entry.chars().count() != wide.len() {
        return false;
    }
    entry.chars().zip(wide.iter()).all(|(e, w)| {
        let w = *w;
        // Non-ASCII wide units can never equal a lowercase ASCII entry char.
        if w > 0x7F {
            return false;
        }
        (w as u8).eq_ignore_ascii_case(&(e as u8))
    })
}

fn any_matches(entries: &[String], wide: &[u16]) -> bool {
    entries.iter().any(|e| eq_ignore_ascii_case_wide(e, wide))
}

/// Classify a borrowed identity against the frozen policy.
/// Mirrors [`BypassPolicy::classify`] exactly, including the fail-open rule —
/// but over UTF-16 slices so the callback allocates nothing.
pub fn classify_wide(policy: &BypassPolicy, identity: &WideIdentity<'_>) -> BypassDecision {
    if let Some(process) = identity.process {
        if any_matches(policy.processes(), process) {
            return BypassDecision::Passthrough(BypassReason::ProcessMatch);
        }
    }
    if let Some(class) = identity.class {
        if any_matches(policy.classes(), class) {
            return BypassDecision::Passthrough(BypassReason::ClassMatch);
        }
    }
    if identity.process.is_none() || identity.class.is_none() {
        return BypassDecision::Passthrough(BypassReason::IdentityUnavailable);
    }
    BypassDecision::ContinueWiraDeskMatching
}

/// Convenience for the Hook Thread: collect and classify in one bounded step.
pub fn evaluate_foreground(
    policy: &BypassPolicy,
    collector: &mut HookIdentityCollector,
) -> BypassDecision {
    let identity = collector.collect();
    classify_wide(policy, &identity)
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;
    use shared::config::VmBypassConfig;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    // --- basename extraction, allocation-free ------------------------------

    #[test]
    fn basename_of_backslash_path() {
        let p = wide(r"C:\Windows\System32\mstsc.exe");
        let (start, len) = basename_range(&p);
        assert_eq!(
            String::from_utf16_lossy(&p[start..start + len]),
            "mstsc.exe"
        );
    }

    #[test]
    fn basename_of_forward_slash_path() {
        let p = wide("C:/Program Files/VMware/vmware.exe");
        let (start, len) = basename_range(&p);
        assert_eq!(
            String::from_utf16_lossy(&p[start..start + len]),
            "vmware.exe"
        );
    }

    #[test]
    fn basename_of_bare_name_is_the_whole_slice() {
        let p = wide("mstsc.exe");
        assert_eq!(basename_range(&p), (0, 9));
    }

    #[test]
    fn basename_of_trailing_separator_is_empty() {
        let p = wide(r"C:\Windows\");
        let (start, len) = basename_range(&p);
        assert_eq!(len, 0);
        assert_eq!(start, p.len());
    }

    // --- wide comparison ----------------------------------------------------

    #[test]
    fn wide_comparison_is_case_insensitive() {
        assert!(eq_ignore_ascii_case_wide("mstsc.exe", &wide("MSTSC.EXE")));
        assert!(eq_ignore_ascii_case_wide("mstsc.exe", &wide("mstsc.exe")));
        assert!(eq_ignore_ascii_case_wide("mstsc.exe", &wide("MsTsC.ExE")));
    }

    #[test]
    fn wide_comparison_rejects_different_lengths() {
        assert!(!eq_ignore_ascii_case_wide("mstsc.exe", &wide("mstsc.ex")));
        assert!(!eq_ignore_ascii_case_wide("mstsc.exe", &wide("mstsc.exee")));
    }

    #[test]
    fn wide_comparison_rejects_non_ascii() {
        assert!(!eq_ignore_ascii_case_wide("mstsc.exe", &wide("mstsc.exé")));
    }

    #[test]
    fn wide_comparison_rejects_empty_against_nonempty() {
        assert!(!eq_ignore_ascii_case_wide("mstsc.exe", &wide("")));
        assert!(eq_ignore_ascii_case_wide("", &wide("")));
    }

    // --- 002: process and class matches -----------------------

    #[test]
    fn configured_process_passes_through() {
        let policy = default_policy();
        let p = wide("mstsc.exe");
        let c = wide("SomeClass");
        let id = WideIdentity {
            process: Some(&p),
            class: Some(&c),
        };
        assert_eq!(
            classify_wide(&policy, &id),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    #[test]
    fn process_match_is_case_insensitive() {
        let policy = default_policy();
        let p = wide("VirtualBoxVM.EXE");
        let c = wide("SomeClass");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c)
                }
            ),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    #[test]
    fn configured_class_passes_through_when_process_does_not_match() {
        let policy = default_policy();
        let p = wide("unknown.exe");
        let c = wide("VMwareUnityWindow");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c)
                }
            ),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
    }

    #[test]
    fn class_match_is_case_insensitive() {
        let policy = default_policy();
        let p = wide("unknown.exe");
        let c = wide("vmwareUNITYwindow");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c)
                }
            ),
            BypassDecision::Passthrough(BypassReason::ClassMatch)
        );
    }

    // --- confirmed non-match -----------------------------------

    #[test]
    fn fully_resolved_non_match_continues_wira_desk_matching() {
        let policy = default_policy();
        let p = wide("notepad.exe");
        let c = wide("Notepad");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c)
                }
            ),
            BypassDecision::ContinueWiraDeskMatching
        );
    }

    // --- conservative failure -----------------------------------

    #[test]
    fn unresolved_process_fails_open() {
        let policy = default_policy();
        let c = wide("Notepad");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: None,
                    class: Some(&c)
                }
            ),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn unresolved_class_fails_open() {
        let policy = default_policy();
        let p = wide("notepad.exe");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: None
                }
            ),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn fully_unresolved_fails_open() {
        let policy = default_policy();
        assert_eq!(
            classify_wide(&policy, &WideIdentity::default()),
            BypassDecision::Passthrough(BypassReason::IdentityUnavailable)
        );
    }

    #[test]
    fn match_still_wins_over_partial_uncertainty() {
        let policy = default_policy();
        let p = wide("mstsc.exe");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: None
                }
            ),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
    }

    // --- policy immutability and duplicates --------------------

    #[test]
    fn duplicate_and_mixed_case_config_entries_collapse_correctly() {
        let cfg = VmBypassConfig {
            bypass_processes: vec![
                "MSTSC.EXE".to_string(),
                "mstsc.exe".to_string(),
                "  MsTsC.ExE  ".to_string(),
            ],
            bypass_classes: vec!["VMwareUnityWindow".to_string()],
        };
        let policy = BypassPolicy::from_config(&cfg);
        // Duplicates are harmless: matching is any, so the decision is stable.
        let p = wide("mstsc.exe");
        let c = wide("X");
        assert_eq!(
            classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c)
                }
            ),
            BypassDecision::Passthrough(BypassReason::ProcessMatch)
        );
        assert!(policy.processes().iter().all(|e| e == "mstsc.exe"));
    }

    #[test]
    fn policy_is_immutable_for_the_lifetime_of_a_configuration() {
        let policy = default_policy();
        let snapshot = policy.clone();
        let p = wide("notepad.exe");
        let c = wide("Notepad");
        for _ in 0..8 {
            let _ = classify_wide(
                &policy,
                &WideIdentity {
                    process: Some(&p),
                    class: Some(&c),
                },
            );
        }
        assert_eq!(policy, snapshot, "classification mutated the policy");
    }

    #[test]
    fn wide_and_owned_classification_agree() {
        // The Hook path and the Worker path must never disagree.
        let policy = default_policy();
        for (proc_s, class_s) in [
            (Some("mstsc.exe"), Some("X")),
            (Some("x.exe"), Some("VMwareUnityWindow")),
            (Some("notepad.exe"), Some("Notepad")),
            (None, Some("Notepad")),
            (Some("notepad.exe"), None),
            (None, None),
        ] {
            let pw = proc_s.map(wide);
            let cw = class_s.map(wide);
            let wide_id = WideIdentity {
                process: pw.as_deref(),
                class: cw.as_deref(),
            };
            let owned = identity(proc_s, class_s);
            assert_eq!(
                classify_wide(&policy, &wide_id),
                policy.classify(&owned),
                "hook and worker paths disagree for {proc_s:?}/{class_s:?}"
            );
        }
    }

    // --- deferred diagnostics ----------------------------------------------

    #[test]
    fn identity_failure_counter_is_readable_and_resettable() {
        reset_identity_failures();
        assert_eq!(identity_failure_count(), 0);
        IDENTITY_FAILURES.fetch_add(3, Ordering::Relaxed);
        assert_eq!(identity_failure_count(), 3);
        reset_identity_failures();
        assert_eq!(identity_failure_count(), 0);
    }

    #[test]
    fn collector_buffers_are_reused_not_reallocated() {
        // Structural: the collector owns fixed arrays, so repeated collection
        // cannot allocate. This asserts the type shape rather than behaviour.
        let mut c = HookIdentityCollector::new();
        assert_eq!(c.class.len(), CLASS_CAPACITY);
        assert_eq!(c.path.len(), PATH_CAPACITY);
        let _ = c.collect();
        assert_eq!(c.class.len(), CLASS_CAPACITY);
        assert_eq!(c.path.len(), PATH_CAPACITY);
    }
}
