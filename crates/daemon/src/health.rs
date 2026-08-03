//! Pure heartbeat timer for Hook Health Monitoring.
//! This thread does NOT touch `HHOOK` — it only `PostThreadMessageW` ticks
//! `WM_APP_HOOK_CHECK` to the Hook Thread every `HOOK_HEARTBEAT_SECS` seconds.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW;

use shared::constants::{HOOK_HEARTBEAT_SECS, WM_APP_HOOK_CHECK};

/// Start the heartbeat thread after the Hook Thread is ready (`thread_id` from `WM_APP_HOOK_READY`).
pub fn spawn(hook_thread_id: u32, shutdown: Arc<AtomicBool>) {
    thread::spawn(move || {
        while !shutdown.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(HOOK_HEARTBEAT_SECS));
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            // SAFETY: `hook_thread_id` need not still name a live thread. Thread ids are
            // integers, not handles, so `PostThreadMessageW` validates it internally and
            // fails with `ERROR_INVALID_THREAD_ID` rather than faulting — that is exactly
            // why the result is discarded instead of checked. Both `wParam` and `lParam`
            // are zero, so nothing is leaked when a tick is dropped: this heartbeat is
            // pure signal, and the Hook Thread's absence is the condition it exists to
            // surface rather than a state it must handle.
            unsafe {
                let _ = PostThreadMessageW(hook_thread_id, WM_APP_HOOK_CHECK, 0, 0);
            }
        }
    });
}
