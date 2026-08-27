//! Non-blocking Win32 arrangement adapter for placement plans.
//! Resolves platform context fresh for every command and applies placements
//! without activating, reordering, or blocking. Nothing is cached between
//! commands : no `static` geometry, no memoized monitor, no DPI
//! snapshot.
//! Absent by design : `SendMessage`, `GetWindowText`, any internal
//! geometry cache, and any virtual-desktop integration.

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HWND, RECT, S_OK};
use windows_sys::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromRect, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONULL,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, GetWindowRect, GetWindowThreadProcessId, IsWindow,
    SetWindowPos, ShowWindowAsync, GWL_STYLE, SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS,
    SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER, SW_MAXIMIZE, WS_MAXIMIZEBOX,
};

use shared::constants::SETTINGS_EXE_NAME;

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

    // The daemon must never grow its own Settings window into an arrangement
    // target (DEC-006, LBR-WM-6): Settings is frameless, transparent, and
    // laid out at a fixed size, so an enlarged frame leaves an invisible
    // region that still swallows clicks. Checked after cloaking (an invalid
    // or invisible target is already excluded) and before any geometry is
    // touched, so the sequence reads validity → visibility → ownership →
    // geometry.
    // SAFETY: `is_own_window` accepts any handle — every identity lookup inside it
    // degrades to "not ours" on failure rather than propagating an error, so no
    // precondition is placed on `hwnd` beyond what `IsWindow` already confirmed above.
    if unsafe { is_own_window(hwnd) } {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!("ARRANGE_CONTEXT: own_window target={hwnd}"));
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

/// This monitor's effective DPI, falling back to the 96-DPI baseline.
///
/// The per-*monitor* counterpart of `GetDpiForWindow`, needed because a monitor move reads
/// the destination's scaling before any window is on it. Carried for traceability only; no
/// planner scales by it, and applying it would scale coordinates a second time.
pub fn monitor_dpi(monitor: windows_sys::Win32::Graphics::Gdi::HMONITOR) -> u32 {
    let mut x: u32 = 0;
    let mut y: u32 = 0;
    // SAFETY: `monitor` is a non-null `HMONITOR` from the caller, and both out-params are
    // unique pointers to live locals. `MDT_EFFECTIVE_DPI` is the documented value for the
    // scaling Windows actually applies. A failed call leaves the locals untouched, which is
    // why they are pre-initialised and the result is checked rather than trusted.
    let ok = unsafe {
        windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(
            monitor,
            windows_sys::Win32::UI::HiDpi::MDT_EFFECTIVE_DPI,
            &mut x,
            &mut y,
        )
    };
    // Horizontal DPI only: Windows reports the two separately but sets them equal for every
    // display mode this product supports, and the arrangement contract carries one number.
    if ok == 0 && x != 0 {
        x
    } else {
        DEFAULT_DPI
    }
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

/// Upper bound for a full process image path, matching the capacity used
/// elsewhere in the workspace for `QueryFullProcessImageNameW`.
const IMAGE_PATH_CAPACITY: usize = 32_768;

/// Ask whether `hwnd` belongs to the daemon itself or to the Settings process.
/// Duplicated locally rather than imported from `crate::cycling`, to keep
/// `arrangement` and `cycling` decoupled (per this file's story-isolation
/// design, same as [`is_cloaked`] above).
///
/// Identity is read with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ..)` +
/// `QueryFullProcessImageNameW` and nothing else — no `SendMessage`, no
/// `GetWindowText`, no blocking cross-process call (`LBR-WM-3`).
///
/// Fails open, not closed: every lookup failure (no PID, denied `OpenProcess`,
/// vanished process, failed query) degrades to "not ours" and lets the
/// arrangement proceed, exactly like [`is_cloaked`]'s degrade-to-"not cloaked".
/// Failing closed would silently kill snapping for any window whose process
/// cannot be opened; the residual false negative this leaves (an enlarged
/// frame this check missed) is covered on the Settings side instead.
///
/// # Safety
/// `hwnd` may be any value, including 0 or a stale handle — `GetWindowThreadProcessId`
/// reports failure rather than faulting on one, and every Win32 call below is checked
/// before its result is used.
unsafe fn is_own_window(hwnd: HWND) -> bool {
    let mut pid: u32 = 0;
    let thread_id = GetWindowThreadProcessId(hwnd, &mut pid);
    if thread_id == 0 || pid == 0 {
        return false;
    }

    if pid == GetCurrentProcessId() {
        return true;
    }

    // PROCESS_QUERY_LIMITED_INFORMATION is the narrowest right that still permits
    // `QueryFullProcessImageNameW`.
    let process: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if process == 0 {
        return false;
    }

    // One `vec![0u16; ..]` buffer, allocated fresh for this call. This path runs
    // once per arrangement command on the Worker thread, never once per window
    // across a sweep of hundreds — unlike `cycling::source`, which reuses a
    // scratch buffer for exactly that reason. There is no equivalent reuse
    // opportunity here, so this is deliberate, not an oversight to "fix".
    let mut path = vec![0u16; IMAGE_PATH_CAPACITY];
    let mut size = path.len() as u32;
    let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, path.as_mut_ptr(), &mut size);
    CloseHandle(process);

    if ok == FALSE || size == 0 {
        return false;
    }

    basename_matches_settings_exe(&path[..(size as usize).min(path.len())])
}

