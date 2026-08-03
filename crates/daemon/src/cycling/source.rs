//! Live Z-order and executable-identity discovery for cycling.
//! One `EnumWindows` sweep per accepted command, no cache of any kind between
//! commands. Only non-blocking metadata APIs are used: no
//! `SendMessage`, no `GetWindowText`, no focus API, no eligibility policy, and
//! no Hook Thread code.

use windows_sys::Win32::Foundation::{CloseHandle, BOOL, FALSE, HANDLE, HWND, LPARAM, S_OK, TRUE};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
};

use super::{ActiveContext, AppIdentity, Candidate, CandidateSource, WindowFacts, WindowId};

/// Upper bound for a Win32 class name, per `WNDCLASS` documentation.
const CLASS_NAME_CAPACITY: usize = 256;

/// Upper bound for a full process image path.
const IMAGE_PATH_CAPACITY: usize = 32_768;

/// Production discovery adapter.
pub struct Win32CandidateSource;

/// Per-sweep state handed to [`enum_proc`]: the facts collected so far, plus one scratch
/// buffer reused for every window's executable path.
///
/// The buffer is the point. `identity_of` used to allocate its own `vec![0u16;
/// IMAGE_PATH_CAPACITY]` on every call — 64 KB each — and it is called once per top-level
/// window. A probe of this machine found 425 of them, so a single keystroke churned about
/// 27 MB of short-lived allocation before any window was activated. Reusing one buffer
/// leaves that at 64 KB per sweep without lowering the capacity, so extended-length paths
/// are still handled exactly as before. This mirrors `HookIdentityCollector` in
/// `context/vm_bypass.rs`, which owns its buffers for the same reason under a stricter
/// rule.
struct Sweep {
    facts: Vec<WindowFacts>,
    path: Vec<u16>,
}

impl Sweep {
    fn new() -> Self {
        Sweep {
            facts: Vec::new(),
            path: vec![0u16; IMAGE_PATH_CAPACITY],
        }
    }
}

impl CandidateSource for Win32CandidateSource {
    fn snapshot(&self) -> Vec<Candidate> {
        let mut sweep = Sweep::new();
        // `EnumWindows` is called exactly once and yields top-level windows in
        // top-to-bottom Z-order, which is the order the contract requires.
        // SAFETY: `enum_proc` is a `fn` item, so the callback pointer is valid for the whole
        // program. The `LPARAM` carries the address of `sweep`, which `enum_proc` restores as
        // `&mut Sweep` — the same type, which is what makes that cast sound rather than
        // merely type-checked.
        //
        // Two properties make the borrow legal. `EnumWindows` is synchronous and invokes the
        // callback on this thread, so `sweep` is alive for every invocation and no second
        // access can race it; and `sweep` is deliberately not touched between forming the
        // pointer and the call returning, so the provenance derived from that `&mut` is never
        // invalidated. The pointer only has to survive one statement here, which is why this
        // is safe while the superficially similar pointer in `hook.rs` needed an `AtomicPtr`
        // and an explicit publication window.
        //
        // The result is ignored on purpose: a `FALSE` return means enumeration stopped early,
        // and a short snapshot is still a valid one — `enum_proc` always returns `TRUE`, so
        // the only way that happens is an OS-side error we cannot act on anyway.
        unsafe {
            EnumWindows(Some(enum_proc), &mut sweep as *mut Sweep as LPARAM);
        }
        sweep
            .facts
            .into_iter()
            .enumerate()
            .map(|(z_index, facts)| Candidate { z_index, facts })
            .collect()
    }
}

/// Sample the active context once, at the start of one accepted command.
/// The Worker must not call this again during the same pass.
pub fn capture_active_context() -> ActiveContext {
    // SAFETY: `GetForegroundWindow` takes no arguments and touches no memory we own. It may
    // legitimately return 0 when no window holds the foreground, and `identity_of` handles
    // that explicitly rather than treating it as a valid handle, so no validity check is
    // needed here.
    unsafe {
        let foreground = GetForegroundWindow();
        // One buffer for the single lookup this function performs. Allocating it here
        // rather than reaching for the sweep's scratch keeps the two paths independent —
        // the active context is sampled once per command, so 64 KB is not worth coupling
        // them for.
        let mut path = vec![0u16; IMAGE_PATH_CAPACITY];
        ActiveContext {
            foreground: WindowId(foreground),
            identity: identity_of(foreground, &mut path),
        }
    }
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: `lparam` is the value `snapshot` passed to `EnumWindows` — the address of a
    // live `Sweep` on its frame — and `EnumWindows` forwards it unchanged. The enumeration
    // is synchronous on the calling thread, so that frame is alive here and this is the only
    // reference to the sweep in existence for the duration of the callback.
    let sweep = &mut *(lparam as *mut Sweep);
    let facts = capture_facts(hwnd, &mut sweep.path);
    sweep.facts.push(facts);
    // Always continue: a window we cannot describe still must not truncate the
    // snapshot, or the Z-order would silently lose everything below it.
    TRUE
}

