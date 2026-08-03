//! Tier 1 (Fatal) — exactly one `MessageBoxW` then exit, no retry.

use windows_sys::Win32::System::Threading::ExitProcess;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};

use crate::util::message_box;

/// Show one error message box then exit with code 1. Divergent (`-> !`)
/// so the invariant "exactly once, no retry" is enforced automatically — the caller cannot
/// continue executing other paths after calling this.
pub fn fatal(msg: &str) -> ! {
    message_box(0, msg, "Wira Desk - Fatal Error", MB_OK | MB_ICONERROR);
    // `ExitProcess` is declared `-> !` in windows-sys — it never returns,
    // no extra `loop {}` needed to satisfy the type.
    //
    // SAFETY: the abrupt-termination hazard of `ExitProcess` — other threads die without
    // unwinding — does not apply on this path. `fatal` is reachable only from startup
    // failures, before the hook, worker, and heartbeat threads are spawned, so there is no
    // thread whose destructors could be skipped and no hook left installed to unhook. The
    // `message_box` above has already returned, meaning the user has dismissed it.
    unsafe { ExitProcess(1) }
}