/// Pure comparison over a UTF-16 process-image path: extract the basename
/// (after the last `\` or `/`, or the whole slice if there is none) and
/// compare it case-insensitively against [`SETTINGS_EXE_NAME`].
/// Kept separate from [`is_own_window`] so the comparison is unit-testable
/// without a live window or process handle.
fn basename_matches_settings_exe(path: &[u16]) -> bool {
    let basename_start = path
        .iter()
        .rposition(|&unit| unit == b'\\' as u16 || unit == b'/' as u16)
        .map(|i| i + 1)
        .unwrap_or(0);
    let basename = String::from_utf16_lossy(&path[basename_start..]);
    basename.eq_ignore_ascii_case(SETTINGS_EXE_NAME)
}

/// Convert a Win32 `RECT` into the contract's half-open [`Rect`].
/// `RECT` is already half-open on the right and bottom edges, so this is a
/// field copy plus validation — no coordinate adjustment, which is exactly what
/// keeps physical pixels intact.
pub fn rect_from_win32(r: RECT) -> Result<Rect, PlanError> {
    Rect::new(r.left, r.top, r.right, r.bottom)
}

/// The gap between a window's true outer rect (`GetWindowRect`) and its
/// visible frame (`DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`).
/// Windows 10 replaced the classic thick resize border with a border that is
/// hit-testable but invisible, extending a few pixels *outside* the visible
/// frame on most windows — so `SetWindowPos`, which positions the outer
/// rect, leaves that many pixels of gap between the visible window and
/// whatever edge it was asked to touch. Chromium-based apps (Chrome, Edge,
/// and Electron apps) draw their own frame and tend to carry a larger,
/// more visible instance of exactly this gap, which is what actually
/// surfaces the problem to a user snapping windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct FrameInsets {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl FrameInsets {
    /// Clamped to non-negative: Windows makes no promise that the extended
    /// frame is never larger than the outer rect, and a negative inset here
    /// would flip [`compensate_for_frame_insets`] into shrinking the target
    /// instead of widening past it.
    fn from_rects(outer: RECT, extended: RECT) -> FrameInsets {
        FrameInsets {
            left: (extended.left - outer.left).max(0),
            top: (extended.top - outer.top).max(0),
            right: (outer.right - extended.right).max(0),
            bottom: (outer.bottom - extended.bottom).max(0),
        }
    }
}

