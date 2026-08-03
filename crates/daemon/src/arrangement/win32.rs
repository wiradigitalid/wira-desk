//! Non-blocking Win32 arrangement adapter for placement plans.
//! Resolves platform context fresh for every command and applies placements
//! without activating, reordering, or blocking. Nothing is cached between
//! commands : no `static` geometry, no memoized monitor, no DPI
//! snapshot.
//! Absent by design : `SendMessage`, `GetWindowText`, any internal
//! geometry cache, and any virtual-desktop integration.

use windows_sys::Win32::Foundation::{FALSE, HWND, RECT, S_OK};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONULL,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, IsWindow, SetWindowPos, SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS,
    SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER,
};

use crate::cycling::WindowId;

use super::{Placement, PlanError, Rect, WindowMover, WorkArea};

/// Default DPI when `GetDpiForWindow` cannot answer. 96 is the Win32 baseline.
const DEFAULT_DPI: u32 = 96;

/// Platform context resolved once per arrangement command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformContext {
    pub target: WindowId,
    pub work_area: WorkArea,
}

/// Resolve the foreground window, its monitor work area, and its DPI.
/// Returns `None` when any part cannot be determined, so the caller performs no
/// partial arrangement.
pub fn resolve_context() -> Option<PlatformContext> {
    // SAFETY: no arguments, no memory we own, and a zero return simply means nothing holds
    // the foreground — checked immediately below rather than passed on as a handle.
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == 0 {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace("ARRANGE_CONTEXT: no_foreground_window=1");
        return None;
    }
    resolve_context_for(hwnd)
}

/// Same as [`resolve_context`] for an explicit window.
pub fn resolve_context_for(hwnd: HWND) -> Option<PlatformContext> {
    // SAFETY: `IsWindow` looks the handle up in the window manager instead of dereferencing
    // it, so any `isize` is a legal argument and an unusable one answers false. It reports
    // existence only — never responsiveness — so a hung window passes exactly like a healthy
    // one, which is what keeps this whole module non-blocking.
    if hwnd == 0 || unsafe { IsWindow(hwnd) } == FALSE {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!("ARRANGE_CONTEXT: invalid_target target={hwnd}"));
        return None;
    }

    // A cloaked window (suspended UWP surface, or a leftover from a
    // virtual-desktop switch) can transiently be `GetForegroundWindow`'s
    // answer while invisible to the user. Treat it the same as any other
    // "no valid target" case rather than silently building a context for it.
    // SAFETY: `is_cloaked` accepts any handle — a failed `DwmGetWindowAttribute` degrades to
    // "not cloaked" — and reads DWM's own state without messaging the owning thread.
    if unsafe { is_cloaked(hwnd) } {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!("ARRANGE_CONTEXT: cloaked target={hwnd}"));
        return None;
    }

    // `MONITOR_DEFAULTTONULL`: a window on no monitor must fail rather than be
    // arranged onto an arbitrary one.
    // SAFETY: resolves the handle rather than dereferencing it; passes no memory in and
    // retains none. A null result is checked below.
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL) };
    if monitor == 0 {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!("ARRANGE_CONTEXT: no_monitor target={hwnd}"));
        return None;
    }

    // SAFETY: `MONITORINFO` is a plain C struct — a size field, two `RECT`s, and a flag word —
    // so the all-zero bit pattern is valid and `zeroed` cannot produce an invalid value.
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    // SAFETY: `monitor` is a non-null `HMONITOR` from the call above, and `&mut info` is a
    // unique pointer to a live local. `cbSize` is the load-bearing part: it is how
    // `GetMonitorInfoW` decides how much of the struct it may write, so it is set to the size
    // of the exact type passed on the line above. A stale or oversized value there is the one
    // way this call writes out of bounds; a wrong one makes it fail rather than truncate.
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == FALSE {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!(
            "ARRANGE_CONTEXT: monitor_info_failed target={hwnd}"
        ));
        return None;
    }

    // `rcWork`, not `rcMonitor`: the taskbar and any reserved appbar are
    // already excluded, which is what the arrangement contract expects.
    let work_rect = match rect_from_win32(info.rcWork) {
        Ok(r) => r,
        Err(_) => {
            #[cfg(debug_assertions)]
            crate::util::append_debug_trace(&format!(
                "ARRANGE_CONTEXT: invalid_work_area target={hwnd}"
            ));
            return None;
        }
    };

    // Physical pixels in, physical pixels out. DPI is carried for traceability
    // and never used to rescale.
    // SAFETY: takes only the handle, writes nothing, and returns 0 when it cannot answer —
    // handled by falling back to the 96-DPI baseline rather than dividing by the result.
    let dpi = match unsafe { GetDpiForWindow(hwnd) } {
        0 => DEFAULT_DPI,
        d => d,
    };

    let work_area = match WorkArea::new(work_rect, dpi) {
        Ok(w) => w,
        Err(_) => {
            #[cfg(debug_assertions)]
            crate::util::append_debug_trace(&format!(
                "ARRANGE_CONTEXT: work_area_rejected target={hwnd}"
            ));
            return None;
        }
    };
    Some(PlatformContext {
        target: WindowId(hwnd),
        work_area,
    })
}

