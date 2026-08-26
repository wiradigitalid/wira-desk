//! Physical-monitor resolution. Worker-domain, no COM.
//! `MonitorFromWindow` is a bounded non-blocking lookup. The origin monitor is
//! resolved **once** per cycle operation by the caller and reused for every
//! candidate; nothing is cached between commands.

use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, RECT, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO,
    MONITOR_DEFAULTTONULL,
};

use crate::arrangement::{Rect, WorkArea};
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

// ── Live enumeration of the attached display set ────────────────────────────────
//
// `AD-14`: enumerated fresh on every command that needs it, and cached nowhere — not in a
// `static`, not memoized, and with no display-change subscription, because nothing is stored
// for such a notification to invalidate. An `HMONITOR` is a handle, not an identity: it does
// not survive an unplug, so a list kept between keypresses would outlive the configuration
// it described.

/// One attached monitor, as the arrangement planners need it.
///
/// Carries the work area and DPI and **not** the `HMONITOR`. The planners are pure geometry
/// with no Win32 in them, and a struct that never receives a handle cannot cache one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorGeometry {
    pub work: WorkArea,
    /// Full monitor bounds (`rcMonitor`), which the border clamp needs — the work area
    /// excludes the taskbar and clamping to it would inset every placement by its height.
    pub bounds: Rect,
}

/// Accumulator handed to the enumeration callback through `LPARAM`.
struct Collector {
    monitors: Vec<MonitorGeometry>,
}

/// `EnumDisplayMonitors` callback. Skips any monitor whose geometry cannot be read or is
/// degenerate, rather than failing the whole enumeration: one unreadable display must not
/// make the command impossible on the others.
///
/// # Safety
/// Called only by `EnumDisplayMonitors` from [`enumerate_monitors`], which passes a pointer
/// to a live `Collector` on its own stack as `lparam` and does not return until enumeration
/// has finished. `monitor` is a valid `HMONITOR` supplied by the OS for the duration of the
/// call.
unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    if lparam == 0 {
        return FALSE;
    }
    let collector = &mut *(lparam as *mut Collector);

    // SAFETY: `MONITORINFO` is a plain C struct — a size field, two `RECT`s and a flag word
    // — so the all-zero bit pattern is a valid value and `zeroed` cannot produce an invalid
    // one. `cbSize` is the load-bearing field: it is how `GetMonitorInfoW` decides how much
    // of the struct it may write, so it is set to the size of the exact type declared here.
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(monitor, &mut info) == FALSE {
        return TRUE; // keep enumerating; this display simply does not participate
    }

    let Ok(work_rect) = crate::arrangement::win32::rect_from_win32(info.rcWork) else {
        return TRUE;
    };
    let Ok(bounds) = crate::arrangement::win32::rect_from_win32(info.rcMonitor) else {
        return TRUE;
    };
    // DPI is read per monitor rather than inherited from the foreground window's: on a mixed
    // scaling setup those differ, and the destination's own value is the one a reader of the
    // plan would expect to see. It is carried for traceability only; no planner scales by it.
    let dpi = crate::arrangement::win32::monitor_dpi(monitor);
    if let Ok(work) = WorkArea::new(work_rect, dpi) {
        collector.monitors.push(MonitorGeometry { work, bounds });
    }
    TRUE
}

/// Every attached monitor, in the order Windows reports them.
///
/// The order is what "next monitor" walks (`LBR-WM-7`). It is deliberately not sorted by
/// coordinate: coordinates give no usable ordering for monitors stacked vertically or
/// arranged in an L, and the resulting surprise would happen on a desk that cannot be
/// reproduced here.
pub fn enumerate_monitors() -> Vec<MonitorGeometry> {
    let mut collector = Collector {
        monitors: Vec::new(),
    };
    // SAFETY: both device-context and clip-rect arguments are null, which the API documents
    // as "the whole virtual screen, no clipping". The callback is a valid `extern "system"`
    // function and `lparam` is a pointer to `collector`, which lives on this stack frame for
    // the whole call — `EnumDisplayMonitors` is synchronous and retains nothing afterwards.
    unsafe {
        EnumDisplayMonitors(
            0 as HDC,
            std::ptr::null(),
            Some(collect_monitor),
            &mut collector as *mut Collector as LPARAM,
        );
    }
    collector.monitors
}

/// Index of the monitor hosting `window` within [`enumerate_monitors`]'s order.
///
/// Matched by work-area equality rather than by handle, because the planners never receive a
/// handle and matching on one would mean threading an `HMONITOR` through them purely to look
/// it up again. Two monitors cannot share a work area — work areas partition the virtual
/// desktop — so equality identifies exactly one entry.
pub fn index_of_window_monitor(window: HWND, monitors: &[MonitorGeometry]) -> Option<usize> {
    if window == 0 {
        return None;
    }
    // SAFETY: resolves the handle through the window manager rather than dereferencing it,
    // so a stale handle yields null, which is checked. Passes no memory in, retains none.
    let handle = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONULL) };
    if handle == 0 {
        return None;
    }
    // SAFETY: `handle` is a non-null `HMONITOR` from the call above; `cbSize` is set to the
    // size of the exact type declared, which is what bounds the write.
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    // SAFETY: same contract as the `zeroed` above — `handle` is a non-null `HMONITOR` from
    // `MonitorFromWindow`, `&mut info` is a unique pointer to a live local of exactly the
    // declared type, and `cbSize` bounds the write.
    if unsafe { GetMonitorInfoW(handle, &mut info) } == FALSE {
        return None;
    }
    let work = crate::arrangement::win32::rect_from_win32(info.rcWork).ok()?;
    monitors.iter().position(|m| m.work.rect == work)
}
