#![windows_subsystem = "windows"]

mod arrangement;
mod autostart;
mod config;
mod context;
mod cycling;
mod error;
mod health;
mod hook;
mod icon;
mod legacy;
mod log;
mod menu;
#[cfg(debug_assertions)]
mod metrics;
mod ring;
mod tray;
mod util;
mod worker;

use std::mem;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, FALSE, HANDLE,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW;
use windows_sys::Win32::System::Threading::{
    CreateMutexW, ExitProcess, GetCurrentProcess, OpenProcessToken,
};

use shared::constants::SINGLE_INSTANCE_MUTEX;
use shared::migrate_appdata;

use crate::util::wide;

const DLL_SEARCH_EXCLUDE: &[u16] = &[0];

fn is_elevated() -> bool {
    let mut token: HANDLE = 0;
    // SAFETY: `GetCurrentProcess` returns a pseudo-handle that is always valid and must
    // never be closed, so only `token` needs releasing. `token` is a live local, written
    // only when `OpenProcessToken` reports success; the early return leaves it untouched
    // at `0` and skips the `CloseHandle`, so the handle is closed exactly once on exactly
    // the path that opened it. `elevation` is a `TOKEN_ELEVATION` — a single `u32` field,
    // for which the all-zero bit pattern is a valid value — so `mem::zeroed` initialises
    // it fully before the call. The `*mut _ as *mut _` cast erases the type, which makes
    // `size` the only thing telling `GetTokenInformation` how much it may write: it is
    // `size_of::<TOKEN_ELEVATION>()`, matching the class `TokenElevation` requests, so
    // the write stays inside `elevation`.
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = mem::zeroed();
        let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );
        CloseHandle(token);
        if result == FALSE {
            return false;
        }
        elevation.TokenIsElevated != 0
    }
}

fn main() {
    // SAFETY: `DLL_SEARCH_EXCLUDE` is a `'static` slice holding a single NUL, i.e. the
    // empty wide string, which is the documented argument for removing the current
    // directory from the DLL search order. Being `'static`, the pointer outlives the
    // call. This runs before anything else in `main` on purpose: the daemon is elevated,
    // so a planted DLL in the working directory would load with those privileges.
    unsafe {
        if SetDllDirectoryW(DLL_SEARCH_EXCLUDE.as_ptr()) == FALSE {
            ExitProcess(1);
        }
    }

    if !is_elevated() {
        error::fatal(
            "Wira Desk must be run as Administrator.\n\nWithout it, Windows blocks the app from activating or moving windows that belong to elevated programs, so switching would silently fail on some of them.\n\nThe application will now close.",
        );
    }

    migrate_appdata();

    let mutex_name = wide(SINGLE_INSTANCE_MUTEX);
    // SAFETY: `mutex_name` comes from `wide`, so it is NUL-terminated, and it is a local
    // that outlives this block — the pointer stays valid for the whole call. A null
    // `lpSecurityAttributes` is the documented request for the default descriptor.
    // Both `GetLastError` reads are correct because nothing between them can clobber the
    // thread's last-error value: only an integer comparison separates `CreateMutexW` from
    // the first read, and the success path reaches the second read with no intervening
    // Win32 call. `CloseHandle` runs exactly once, on the already-exists path, before
    // exiting. The surviving handle is deliberately never closed: `_mutex` holds it for
    // the process lifetime so the single-instance claim cannot lapse while we run, and
    // `ExitProcess` releases it.
    let _mutex = unsafe {
        let handle = CreateMutexW(std::ptr::null(), FALSE, mutex_name.as_ptr());
        if handle == 0 {
            let err = GetLastError();
            if err == ERROR_ACCESS_DENIED {
                ExitProcess(0);
            }
            ExitProcess(1);
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            ExitProcess(0);
        }
        handle
    };

    legacy::stop_legacy_daemon();
    legacy::migrate_scheduled_task();

    let exit_code = tray::run_message_loop();

    // SAFETY: `ExitProcess` terminates every other thread abruptly, without unwinding or
    // running their destructors, so it is only sound where no thread still holds state
    // that must be flushed. That holds here: `run_message_loop` has already returned,
    // which means the window is destroyed and `TrayData` reclaimed, and the Tier-2 logger
    // opens and closes `wiradesk.log` per line rather than holding a buffered handle.
    unsafe {
        ExitProcess(exit_code as u32);
    }
}

#[cfg(test)]
mod tests {
    /// The build script reports what it did with the resource script, and this is where
    /// that report is consumed. `env!` rather than `option_env!` on purpose: if a future
    /// edit adds a path through `build.rs` that returns without embedding the resource
    /// *and* without saying so, this crate stops compiling. That is the whole point —
    /// the failure being guarded against is a silent one, so the guard cannot be a
    /// runtime check that a silent failure would also skip.
    ///
    /// This does not assert the manifest is present in *this* binary, and cannot: the
    /// test harness is built with `WIRADESK_SKIP_MANIFEST` set, so during `cargo test`
    /// the honest answer is `skipped`. The guarantee for a real build comes from
    /// `build.rs` panicking, which fails the build rather than a test.
    #[test]
    fn build_script_reports_what_it_did_with_the_resource() {
        const STATE: &str = env!("WIRADESK_RESOURCE_STATE");
        assert!(
            matches!(STATE, "embedded" | "skipped" | "not-windows"),
            "unrecognised resource state {STATE:?} - build.rs reported something this test \
             does not know how to interpret, which means one of them is out of date"
        );

        // On Windows there are exactly two honest outcomes, and `embedded` is the one a
        // shipped binary must have. `skipped` is only reachable through the explicit
        // opt-out, so seeing it here confirms the harness took that route rather than
        // the resource silently failing to compile.
        #[cfg(windows)]
        assert_ne!(
            STATE, "not-windows",
            "build.rs decided this is not a Windows target while the crate is compiled \
             for Windows - the target detection in build.rs disagrees with cfg(windows)"
        );
    }

    /// A resource script cannot read `Cargo.toml`, so the version is written in both
    /// places. This is what keeps the duplicate honest: an edit that updates one and
    /// forgets the other fails here, instead of shipping a binary whose properties
    /// dialog and UAC prompt report a version the product is not.
    #[test]
    fn version_resource_matches_cargo_manifest() {
        let rc = include_str!("../wiradesk.rc");
        let version = env!("CARGO_PKG_VERSION");

        // Win32 wants a four-field comma form; `Cargo.toml` carries three fields.
        let mut fields: Vec<&str> = version.split('.').collect();
        while fields.len() < 4 {
            fields.push("0");
        }
        let comma = fields.join(",");

        for expected in [
            format!("FILEVERSION {comma}"),
            format!("PRODUCTVERSION {comma}"),
            format!("VALUE \"FileVersion\", \"{version}\""),
            format!("VALUE \"ProductVersion\", \"{version}\""),
        ] {
            assert!(
                rc.contains(&expected),
                "wiradesk.rc is missing {expected:?} - it drifted from Cargo.toml version {version}"
            );
        }
    }
}