/// Ask the compositor whether it is drawing this window at all.
/// Duplicated locally from `cycling::source::is_cloaked` rather than imported
/// from that module, to keep `arrangement` and `cycling` decoupled (per this
/// file's story-isolation design). `IsWindowVisible` only reports the
/// `WS_VISIBLE` style bit, and a cloaked window keeps that bit set —
/// suspended UWP surfaces and windows left over from a virtual-desktop switch
/// both look perfectly ordinary through it. `DWMWA_CLOAKED` returns a
/// non-zero reason code (app, shell, or inherited) whenever the user cannot
/// actually see the window.
/// A failed query degrades to `false` — not cloaked — consistent with this
/// file's existing fail-safe posture elsewhere.
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

/// Convert a Win32 `RECT` into the contract's half-open [`Rect`].
/// `RECT` is already half-open on the right and bottom edges, so this is a
/// field copy plus validation — no coordinate adjustment, which is exactly what
/// keeps physical pixels intact.
pub fn rect_from_win32(r: RECT) -> Result<Rect, PlanError> {
    Rect::new(r.left, r.top, r.right, r.bottom)
}

/// Non-activating, Z-order-preserving, asynchronous. `SWP_ASYNCWINDOWPOS` is
/// what keeps a hung target from blocking the Worker: the request is posted
/// rather than delivered synchronously.
/// Hoisted into a shared const so the flag-pinning test below asserts the
/// flags [`Win32WindowMover::apply`] actually passes to `SetWindowPos`,
/// rather than a value the test recomputes independently.
const PLACEMENT_FLAGS: SET_WINDOW_POS_FLAGS =
    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_ASYNCWINDOWPOS;

/// Production mover.
pub struct Win32WindowMover;

