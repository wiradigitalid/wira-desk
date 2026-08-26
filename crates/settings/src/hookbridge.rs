//! Hidden receiver window for the daemon's `WM_APP_RECORDED_CHORD` reports
//! (`DEC-004`'s observe/record lease).
//!
//! Kept as its own tiny top-level window on a dedicated thread, separate from
//! the Slint-owned window, for one reason: the daemon locates it with a plain
//! `FindWindowW` by class and title — the same way Settings already locates
//! the daemon's hidden window (`persistence::signal_reload`) — so nothing
//! here has to reach into winit's window handle or subclass a window owned by
//! a UI toolkit this crate does not control.
//!
//! The window's only job is to hand received chords to the caller through a
//! channel. Nothing here touches the Slint model or event loop directly —
//! `main.rs` drains the receiver on a UI-thread timer instead, which is what
//! keeps every model mutation on the thread that already owns it.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::OnceLock;

use shared::constants::{
    SETTINGS_HOOK_WINDOW_CLASS, SETTINGS_HOOK_WINDOW_TITLE, WM_APP_RECORDED_CHORD,
};

use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, MSG, WM_DESTROY, WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A chord the daemon's hook actually observed. Carries the raw pieces the
/// wire message carries, unparsed — the caller decides what an
/// unrepresentable virtual-key code means; this module only relays what
/// arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordedChord {
    pub vk: u16,
    pub ctrl: bool,
    pub win: bool,
    pub alt: bool,
    pub shift: bool,
}

fn unpack(wparam: WPARAM, lparam: LPARAM) -> RecordedChord {
    let bits = lparam as u32;
    RecordedChord {
        vk: wparam as u16,
        ctrl: bits & 1 != 0,
        win: bits & 2 != 0,
        alt: bits & 4 != 0,
        shift: bits & 8 != 0,
    }
}

/// Set exactly once, before the receiver window is created, and read from
/// `wndproc` for the life of the process. A `OnceLock` rather than passing the
/// sender through `GWLP_USERDATA`/`WM_NCCREATE` — this window never needs
/// per-instance state, only this one channel endpoint.
static SENDER: OnceLock<Sender<RecordedChord>> = OnceLock::new();

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_APP_RECORDED_CHORD {
        if let Some(tx) = SENDER.get() {
            let _ = tx.send(unpack(wparam, lparam));
        }
        return 0;
    }
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return 0;
    }
    // SAFETY: standard default handling for every message this window does
    // not itself interpret; `hwnd`/`msg`/`wparam`/`lparam` are forwarded
    // exactly as Windows delivered them.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Create the hidden window and run its message loop until `WM_DESTROY`.
/// Never returns on the happy path — the caller runs this on a dedicated
/// background thread.
fn run_message_loop() {
    // SAFETY: this function owns the window's whole lifetime, so the
    // obligations are stated once here. `class_name`/`title` are locals that
    // outlive every call taking them; Windows copies the class name into its
    // atom table during `RegisterClassW`, so it need not outlive the window
    // itself. `zeroed` is valid for `WNDCLASSW` and `MSG`: both are plain C
    // structs of integers and nullable pointers, including `lpfnWndProc`, an
    // `Option<unsafe extern fn>` for which all-zero is exactly `None` and
    // which is overwritten with `Some(wndproc)` regardless. This window is
    // never shown (`WS_VISIBLE` is not set; `WS_EX_TOOLWINDOW` prevents
    // Alt-Tab even if some future code path accidentally shows it), and it
    // must be a real top-level window rather than message-only so a plain
    // `FindWindowW` from the daemon's process can find it — the same
    // constraint the daemon's own hidden window documents.
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(std::ptr::null());
        let class_name = wide(SETTINGS_HOOK_WINDOW_CLASS);

        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(wndproc);
        wc.hInstance = hinstance;
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassW(&wc);

        let title = wide(SETTINGS_HOOK_WINDOW_TITLE);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            0,
            0,
            hinstance,
            std::ptr::null(),
        );
        if hwnd == 0 {
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let r = GetMessageW(&mut msg, 0, 0, 0);
            if r == 0 || r == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Start the receiver window on a dedicated background thread and return the
/// channel end that yields chords as the daemon reports them. Idempotent to
/// call more than once is not guaranteed — call exactly once, at startup.
pub fn spawn() -> Receiver<RecordedChord> {
    let (tx, rx) = channel();
    // `OnceLock::set` cannot fail on the very first call, which this is.
    let _ = SENDER.set(tx);
    std::thread::spawn(run_message_loop);
    rx
}