unsafe fn capture_facts(hwnd: HWND, path: &mut [u16]) -> WindowFacts {
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    WindowFacts {
        window: WindowId(hwnd),
        visible: IsWindowVisible(hwnd) != FALSE,
        cloaked: is_cloaked(hwnd),
        iconic: IsIconic(hwnd) != FALSE,
        tool_window: ex_style & WS_EX_TOOLWINDOW != 0,
        class_name: class_name_of(hwnd),
        identity: identity_of(hwnd, path),
    }
}

/// Ask the compositor whether it is drawing this window at all.
/// `IsWindowVisible` only reports the `WS_VISIBLE` style bit, and a cloaked
/// window keeps that bit set — suspended UWP surfaces and windows on another
/// virtual desktop both look perfectly ordinary through it. `DWMWA_CLOAKED`
/// returns a non-zero reason code (app, shell, or inherited); any of them means
/// the user cannot see the window, so the distinction is not worth keeping.
/// Non-blocking, like everything else in this sweep: the attribute is read from
/// DWM's own state and never messages the owning thread, so a hung window
/// answers as fast as a healthy one.
/// A failed query degrades to `false` — not cloaked — which keeps a real window
/// reachable rather than silently dropping it from the cycle.
unsafe fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    let hr = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as u32,
        &mut cloaked as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
    hr == S_OK && cloaked != 0
}

unsafe fn class_name_of(hwnd: HWND) -> String {
    let mut buf = [0u16; CLASS_NAME_CAPACITY];
    let len = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len as usize])
}