impl WindowMover for Win32WindowMover {
    fn apply(&mut self, placement: &Placement) -> bool {
        let hwnd = placement.window.0;

        // Revalidate immediately before placement: a target may have closed
        // between planning and application.
        // SAFETY: handle-tolerant, as in `resolve_context_for` — existence only, never
        // responsiveness. Repeating the check here is not redundant: planning and application
        // are separate steps, and a window may close in between.
        if hwnd == 0 || unsafe { IsWindow(hwnd) } == FALSE {
            #[cfg(debug_assertions)]
            crate::util::append_debug_trace(&format!(
                "ARRANGE_APPLY: invalid_target target={hwnd}"
            ));
            return false;
        }

        let rect = placement.rect;
        let width = match rect.checked_width() {
            Some(w) if w > 0 => w,
            _ => {
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace(&format!(
                    "ARRANGE_APPLY: degenerate_width target={hwnd}"
                ));
                return false;
            }
        };
        let height = match rect.checked_height() {
            Some(h) if h > 0 => h,
            _ => {
                #[cfg(debug_assertions)]
                crate::util::append_debug_trace(&format!(
                    "ARRANGE_APPLY: degenerate_height target={hwnd}"
                ));
                return false;
            }
        };

        // SAFETY: `hwnd` was revalidated immediately above, and `width`/`height` are both
        // confirmed positive — `SetWindowPos` takes them as plain `i32`, so a negative or
        // overflowed value would be a nonsensical size request rather than a memory error, but
        // rejecting it here keeps the failure at the planner instead of the OS. No pointers are
        // passed, so nothing can dangle. `hwndInsertAfter` is 0, which is inert because
        // `PLACEMENT_FLAGS` sets `SWP_NOZORDER` and Windows then ignores that argument
        // entirely. `SWP_ASYNCWINDOWPOS` is what makes calling this from the Worker sound in
        // the blocking sense: the request is queued to the target's thread rather than
        // delivered synchronously, so a hung window cannot stall the command drain.
        let ok = unsafe {
            SetWindowPos(hwnd, 0, rect.left, rect.top, width, height, PLACEMENT_FLAGS) != FALSE
        };
        if !ok {
            #[cfg(debug_assertions)]
            {
                // SAFETY: no arguments and no memory involved. The real precondition is
                // ordering, not memory: `GetLastError` reports the *most recent* failure on
                // this thread, so it is only meaningful while nothing else has run in
                // between. Nothing has — only the `!= FALSE` comparison and this branch
                // separate it from the `SetWindowPos` above. Inserting any Win32 call
                // between the two would silently make this log the wrong error.
                let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
                crate::util::append_debug_trace(&format!(
                    "ARRANGE_APPLY: SetWindowPos_failed target={hwnd} err={err}"
                ));
            }
        }
        ok
    }
}

