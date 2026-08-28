//! Right-click context menu for the System Tray icon.
//! Pure Win32 (`CreatePopupMenu` + `AppendMenuW` + `TrackPopupMenu`), no GUI
//! framework — keeps RAM footprint and binary size small. Native menus provide
//! keyboard navigation and free UI Automation exposure; `&` mnemonics are added
//! for tray keyboard access (outside Settings UI a11y scope).
//! Called from `wndproc_impl` (`WM_TRAYICON` arm) so it inherits the FFI
//! `catch_unwind` guard in `tray::wndproc`. This function only needs `hwnd`
//! (daemon host window) — it does not touch `TrayData` internals.

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::ptr::null;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, DestroyWindow, PostMessageW, SetForegroundWindow,
    TrackPopupMenu, MB_ICONINFORMATION, MB_OK, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
    SW_SHOWNORMAL, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_NULL,
};

use crate::autostart;
use crate::util::{debug_log, message_box, wide};

// ── Menu item command ids (TrackPopupMenu return values with TPM_RETURNCMD) ───
const CMD_SETTINGS: u32 = 1;
const CMD_VIEW_LOGS: u32 = 2;
const CMD_AUTOSTART: u32 = 3;
const CMD_ABOUT: u32 = 4;
const CMD_EXIT: u32 = 5;
const CMD_UPDATE: u32 = 6;

use shared::constants::SETTINGS_EXE_NAME;

/// Build and show the tray context menu at screen anchor `(x, y)`, then
/// run the selected item action. `x`/`y` come from the `NOTIFYICON_VERSION_4`
/// callback `wParam` (screen coordinates).
pub fn show(hwnd: HWND, x: i32, y: i32) {
    // SAFETY: `hwnd` is the daemon host window and this runs on the thread that owns it —
    // `show` is reachable only from the `WM_TRAYICON` arm of `wndproc_impl`.
    //
    // The `wide("...").as_ptr()` arguments look like dangling temporaries and are not:
    // `AppendMenuW` copies the label into the menu item, so the buffer only has to survive
    // the call, and a temporary lives to the end of the statement containing it. Were the
    // menu to retain the pointer instead, every label would be freed memory by the time the
    // menu was shown. For the two `MF_SEPARATOR` items `lpNewItem` is documented as ignored,
    // so passing null is correct rather than merely tolerated.
    //
    // `hmenu` is checked against 0 before use and destroyed exactly once. `TPM_RETURNCMD`
    // makes `TrackPopupMenu` return the chosen id instead of posting `WM_COMMAND`, so the
    // menu has served its purpose by the time it returns and `DestroyMenu` can run before
    // the action dispatch below — none of those actions touch `hmenu`.
    unsafe {
        let hmenu = CreatePopupMenu();
        if hmenu == 0 {
            debug_log("Wira Desk: CreatePopupMenu failed");
            return;
        }

        // Auto-Start checkmark follows Scheduler task presence, not
        // config.auto_start — authoritative source is schtasks /Query.
        let autostart_flag = if autostart::is_registered() {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };

        // An update, offered only when there is one.
        //
        // This is the item rather than "Check for updates", and the difference matters: a
        // menu closes on click, so a check started from one has nowhere to report to. An
        // announcement does not need anywhere to report -- it *is* the report -- and it
        // costs the menu nothing on the days there is nothing to say, which is almost all
        // of them.
        let update = crate::updatecheck::snapshot();
        if let Some(version) = update.available.as_deref() {
            AppendMenuW(
                hmenu,
                MF_STRING,
                CMD_UPDATE as usize,
                wide(&format!("&Update to {version}...")).as_ptr(),
            );
            AppendMenuW(hmenu, MF_SEPARATOR, 0, null());
        }

        // Group 1: Settings, View Logs, Auto-Start.
        AppendMenuW(
            hmenu,
            MF_STRING,
            CMD_SETTINGS as usize,
            wide("&Settings...").as_ptr(),
        );
        AppendMenuW(
            hmenu,
            MF_STRING,
            CMD_VIEW_LOGS as usize,
            wide("&View Logs").as_ptr(),
        );
        AppendMenuW(
            hmenu,
            MF_STRING | autostart_flag,
            CMD_AUTOSTART as usize,
            wide("&Auto-Start").as_ptr(),
        );
        AppendMenuW(hmenu, MF_SEPARATOR, 0, null());
        // Group 2: About.
        AppendMenuW(
            hmenu,
            MF_STRING,
            CMD_ABOUT as usize,
            wide("A&bout").as_ptr(),
        );
        AppendMenuW(hmenu, MF_SEPARATOR, 0, null());
        // Group 3: Exit.
        AppendMenuW(hmenu, MF_STRING, CMD_EXIT as usize, wide("E&xit").as_ptr());

        // Foreground window is required before TrackPopupMenu so the menu receives
        // focus; PostMessage(WM_NULL) afterward (MSDN KB135788) so the menu
        // dismisses on the first outside click on tool/hidden windows.
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            hmenu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD,
            x,
            y,
            0,
            hwnd,
            null(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(hmenu);

        // cmd == 0 → menu dismissed without a selection (or error); no-op.
        match cmd as u32 {
            // Same destination as Settings. The item names the update because that is why
            // the user is clicking, but the place to read what changed and press Install is
            // the About pane, which is where Settings opens anyway.
            CMD_SETTINGS | CMD_UPDATE => launch_settings(),
            CMD_VIEW_LOGS => view_logs(),
            CMD_AUTOSTART => toggle_autostart(hwnd),
            CMD_ABOUT => show_about(hwnd),
            CMD_EXIT => request_exit(hwnd),
            _ => {}
        }
    }
}

/// Symmetric teardown. `DestroyWindow` → `WM_DESTROY` → `cleanup`
/// (`NIM_DELETE` + `DestroyIcon`×3 + `UnhookWindowsHookEx`) → `PostQuitMessage`.
/// Do NOT call `PostQuitMessage(0)` directly — `TrayData` has no `Drop`, so the
/// direct path leaks GDI icons and the hook.
fn request_exit(hwnd: HWND) {
    // SAFETY: `DestroyWindow` may only be called from the thread that created the window,
    // and that is satisfied structurally rather than by convention — this is reached from
    // `menu::show`, itself reached only from `wndproc_impl`, which Windows invokes on the
    // owning thread. Calling it from the hook or heartbeat thread would fail instead of
    // tearing the window down, which is why exit is routed through a posted menu command.
    if unsafe { DestroyWindow(hwnd) } == 0 {
        debug_log("Wira Desk: DestroyWindow (Exit) failed");
    }
}

/// Launch `wiradesk-settings.exe` via `ShellExecuteW` (inherits Admin elevation,
/// separate/decoupled process). Path is resolved relative to the running daemon exe.
pub fn launch_settings() {
    launch_settings_with(None);
}

/// Launch Settings in onboarding mode on first run.
/// The argument is the frozen `ONBOARDING_FLAG`, passed through as-is instead of
/// a string literal so both sides of the contract cannot diverge without a compile error.
pub fn launch_onboarding() {
    launch_settings_with(Some(shared::ONBOARDING_FLAG));
}

fn launch_settings_with(args: Option<&str>) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            debug_log("Wira Desk: launch_settings — current_exe() failed");
            return;
        }
    };
    let settings_path = match exe.parent() {
        Some(dir) => dir.join(SETTINGS_EXE_NAME),
        None => {
            debug_log("Wira Desk: launch_settings — no parent dir for daemon exe");
            return;
        }
    };

    let verb = wide("open");
    let path_w = wide(&settings_path.to_string_lossy());
    let args_w = args.map(wide);
    let args_ptr = args_w.as_ref().map_or(null(), |a| a.as_ptr());
    // SAFETY: `verb`, `path_w`, and `args_w` are all NUL-terminated (`wide`) locals that
    // outlive the call. `args_w` is bound to a named local for exactly this reason: writing
    // `args.map(wide).map(|a| a.as_ptr())` would drop the buffer at the end of that
    // expression and leave `args_ptr` dangling before `ShellExecuteW` ever read it. A null
    // `args_ptr` (no onboarding flag) and null `lpDirectory` are both documented as
    // "no argument", not as pointers to be dereferenced.
    let result = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            path_w.as_ptr(),
            args_ptr,
            null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns HINSTANCE; values <= 32 indicate failure.
    if result as isize <= 32 {
        debug_log("Wira Desk: launch_settings — ShellExecuteW failed");
    }
}