/// Best-effort: `None` on any Win32/DWM failure, which the caller treats as
/// "no border" — the placement still lands, just without compensation,
/// rather than failing the whole command over a diagnostic query.
///
/// SAFETY: `hwnd` must be a value `IsWindow` has already confirmed live —
/// true of `apply`'s only call site, immediately after that check. Both
/// out-params are live locals of the exact type/size the two APIs are told
/// to write into, and neither API retains the pointer past the call.
unsafe fn frame_insets(hwnd: HWND) -> Option<FrameInsets> {
    let mut outer: RECT = std::mem::zeroed();
    if GetWindowRect(hwnd, &mut outer) == FALSE {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!(
            "ARRANGE_INSETS: GetWindowRect_failed target={hwnd}"
        ));
        return None;
    }
    let mut extended: RECT = std::mem::zeroed();
    let hr = DwmGetWindowAttribute(
        hwnd,
        DWMWA_EXTENDED_FRAME_BOUNDS as u32,
        &mut extended as *mut RECT as *mut core::ffi::c_void,
        core::mem::size_of::<RECT>() as u32,
    );
    if hr != S_OK {
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!(
            "ARRANGE_INSETS: DwmGetWindowAttribute_failed target={hwnd} hr={hr:#x}"
        ));
        return None;
    }
    let insets = FrameInsets::from_rects(outer, extended);
    #[cfg(debug_assertions)]
    crate::util::append_debug_trace(&format!(
        "ARRANGE_INSETS: target={hwnd} \
         outer=({},{},{},{}) extended=({},{},{},{}) insets={insets:?}",
        outer.left,
        outer.top,
        outer.right,
        outer.bottom,
        extended.left,
        extended.top,
        extended.right,
        extended.bottom,
    ));
    Some(insets)
}

/// Widen `target` outward by `insets` so that, once passed to
/// `SetWindowPos`, the window's *visible* frame lands on `target` instead
/// of its outer rect landing there. `None` only on coordinate overflow —
/// the same [`PlanError::UnrepresentableGeometry`] shape `Rect` itself
/// guards against, though this returns `Option` rather than that enum
/// since [`Win32WindowMover::apply`] has no planning-stage error to report
/// through; a `None` here falls back to the uncompensated `target`.
fn compensate_for_frame_insets(target: Rect, insets: FrameInsets) -> Option<Rect> {
    Rect::new(
        target.left.checked_sub(insets.left)?,
        target.top.checked_sub(insets.top)?,
        target.right.checked_add(insets.right)?,
        target.bottom.checked_add(insets.bottom)?,
    )
    .ok()
}

/// Clamp `rect` to `monitor`'s full physical extent (`rcMonitor`, never the
/// work area — the invisible border legitimately occupies the strip between
/// the work area and the true monitor edge).
///
/// Widening outward by [`FrameInsets`] is only safe up to the monitor's own
/// pixels. At the *outermost* edge of the whole virtual desktop — a monitor
/// with nothing adjacent to it, which is exactly where a leftmost or
/// topmost monitor's border sits — there is no neighbour to absorb the
/// extra width, and without this clamp the compensated rect bleeds into
/// space no monitor occupies. Falls back to the unclamped `rect` if
/// clamping would invert it (degenerate on any real monitor, given
/// `insets` is a handful of pixels against a monitor thousands of pixels
/// wide) — the width/height check right after this call is what actually
/// rejects a placement that still cannot be made to fit.
///
/// Deliberately clamps against the *target's own* monitor, not the union of
/// every monitor's pixels. A union-based version was tried and measured
/// wrong on a real two-monitor, mixed-DPI setup (150% / 175%): letting the
/// outer rect cross even a few pixels into a neighbouring monitor with a
/// *different* DPI made Windows itself decide the window had moved onto
/// that monitor and forcibly re-rescale/reposition it for the new DPI —
/// visible as the window jumping hundreds of pixels right after this
/// placement, not the few-pixel bleed the union version intended. The
/// per-monitor clamp accepts a margin at a shared monitor boundary as the
/// safe trade-off instead of risking that.
fn clamp_to_monitor(rect: Rect, monitor: RECT) -> Rect {
    let left = rect.left.max(monitor.left);
    let top = rect.top.max(monitor.top);
    let right = rect.right.min(monitor.right);
    let bottom = rect.bottom.min(monitor.bottom);
    Rect::new(left, top, right, bottom).unwrap_or(rect)
}