/// Apply a plan, skipping invalid targets.
/// Returns `(applied, skipped)`. One bad target never aborts the rest
/// , and nothing here can pop up a dialog or block.
pub fn apply_plan<M: WindowMover + ?Sized>(
    mover: &mut M,
    placements: &[Placement],
) -> (usize, usize) {
    let mut applied = 0;
    let mut skipped = 0;
    for placement in placements {
        if mover.apply(placement) {
            applied += 1;
        } else {
            skipped += 1;
        }
    }
    (applied, skipped)
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    /// Records every call so argument and ordering assertions are possible
    /// without touching User32.
    struct FakeMover {
        calls: Vec<Placement>,
        invalid: Vec<WindowId>,
    }

    impl FakeMover {
        fn new(invalid: Vec<WindowId>) -> Self {
            FakeMover {
                calls: Vec::new(),
                invalid,
            }
        }
    }

    impl WindowMover for FakeMover {
        fn apply(&mut self, placement: &Placement) -> bool {
            self.calls.push(*placement);
            !self.invalid.contains(&placement.window)
        }
    }

    fn placement(w: isize, left: i32, right: i32) -> Placement {
        Placement {
            window: WindowId(w),
            rect: Rect::new(left, 0, right, 100).unwrap(),
        }
    }

    // --- partial failure ---------------------------------------

    #[test]
    fn all_valid_targets_are_applied() {
        let mut mover = FakeMover::new(vec![]);
        let plan = [placement(1, 0, 100), placement(2, 100, 200)];
        assert_eq!(apply_plan(&mut mover, &plan), (2, 0));
        assert_eq!(mover.calls.len(), 2);
    }

    #[test]
    fn an_invalid_target_is_skipped_and_the_rest_continue() {
        let mut mover = FakeMover::new(vec![WindowId(2)]);
        let plan = [
            placement(1, 0, 100),
            placement(2, 100, 200),
            placement(3, 200, 300),
        ];
        assert_eq!(apply_plan(&mut mover, &plan), (2, 1));
        // Every target was still attempted, in order.
        let attempted: Vec<WindowId> = mover.calls.iter().map(|p| p.window).collect();
        assert_eq!(attempted, vec![WindowId(1), WindowId(2), WindowId(3)]);
    }

    #[test]
    fn every_target_invalid_yields_no_applications_and_no_panic() {
        let mut mover = FakeMover::new(vec![WindowId(1), WindowId(2)]);
        let plan = [placement(1, 0, 100), placement(2, 100, 200)];
        assert_eq!(apply_plan(&mut mover, &plan), (0, 2));
    }

    #[test]
    fn empty_plan_applies_nothing() {
        let mut mover = FakeMover::new(vec![]);
        assert_eq!(apply_plan(&mut mover, &[]), (0, 0));
        assert!(mover.calls.is_empty());
    }

    #[test]
    fn call_order_matches_plan_order() {
        let mut mover = FakeMover::new(vec![]);
        let plan = [
            placement(9, 0, 50),
            placement(4, 50, 100),
            placement(7, 100, 150),
        ];
        apply_plan(&mut mover, &plan);
        let order: Vec<WindowId> = mover.calls.iter().map(|p| p.window).collect();
        assert_eq!(order, vec![WindowId(9), WindowId(4), WindowId(7)]);
    }

    // --- RECT conversion preserves coordinates -----------------

    #[test]
    fn win32_rect_converts_without_adjustment() {
        let r = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };
        assert_eq!(
            rect_from_win32(r).unwrap(),
            Rect::new(0, 0, 1920, 1040).unwrap()
        );
    }

    #[test]
    fn negative_monitor_coordinates_survive_conversion() {
        let r = RECT {
            left: -1920,
            top: -200,
            right: 0,
            bottom: 880,
        };
        let converted = rect_from_win32(r).unwrap();
        assert_eq!(converted, Rect::new(-1920, -200, 0, 880).unwrap());
        assert_eq!(converted.width(), 1920);
    }

    #[test]
    fn degenerate_win32_rect_is_rejected() {
        let empty = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 100,
        };
        assert_eq!(
            rect_from_win32(empty),
            Err(PlanError::EmptyOrInvertedWorkArea)
        );
    }

    #[test]
    fn conversion_never_rescales_by_dpi() {
        // Same RECT must convert identically regardless of the DPI later
        // attached to the WorkArea.
        let r = RECT {
            left: 100,
            top: 100,
            right: 1000,
            bottom: 900,
        };
        let base = rect_from_win32(r).unwrap();
        for work in dpi_variants() {
            let again = WorkArea::new(base, work.dpi).unwrap();
            assert_eq!(again.rect, base, "DPI {} altered the rect", work.dpi);
        }
    }

    // --- 003: real adapter degrades safely --------------------

    #[test]
    fn context_for_invalid_window_is_none() {
        assert_eq!(resolve_context_for(0), None);
        assert_eq!(resolve_context_for(-1), None);
    }

    #[test]
    fn real_mover_rejects_invalid_targets() {
        let mut mover = Win32WindowMover;
        assert!(!mover.apply(&placement(0, 0, 100)));
        assert!(!mover.apply(&placement(-1, 0, 100)));
    }

    // `real_mover_rejects_degenerate_geometry` was removed: it used
    // `WindowId(0)`, which the earlier `hwnd == 0` guard already rejects
    // before `apply` ever reaches the width/height check, making it an
    // accidental duplicate of `real_mover_rejects_invalid_targets` above.
    // Exercising the actual `w <= 0`/`h <= 0` branch against a real mover
    // requires a live HWND, which is the already-disclosed
    // live-window-harness gap, not something fakeable here.

    #[test]
    fn placement_flags_are_non_activating_and_z_order_preserving() {
        // Pins the flag set: dropping SWP_NOACTIVATE would steal focus, and
        // dropping SWP_ASYNCWINDOWPOS would let a hung window block the
        // Worker. References the same `PLACEMENT_FLAGS` const that
        // `apply` passes to `SetWindowPos`, so a dropped flag in the real
        // code path fails this test too, rather than a locally recomputed
        // copy that could never disagree with itself.
        assert_ne!(PLACEMENT_FLAGS & SWP_NOACTIVATE, 0);
        assert_ne!(PLACEMENT_FLAGS & SWP_NOZORDER, 0);
        assert_ne!(PLACEMENT_FLAGS & SWP_NOOWNERZORDER, 0);
        assert_ne!(PLACEMENT_FLAGS & SWP_ASYNCWINDOWPOS, 0);
    }
}
