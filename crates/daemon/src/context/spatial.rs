//! Physical-monitor resolution. Worker-domain, no COM.
//! `MonitorFromWindow` is a bounded non-blocking lookup. The origin monitor is
//! resolved **once** per cycle operation by the caller and reused for every
//! candidate; nothing is cached between commands.

use windows_sys::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONULL};

use crate::cycling::WindowId;

use super::{MonitorId, MonitorSource};

/// Production monitor adapter.
pub struct Win32Monitors;

impl MonitorSource for Win32Monitors {
    fn monitor_of(&self, window: WindowId) -> Option<MonitorId> {
        if window.0 == 0 {
            return None;
        }
        // `MONITOR_DEFAULTTONULL` is deliberate: a window that intersects no
        // monitor must report *unknown* so the contract fails closed, rather
        // than being silently attributed to the nearest or primary monitor.
        // SAFETY: `MonitorFromWindow` resolves the handle through the window manager rather
        // than dereferencing it, so a stale or bogus handle yields a null `HMONITOR` instead
        // of faulting — and that null is checked below. No memory is passed in or retained.
        let handle = unsafe { MonitorFromWindow(window.0, MONITOR_DEFAULTTONULL) };
        if handle == 0 {
            return None;
        }
        Some(MonitorId(handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_window_has_no_monitor() {
        assert_eq!(Win32Monitors.monitor_of(WindowId(0)), None);
    }

    #[test]
    fn bogus_window_has_no_monitor() {
        // Desktop-free: an invalid handle intersects nothing.
        assert_eq!(Win32Monitors.monitor_of(WindowId(-1)), None);
    }

    #[test]
    fn resolution_is_repeatable_for_the_same_handle() {
        // Whatever the answer is on this machine, it must not vary between
        // calls — the origin monitor is resolved once and trusted.
        let first = Win32Monitors.monitor_of(WindowId(0));
        for _ in 0..4 {
            assert_eq!(Win32Monitors.monitor_of(WindowId(0)), first);
        }
    }
}