/// The monitor containing `rect`, in full physical pixels (`rcMonitor`).
/// Best-effort: `None` on any failure, which the caller treats as "no known
/// bounds" — the placement still lands unclamped rather than failing the
/// whole command over a diagnostic query.
///
/// SAFETY: `hwnd` must be a value `IsWindow` has already confirmed live —
/// true of `apply`'s only call site. `&mut info` is a live local of the
/// exact struct `GetMonitorInfoW` is told to write into, with `cbSize` set
/// to that struct's size beforehand, which is how the call knows how much
/// of it may write.
unsafe fn monitor_rect_for(rect: Rect) -> Option<RECT> {
    let as_win32 = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    let monitor = MonitorFromRect(&as_win32, MONITOR_DEFAULTTONULL);
    if monitor == 0 {
        return None;
    }
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(monitor, &mut info) == FALSE {
        return None;
    }
    Some(info.rcMonitor)
}

/// Non-activating, Z-order-preserving, asynchronous. `SWP_ASYNCWINDOWPOS` is
/// what keeps a hung target from blocking the Worker: the request is posted
/// rather than delivered synchronously.
/// Hoisted into a shared const so the flag-pinning test below asserts the
/// flags [`Win32WindowMover::apply`] actually passes to `SetWindowPos`,
/// rather than a value the test recomputes independently.
const PLACEMENT_FLAGS: SET_WINDOW_POS_FLAGS =
    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_ASYNCWINDOWPOS;

/// Whether a window's style permits Windows' own maximize.
///
/// Split from the call below so the decision is reachable from a test while the API call is
/// not. `WS_MAXIMIZEBOX` is what puts a working maximize button on the title bar, so a window
/// without it is one whose author said it should not be maximized — a fixed-size dialog, a
/// tool palette, a picker.
pub fn style_allows_maximize(style: u32) -> bool {
    style & WS_MAXIMIZEBOX != 0
}

/// Maximize the way the title bar's own button does, rather than by resizing to the work
/// area.
///
/// **The two are not the same thing, and the difference is visible.** Sizing a window to the
/// work area leaves it in the *normal* state: `IsZoomed` stays false, the title bar still
/// offers Maximize rather than Restore, double-clicking it maximizes again to a slightly
/// different size, and the window never receives `WM_GETMINMAXINFO`, so an application that
/// asks for particular maximized bounds does not get them. It looks maximized and does not
/// behave maximized.
///
/// `SW_MAXIMIZE` is what the title bar sends, so all of that follows for free.
///
/// Returns `false` when the window's own style forbids it, which is the caller's signal to
/// fall back to the geometric plan. `ShowWindowAsync` rather than `ShowWindow` keeps this
/// thread off a cross-process wait, as `LBR-WM-3` requires.
pub fn try_real_maximize(window: WindowId) -> bool {
    // SAFETY: `GetWindowLongW` takes only a handle and an index, writes nothing through a
    // pointer, and returns zero for a stale handle — which `style_allows_maximize` then reads
    // as "not maximizable", so a window that closed between planning and here falls back
    // rather than acting on a garbage style word.
    let style = unsafe { GetWindowLongW(window.0, GWL_STYLE) } as u32;
    if !style_allows_maximize(style) {
        return false;
    }

    // SAFETY: takes only a handle and a documented show command, writes nothing, and is
    // documented to fail benignly on a stale handle. The result is discarded because a
    // failure here is indistinguishable from the window having closed, and the caller has
    // nothing better to do about either.
    unsafe {
        ShowWindowAsync(window.0, SW_MAXIMIZE);
    }
    true
}

/// Production mover.
pub struct Win32WindowMover;

impl WindowMover for Win32WindowMover {
    fn restore(&mut self, window: WindowId) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{IsZoomed, ShowWindowAsync, SW_RESTORE};