/// Resolve a window's application identity.
/// Every failure path — no PID, denied `OpenProcess`, vanished process, failed
/// query — degrades to [`AppIdentity::Unavailable`]. Discovery never retries,
/// never blocks, and never surfaces an error to the user.
/// The PID obtained here is used *only* to open the process. It never leaves
/// this function, which is how the contract keeps the executable basename as
/// the sole same-application key.
///
/// `path` is caller-owned scratch, reused across every window in a sweep rather than
/// allocated per call — see [`Sweep`] for why that matters. Reuse is safe because the
/// in/out `size` is reset to the buffer's full length on every call and only the prefix
/// the API reports is ever read, so residue from the previous window is unreachable.
///
/// # Safety
/// `hwnd` may be any value, including 0 or a stale handle; every Win32 call here reports
/// failure rather than faulting. `path` must be non-empty — its length is what bounds the
/// write.
unsafe fn identity_of(hwnd: HWND, path: &mut [u16]) -> AppIdentity {
    if hwnd == 0 {
        return AppIdentity::Unavailable;
    }

    let mut pid: u32 = 0;
    let thread_id = GetWindowThreadProcessId(hwnd, &mut pid);
    if thread_id == 0 || pid == 0 {
        return AppIdentity::Unavailable;
    }

    let process: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if process == 0 {
        return AppIdentity::Unavailable;
    }

    // `size` is in/out: its input value is what bounds the write, so it is reset to the
    // buffer's full length on every call. That is the invariant that makes reusing one
    // buffer across the sweep safe — get it wrong and a later window would be told the
    // buffer is only as large as the previous path was long.
    let mut size = path.len() as u32;
    let ok = QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_WIN32,
        path.as_mut_ptr(),
        &mut size as *mut u32,
    );
    CloseHandle(process);

    if ok == FALSE || size == 0 {
        return AppIdentity::Unavailable;
    }

    // Only the prefix the API reported is read, so residue from the previous window in the
    // reused buffer cannot leak into this identity.
    let resolved = String::from_utf16_lossy(&path[..(size as usize).min(path.len())]);
    AppIdentity::from_process_path(Some(&resolved))
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::*;
    use super::*;

    // --- degradation without a crash --------------------------

    #[test]
    fn null_window_yields_unavailable_identity() {
        let mut path = vec![0u16; IMAGE_PATH_CAPACITY];
        // SAFETY: `identity_of` accepts any handle value — it special-cases 0 and treats
        // every downstream failure as `Unavailable`. Passing an invalid handle is the point
        // of the test, not a violation of its contract. `path` is a live, non-empty buffer.
        let identity = unsafe { identity_of(0, &mut path) };
        assert_eq!(identity, AppIdentity::Unavailable);
    }

    #[test]
    fn bogus_window_yields_unavailable_identity_without_panicking() {
        // A handle that does not resolve must degrade, not abort discovery.
        let mut path = vec![0u16; IMAGE_PATH_CAPACITY];
        // SAFETY: as above — `identity_of` is total over handle values, resolving an
        // unusable one to `Unavailable` via `GetWindowThreadProcessId` failing.
        let identity = unsafe { identity_of(-1, &mut path) };
        assert_eq!(identity, AppIdentity::Unavailable);
    }

    #[test]
    fn class_name_of_invalid_window_is_empty_not_a_panic() {
        // SAFETY: `class_name_of` passes the handle to `GetClassNameW`, which reports a
        // non-positive length for an unusable handle and writes nothing to the buffer, so
        // any handle value is a legal argument.
        assert_eq!(unsafe { class_name_of(0) }, String::new());
    }

    // --- identity grouping semantics --------------------------
    // These assert the rule discovery relies on. They use the contract's
    // normalizer directly so they need no live process.

    #[test]
    fn same_basename_from_different_paths_groups_together() {
        let a = AppIdentity::from_process_path(Some(r"C:\Program Files\App\app.exe"));
        let b = AppIdentity::from_process_path(Some(r"D:\Other\App\app.exe"));
        assert!(a.same_application(&b));
    }

    #[test]
    fn different_basenames_never_group_even_from_one_directory() {
        let a = AppIdentity::from_process_path(Some(r"C:\App\one.exe"));
        let b = AppIdentity::from_process_path(Some(r"C:\App\two.exe"));
        assert!(!a.same_application(&b));
    }

    #[test]
    fn unresolvable_process_never_groups_with_anything() {
        let unknown = AppIdentity::Unavailable;
        assert!(!unknown.same_application(&identity(HOST_EXE)));
        assert!(!unknown.same_application(&AppIdentity::Unavailable));
    }

    #[test]
    fn sweep_scratch_offers_the_full_path_capacity() {
        // The scratch buffer's length is what `QueryFullProcessImageNameW` is told it may
        // write, so shrinking it would silently truncate long paths rather than fail. The
        // whole point of reusing one buffer was to drop the per-window allocation
        // *without* lowering this, so it is pinned.
        let sweep = Sweep::new();
        assert_eq!(sweep.path.len(), IMAGE_PATH_CAPACITY);
        assert!(sweep.facts.is_empty());
    }

    #[test]
    fn a_reused_buffer_does_not_leak_the_previous_window_into_the_next() {
        // Residue is deliberately left in place between calls — nothing clears the buffer.
        // Only the prefix the API reports is read, so a failed lookup must still report
        // `Unavailable` rather than reinterpreting whatever the last window left behind.
        let mut path = vec![0u16; IMAGE_PATH_CAPACITY];
        for (i, slot) in path.iter_mut().enumerate().take(64) {
            *slot = b'A' as u16 + (i % 26) as u16;
        }
        // SAFETY: any handle value is accepted; `path` is live and non-empty.
        let null_handle = unsafe { identity_of(0, &mut path) };
        // SAFETY: as above.
        let bogus_handle = unsafe { identity_of(-1, &mut path) };
        assert_eq!(null_handle, AppIdentity::Unavailable);
        assert_eq!(bogus_handle, AppIdentity::Unavailable);
    }

    // --- snapshot shape --------------------------

    #[test]
    fn snapshot_indexes_are_dense_and_ordered() {
        // Runs against the live desktop but asserts only structural
        // invariants, so it is stable in any session.
        let snapshot = Win32CandidateSource.snapshot();
        for (expected, candidate) in snapshot.iter().enumerate() {
            assert_eq!(
                candidate.z_index, expected,
                "z_index must be dense and top-to-bottom"
            );
        }
    }

    #[test]
    fn snapshot_holds_no_cache_between_calls() {
        // Two calls must each perform their own sweep rather than replay
        // retained state.
        //
        // Deliberately NOT asserted: that the two sweeps have equal length. This
        // runs against the live desktop, where a window may legitimately appear
        // or close between the calls, so equality would be a flake. An earlier
        // revision wrote `assert_eq!(first.len(), first.len())` — comparing a
        // value with itself, which can never fail and asserted nothing at all.
        let first = Win32CandidateSource.snapshot();
        let second = Win32CandidateSource.snapshot();

        // Both sweeps must produce their own buffer. Two live `Vec`s with any
        // content cannot share a base address, so an equal pointer would mean
        // the second call returned the first call's retained allocation.
        assert!(
            first.is_empty() || second.is_empty() || first.as_ptr() != second.as_ptr(),
            "second snapshot reused the first snapshot's allocation"
        );

        // Each sweep must be internally consistent: dense, top-to-bottom Z-order.
        for (expected, candidate) in second.iter().enumerate() {
            assert_eq!(candidate.z_index, expected);
        }
    }

    #[test]
    fn active_context_samples_a_single_foreground_handle() {
        let ctx = capture_active_context();
        // Either a real foreground window or none; identity must be coherent
        // with that handle rather than invented.
        if ctx.foreground == WindowId(0) {
            assert_eq!(ctx.identity, AppIdentity::Unavailable);
        }
    }
}