/// Open the log file (`shared::log_path`) in the user's default text editor.
/// `.log` files often lack an "open" handler that is an editor, so launch
/// `notepad.exe` directly (built-in OS text editor). Create an empty file first
/// when it does not exist yet.
fn view_logs() {
    let path = shared::log_path();

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::File::create(&path).is_err() {
            debug_log("Wira Desk: view_logs — failed to create empty log file");
            // Continue trying to open; Notepad will offer to create the file.
        }
    }

    let spawned = Command::new("notepad.exe")
        .arg(&path)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    if spawned.is_err() {
        debug_log("Wira Desk: view_logs — failed to spawn notepad.exe");
    }
}

/// Toggle auto-start task registration. The checkmark when the menu opens
/// reflects `is_registered`, so selecting the item flips the state.
///
/// Turning it **on** is the moment the user takes on the risk of an unprompted
/// elevated logon task, so that is where the location is checked — at the decision,
/// rather than only at the next start. The check runs after registration and does
/// not gate it: the warning reports what was done, it does not veto it.
fn toggle_autostart(hwnd: HWND) {
    if autostart::is_registered() {
        autostart::disable();
    } else if autostart::enable() {
        autostart::warn_if_location_replaceable(hwnd);
    }
}

/// About → version info.
fn show_about(hwnd: HWND) {
    let text = format!(
        "Wira Desk v{}\n\nSame-application window switcher and window arrangement for Windows 10/11.\n\nRuns quietly in the background as an elevated system tray service.",
        env!("CARGO_PKG_VERSION")
    );
    message_box(hwnd, &text, "About Wira Desk", MB_OK | MB_ICONINFORMATION);
}

#[cfg(test)]
mod tests {
    use shared::constants::{SETTINGS_BIN_NAME, SETTINGS_EXE_NAME};

    #[test]
    fn settings_exe_matches_cargo_bin_name() {
        assert_eq!(
            SETTINGS_EXE_NAME,
            format!("{SETTINGS_BIN_NAME}.exe"),
            "SETTINGS_EXE_NAME must stay identical to settings [[bin]] name"
        );
    }
}