        // `SW_RESTORE` rather than `SW_SHOWNORMAL`: restore leaves a minimized window's
        // activation alone, and the arrangement path is deliberately non-activating.
        //
        // `ShowWindowAsync` rather than `ShowWindow` keeps this thread off a cross-process
        // wait, which `LBR-WM-3` bans — a hung target must not be able to hang the Worker.
        // SAFETY: both calls take only a window handle, write nothing through a pointer, and
        // are documented to fail benignly on a stale handle, which is why the result is
        // discarded rather than checked. `apply` revalidates the handle immediately after.
        unsafe {
            if IsZoomed(window.0) != 0 {
                ShowWindowAsync(window.0, SW_RESTORE);
            }
        }
    }

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

        // Compensate for the target's invisible resize border, if it has
        // one, so the *visible* window edge lands on `placement.rect`
        // rather than its outer (`GetWindowRect`) edge landing there —
        // otherwise a window with such a border (most windows, on Windows
        // 10+) ends up inset from the screen edge by exactly that border's
        // width when snapped or maximized. Best-effort: a failed query
        // degrades to zero insets, which is the placement this file always
        // produced before this compensation existed.
        // SAFETY: `hwnd` was revalidated live by `IsWindow` immediately above.
        let insets = unsafe { frame_insets(hwnd) }.unwrap_or_default();
        let rect = compensate_for_frame_insets(placement.rect, insets).unwrap_or(placement.rect);
        // Widening outward for the border is only safe up to the target monitor's actual
        // pixels — clamped here so the outermost monitor's edge (no neighbour to bleed into)
        // never sends part of the window into space no monitor occupies.
        //
        // The monitor is resolved from the **planned rect**, not from the window (`DEC-010`).
        // For every command that divides the window's own work area the two are the same
        // monitor. For a monitor move they are not: at this point the window is still on the
        // monitor it is leaving, so resolving from the window would clamp a rect planned for
        // the destination against the bounds of the source and collapse it. That is not a
        // cosmetic difference — `25f52f0` measured what happens when a compensated rect
        // touches a monitor at a different DPI: Windows decides the window moved there and
        // forcibly rescales and repositions it, hundreds of pixels off.
        // SAFETY: a pure query on a by-value rect; passes no borrowed memory and retains none.
        let rect = match unsafe { monitor_rect_for(rect) } {
            Some(monitor) => clamp_to_monitor(rect, monitor),
            None => rect,
        };
        #[cfg(debug_assertions)]
        crate::util::append_debug_trace(&format!(
            "ARRANGE_APPLY: target={hwnd} requested=({},{},{},{}) adjusted=({},{},{},{})",
            placement.rect.left,
            placement.rect.top,
            placement.rect.right,
            placement.rect.bottom,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
        ));
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
        // Restore first, always. Every arrangement command reaches the desktop through this
        // loop, so putting the step here is what stops the next one added from forgetting
        // it — which is the defect this fixes, not a hypothetical.
        mover.restore(placement.window);
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
    /// What the mover was asked to do, in order. Recording restore and apply in one
    /// sequence rather than two lists is deliberate: the property that matters is not that
    /// both happened, it is that restore happened *first* for each window.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum MoverCall {
        Restore(WindowId),
        Apply(Placement),
    }

    struct FakeMover {
        log: Vec<MoverCall>,
        invalid: Vec<WindowId>,
    }

    impl FakeMover {
        fn new(invalid: Vec<WindowId>) -> Self {
            FakeMover {
                log: Vec::new(),
                invalid,
            }
        }

        /// Just the placements, for the tests that only care about those.
        fn calls(&self) -> Vec<Placement> {
            self.log
                .iter()
                .filter_map(|c| match c {
                    MoverCall::Apply(p) => Some(*p),
                    MoverCall::Restore(_) => None,
                })
                .collect()
        }
    }

    impl WindowMover for FakeMover {
        fn restore(&mut self, window: WindowId) {
            self.log.push(MoverCall::Restore(window));
        }

        fn apply(&mut self, placement: &Placement) -> bool {
            self.log.push(MoverCall::Apply(*placement));
            !self.invalid.contains(&placement.window)
        }
    }

    fn placement(w: isize, left: i32, right: i32) -> Placement {
        Placement {
            window: WindowId(w),
            rect: Rect::new(left, 0, right, 100).unwrap(),
        }
    }

    /// The defect this guards was reported from real use: a window maximized by
    /// double-clicking its title bar could not be snapped. Only the move-to-monitor command
    /// restored first; snapping and stacking went straight to `SetWindowPos`, which on a
    /// still-maximized window leaves it where it was. The step now lives in `apply_plan`, so
    /// every command inherits it and a command added later cannot omit it.
    #[test]
    fn every_window_is_restored_before_it_is_moved() {
        let mut mover = FakeMover::new(vec![]);
        let plan = [placement(1, 0, 100), placement(2, 100, 200)];
        apply_plan(&mut mover, &plan);

        assert_eq!(
            mover.log,
            vec![
                MoverCall::Restore(WindowId(1)),
                MoverCall::Apply(plan[0]),
                MoverCall::Restore(WindowId(2)),
                MoverCall::Apply(plan[1]),
            ],
            "each window must be restored immediately before its own move"
        );
    }

    /// A window that cannot be moved is still restored first, because whether it can be
    /// moved is not known until `apply` is attempted. Restoring is what makes the attempt
    /// meaningful, so skipping it for a target that later fails would be backwards.
    #[test]
    fn a_target_that_fails_to_move_was_still_restored() {
        let mut mover = FakeMover::new(vec![WindowId(2)]);
        let plan = [placement(1, 0, 100), placement(2, 100, 200)];
        assert_eq!(apply_plan(&mut mover, &plan), (1, 1));
        assert!(mover.log.contains(&MoverCall::Restore(WindowId(2))));
    }

    // --- real maximize ------------------------------------------

    /// The gate that decides between Windows' own maximize and the geometric fallback.
    /// `WS_MAXIMIZEBOX` is the author's statement that a window may be maximized; without it
    /// the title bar has no working maximize button, and forcing the window to work-area size
    /// anyway would override a deliberate choice.
    #[test]
    fn a_window_without_a_maximize_box_falls_back() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            WS_CAPTION, WS_MAXIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
        };

        let ordinary = WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MAXIMIZEBOX;
        assert!(style_allows_maximize(ordinary));

        let fixed_dialog = WS_CAPTION | WS_SYSMENU;
        assert!(!style_allows_maximize(fixed_dialog));
    }

    /// A stale handle makes `GetWindowLongW` return zero, and zero must read as "fall back"
    /// rather than as a style word to act on.
    #[test]
    fn a_zero_style_word_falls_back() {
        assert!(!style_allows_maximize(0));
    }

    // --- partial failure ---------------------------------------

    #[test]
    fn all_valid_targets_are_applied() {
        let mut mover = FakeMover::new(vec![]);
        let plan = [placement(1, 0, 100), placement(2, 100, 200)];
        assert_eq!(apply_plan(&mut mover, &plan), (2, 0));
        assert_eq!(mover.calls().len(), 2);
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
        let attempted: Vec<WindowId> = mover.calls().iter().map(|p| p.window).collect();
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
        assert!(mover.calls().is_empty());
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
        let order: Vec<WindowId> = mover.calls().iter().map(|p| p.window).collect();
        assert_eq!(order, vec![WindowId(9), WindowId(4), WindowId(7)]);
    }

    // --- Invisible-border compensation --------------------------

    #[test]
    fn no_border_yields_zero_insets() {
        let outer = RECT {
            left: 100,
            top: 100,
            right: 900,
            bottom: 700,
        };
        // Extended frame identical to the outer rect: a window with no
        // invisible resize border at all.
        assert_eq!(
            FrameInsets::from_rects(outer, outer),
            FrameInsets::default()
        );
    }

    #[test]
    fn a_typical_left_right_bottom_border_is_measured_correctly() {
        // The common Windows 10+ shape: a few pixels of invisible border on
        // the left, right, and bottom, none on top.
        let outer = RECT {
            left: 100,
            top: 100,
            right: 900,
            bottom: 700,
        };
        let extended = RECT {
            left: 107,
            top: 100,
            right: 893,
            bottom: 693,
        };
        assert_eq!(
            FrameInsets::from_rects(outer, extended),
            FrameInsets {
                left: 7,
                top: 0,
                right: 7,
                bottom: 7,
            }
        );
    }

    #[test]
    fn an_inverted_extended_frame_clamps_to_zero_rather_than_going_negative() {
        // Not expected in practice, but Windows makes no promise the
        // extended frame is never larger than the outer rect — a negative
        // inset here would flip compensation into shrinking the target.
        let outer = RECT {
            left: 100,
            top: 100,
            right: 900,
            bottom: 700,
        };
        let extended = RECT {
            left: 90, // "outside" the outer rect
            top: 100,
            right: 910,
            bottom: 700,
        };
        assert_eq!(
            FrameInsets::from_rects(outer, extended),
            FrameInsets::default()
        );
    }

    #[test]
    fn compensation_widens_the_target_by_exactly_the_insets() {
        let target = Rect::new(0, 0, 1920, 1040).unwrap();
        let insets = FrameInsets {
            left: 7,
            top: 0,
            right: 7,
            bottom: 7,
        };
        let adjusted = compensate_for_frame_insets(target, insets).unwrap();
        assert_eq!(adjusted, Rect::new(-7, 0, 1927, 1047).unwrap());
    }

    #[test]
    fn zero_insets_leave_the_target_unchanged() {
        let target = Rect::new(0, 0, 1920, 1040).unwrap();
        let adjusted = compensate_for_frame_insets(target, FrameInsets::default()).unwrap();
        assert_eq!(adjusted, target);
    }

    // --- Clamping the compensated rect to the monitor's real pixels ----

    #[test]
    fn a_rect_already_inside_the_monitor_is_unchanged() {
        let monitor = RECT {
            left: -2160,
            top: -838,
            right: 0,
            bottom: 3002,
        };
        let rect = Rect::new(-2160, -838, -1069, 2929).unwrap();
        assert_eq!(clamp_to_monitor(rect, monitor), rect);
    }

    #[test]
    fn compensation_past_the_monitors_outer_edge_is_pulled_back_in() {
        // The exact shape reported for a portrait 4K monitor at 175%
        // scaling sitting at the leftmost edge of the virtual desktop,
        // where compensating outward by the border has nothing to bleed
        // into: the monitor itself is (-2160,-838,0,3002), and widening
        // the target by an 11px border pushes the left edge to -2171 —
        // 11px past the monitor's own left edge.
        let monitor = RECT {
            left: -2160,
            top: -838,
            right: 0,
            bottom: 3002,
        };
        let compensated = Rect::new(-2171, -838, -1069, 2929).unwrap();
        let clamped = clamp_to_monitor(compensated, monitor);
        assert_eq!(
            clamped.left, -2160,
            "must not bleed past the monitor's own left edge"
        );
        assert_eq!(
            clamped.right, -1069,
            "the untouched edge must be left alone"
        );
    }

    #[test]
    fn clamping_never_inverts_a_rect_that_still_fits_the_monitor() {
        let monitor = RECT {
            left: 0,
            top: 0,
            right: 3840,
            bottom: 2160,
        };
        // Compensated past every edge at once — still comfortably smaller
        // than the monitor, so clamping must produce a normal, non-inverted
        // rect rather than falling back to the unclamped input.
        let compensated = Rect::new(-20, -20, 3860, 2180).unwrap();
        let clamped = clamp_to_monitor(compensated, monitor);
        assert_eq!(clamped, Rect::new(0, 0, 3840, 2160).unwrap());
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

    // --- target ownership: basename comparison -----------------

    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn backslash_path_is_ours() {
        assert!(basename_matches_settings_exe(&utf16(
            r"C:\Program Files\Wira Desk\wiradesk-settings.exe"
        )));
    }

    #[test]
    fn forward_slash_path_is_ours() {
        assert!(basename_matches_settings_exe(&utf16(
            "C:/Program Files/Wira Desk/wiradesk-settings.exe"
        )));
    }

    #[test]
    fn bare_basename_with_no_separator_is_ours() {
        assert!(basename_matches_settings_exe(&utf16(
            "wiradesk-settings.exe"
        )));
    }

    #[test]
    fn uppercase_basename_is_ours() {
        assert!(basename_matches_settings_exe(&utf16(
            "WIRADESK-SETTINGS.EXE"
        )));
    }

    #[test]
    fn settings_helper_basename_is_not_ours() {
        assert!(!basename_matches_settings_exe(&utf16(
            "wiradesk-settings-helper.exe"
        )));
    }

    #[test]
    fn unrelated_basename_is_not_ours() {
        assert!(!basename_matches_settings_exe(&utf16("notepad.exe")));
    }

    #[test]
    fn empty_slice_is_not_ours() {
        assert!(!basename_matches_settings_exe(&[]));
    }

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
