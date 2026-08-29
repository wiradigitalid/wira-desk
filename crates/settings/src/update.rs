//! Driving an update: ask, download, verify, launch.
//!
//! The decision itself lives in `shared::update`, because the daemon asks the same question
//! on its daily check. What is here is everything that only a user-facing, non-elevated
//! process should do -- fetching an executable, hashing it, and handing it to Windows.

use std::sync::mpsc::{channel, Receiver, Sender};

use shared::https::HttpError;
use shared::update::{
    decide, latest_json_url, split_https, Decision, Rejected, Release, DESCRIPTOR_LIMIT, REPOSITORY,
};

use crate::sha256::{CngError, Sha256, DIGEST_LEN};

/// Anything that stopped an install, as one type. `Http` and `Hash` are separate because a
/// network failure and a broken crypto provider need different words to a user.
#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallError {
    Http(HttpError),
    Hash(CngError),
}

impl From<HttpError> for InstallError {
    fn from(e: HttpError) -> Self {
        InstallError::Http(e)
    }
}

// ── Orchestration ───────────────────────────────────────────────────────────────────
//
// The three layers joined: decide, download, launch. Everything below runs on a worker
// thread and reports through a channel, following `hookbridge`'s shape rather than
// inventing a second one — the UI thread drains the receiver on a timer, so every model
// mutation stays on the thread that owns the model.

/// Ceiling for the installer. The real one is around 8 MB, and this leaves generous room
/// for it to grow without leaving the download unbounded.
const INSTALLER_LIMIT: u64 = 192 * 1024 * 1024;

/// What the worker reports back. One message per state the UI can be in, so the UI never
/// has to infer anything from a combination of flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// The check finished and there is nothing newer.
    UpToDate,
    /// A newer, validated release exists.
    Available(Release),
    /// The installer is downloading. Bytes so far, and the total if the server said.
    Downloading { received: u64 },
    /// The installer was verified and handed to Windows. Settings should now exit.
    Launched,
    /// Something went wrong, in words a person can act on.
    Failed(String),
}

/// Human wording for a refusal. Deliberately specific: "update failed" tells a user
/// nothing, and the difference between "you are offline" and "the file we were offered was
/// not ours" is the difference between waiting and worrying.
fn describe(rejected: &Rejected) -> String {
    match rejected {
        Rejected::Unreadable => {
            "The update information could not be read. It may be a temporary problem with \
             the download server."
                .to_owned()
        }
        Rejected::BadVersion(v) => {
            format!("The update information named an unusable version ({v}).")
        }
        Rejected::BadChecksum => {
            "The update information carried a malformed checksum, so nothing was \
             downloaded."
                .to_owned()
        }
        Rejected::BadUrl => {
            "The update information pointed somewhere other than this product's own \
             releases, so nothing was downloaded."
                .to_owned()
        }
    }
}

fn describe_http(err: &HttpError) -> String {
    match err {
        HttpError::NotHttps => {
            "The download address was not a secure one, so nothing was fetched.".to_owned()
        }
        HttpError::Win32 { call, code } => {
            format!(
                "The connection failed ({call}, code {code}). Check your network and try again."
            )
        }
        HttpError::Status(code) => {
            format!("The download server answered with status {code}.")
        }
        HttpError::TooLarge { limit } => {
            format!("The download was larger than the {limit}-byte limit and was stopped.")
        }
        HttpError::NotUtf8 => "The update information was not readable text.".to_owned(),
    }
}

/// Words for a failure that stopped an install. HTTP failures defer to `describe_http`;
/// a crypto failure gets its own sentence, because "the download failed" would be a lie
/// about a working download and a broken provider.
fn describe_install(err: &InstallError) -> String {
    match err {
        InstallError::Http(e) => describe_http(e),
        InstallError::Hash(e) => format!(
            "The checksum could not be computed ({}), so the download was not trusted and              was deleted.",
            e.call
        ),
    }
}

/// Ask once, on a worker thread.
pub fn spawn_check(running: String) -> Receiver<Progress> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let msg = match shared::https::get_text(&latest_json_url(), DESCRIPTOR_LIMIT) {
            Err(e) => Progress::Failed(describe_http(&e)),
            Ok(body) => match decide(&running, &body) {
                Decision::UpToDate => Progress::UpToDate,
                Decision::Available(release) => Progress::Available(release),
                Decision::Refused(reason) => Progress::Failed(describe(&reason)),
            },
        };
        let _ = tx.send(msg);
    });
    rx
}

/// Where a downloaded installer is staged.
///
/// A fresh randomly named directory under the user's temp, created for this download and
/// nothing else. The name is unpredictable and the directory is new, so nothing can be
/// waiting there under the name we are about to write.
///
/// **The residual risk is stated rather than hidden.** This directory is writable by the
/// user, so between the moment the digest is verified and the moment Windows starts the
/// file, another process running as that same user could replace it. The window is small
/// and closing it properly needs the Authenticode check that arrives with the certificate —
/// at which point the file is verified by its signature immediately before launch, and a
/// swap is caught. Until then this is the honest state of it, and it is no worse than a
/// user downloading the installer themselves and double-clicking it.
fn staging_dir() -> Result<std::path::PathBuf, String> {
    // Not random for secrecy; random so the path cannot be predicted and pre-created by
    // something else. Two sources, because either alone repeats too easily: the process id,
    // and the clock.
    let nonce = format!(
        "{:x}{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let dir = std::env::temp_dir().join(format!("WiraDesk-update-{nonce}"));
    std::fs::create_dir(&dir)
        .map_err(|e| format!("A temporary folder for the download could not be created ({e})."))?;
    Ok(dir)
}

/// Download, verify, and hand the installer to Windows.
///
/// Runs on a worker thread and reports progress. On success the last message is
/// [`Progress::Launched`], after which Settings must exit — the installer stops the daemon
/// and replaces `wiradesk-settings.exe`, and Windows cannot replace a running image. That
/// contract is written at the matching place in `packaging/wiradesk.iss`.
pub fn spawn_install(release: Release) -> Receiver<Progress> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let msg = install_now(&release, &tx);
        let _ = tx.send(msg);
    });
    rx
}

fn install_now(release: &Release, tx: &Sender<Progress>) -> Progress {
    let dir = match staging_dir() {
        Ok(d) => d,
        Err(e) => return Progress::Failed(e),
    };
    let dest = dir.join("WiraDesk-setup.exe");

    // A coarse heartbeat rather than a byte count from the sink: the download layer reports
    // the total when it finishes, and a progress bar that only moves at the end is worse
    // than one that says "working". Sent once so the UI can show activity.
    let _ = tx.send(Progress::Downloading { received: 0 });

    let digest = match download_to_file(&release.setup_url, &dest, INSTALLER_LIMIT) {
        Ok(d) => d,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Progress::Failed(describe_install(&e));
        }
    };

    // THE GATE. Until a certificate exists this is the only thing standing between a
    // downloaded file and an elevated installer running, so a mismatch destroys the file
    // rather than reporting and leaving it.
    if !crate::sha256::matches_hex(&digest, &release.setup_sha256) {
        let _ = std::fs::remove_dir_all(&dir);
        return Progress::Failed(
            "The downloaded file did not match the checksum published for it, so it was \
             deleted and nothing was run. This usually means the download was corrupted; if \
             it happens again, download the installer from the releases page instead."
                .to_owned(),
        );
    }

    match launch_installer(&dest) {
        Ok(()) => Progress::Launched,
        Err(e) => {
            // The file is left in place deliberately on a launch failure. It is verified, and
            // the most likely cause is a policy that blocked the launch rather than anything
            // wrong with the file — so telling the user where it is lets them run it
            // themselves, which is a better answer than deleting their download.
            Progress::Failed(format!(
                "{e} The verified installer is at {}, and can be run directly.",
                dest.display()
            ))
        }
    }
}

/// Hand the installer to the shell.
///
/// **`ShellExecuteW`, not `CreateProcess`, and this is not a style choice.** The installer's
/// manifest requests administrator, and `CreateProcess` cannot elevate — it fails with
/// `ERROR_ELEVATION_REQUIRED` (740). Only the shell raises the consent prompt. That exact
/// mistake is what made the installer's own finish-page checkbox fail on a real machine, and
/// repeating it here would produce an update that reports success and does nothing.
///
/// `/SILENT` rather than `/VERYSILENT`: progress stays visible, questions the user already
/// answered are not asked again. The launch is detached — the shell does not wait — and the
/// caller exits immediately after, because the installer replaces `wiradesk-settings.exe`
/// and Windows cannot replace a running image.
fn launch_installer(path: &std::path::Path) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let file = wide(&path.to_string_lossy());
    let verb = wide("open");
    let args = wide("/SILENT");

    // SAFETY: `file`, `verb`, and `args` are NUL-terminated wide strings held in locals that
    // outlive the call — the previous version of this bound them inside the argument list,
    // where they would have been dropped before `ShellExecuteW` read them. A null owner
    // window is documented for a caller with no window to parent the consent prompt to, and
    // a null directory means the shell picks one, which is wanted here: the installer must
    // not inherit a working directory that could influence its DLL search.
    let result = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            file.as_ptr(),
            args.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW returns an HINSTANCE-shaped value; anything at or below 32 is an error
    // code rather than a handle. 1223 is ERROR_CANCELLED, which here means the user declined
    // the consent prompt — not a fault, and it must not read like one.
    let code = result as isize;
    if code > 32 {
        return Ok(());
    }
    if code == 5 || code == 1223 {
        return Err("The update needs your permission to install, and it was declined.".to_owned());
    }
    Err(format!(
        "Windows refused to start the installer (code {code}). An application-control policy \
         such as Smart App Control can do this to a program that is not yet signed."
    ))
}

/// Open a release-notes URL in the user's browser.
///
/// **Validated before it is handed to the shell, and by the same rule as the installer
/// download.** `notes_url` arrives in the same descriptor as `setup_url`, so it deserves the
/// same suspicion: a tampered file could otherwise use this to open any address it liked, in
/// a browser, at a moment the user is expecting a page from this project. The host and
/// repository are pinned; anything else is silently not opened, because a failed link is a
/// smaller harm than a link somewhere else.
pub fn open_in_browser(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let Some((repo_host, _)) = split_https(REPOSITORY) else {
        return;
    };
    let Some((host, _)) = split_https(url) else {
        return;
    };
    if host != repo_host || !url.starts_with(REPOSITORY) {
        return;
    }

    let target = wide(url);
    let verb = wide("open");
    // SAFETY: `target` and `verb` are NUL-terminated wide strings in locals that outlive the
    // call. A null owner window is documented for a caller with no window to parent to, and
    // null parameters and directory are the documented "nothing to add" values. The result is
    // discarded: a browser that fails to open is not something this can act on, and the URL
    // is also shown in the release itself.
    unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Download to a file, returning the SHA-256 of exactly the bytes written.
///
/// The digest is computed **as the bytes are written**, not by reading the file back. Two
/// reasons, and the second is the one that matters: reading back would hash whatever is on
/// disk at that later moment rather than what arrived, and it would mean a complete
/// unverified installer existed on disk with nothing yet saying it was the right one.
///
/// On any failure the partial file is removed. A half-downloaded installer left behind is a
/// file a later run — or a user — could mistake for a whole one.
fn download_to_file(
    url: &str,
    dest: &std::path::Path,
    limit: u64,
) -> Result<[u8; DIGEST_LEN], InstallError> {
    use std::io::Write;

    fn io_failure(call: &'static str) -> InstallError {
        InstallError::Http(HttpError::Win32 { call, code: 0 })
    }

    let mut hasher = Sha256::new().map_err(InstallError::Hash)?;
    let mut file = std::fs::File::create(dest).map_err(|_| io_failure("File::create"))?;

    let outcome = shared::https::get_streaming(url, limit, |chunk| {
        hasher.update(chunk).map_err(InstallError::Hash)?;
        file.write_all(chunk)
            .map_err(|_| io_failure("File::write_all"))
    });

    let flushed = file.flush().is_ok();
    drop(file);

    if outcome.is_err() || !flushed {
        let _ = std::fs::remove_file(dest);
        return Err(outcome.err().unwrap_or_else(|| io_failure("File::flush")));
    }

    match hasher.finish() {
        Ok(digest) => Ok(digest),
        Err(e) => {
            let _ = std::fs::remove_file(dest);
            Err(InstallError::Hash(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live smoke test for the download-and-verify half of the update pipeline, run against a
    /// real asset in this project's own releases -- the same repository-pinned shape
    /// `url_is_acceptable` in `shared::update` requires, so nothing hosted elsewhere could
    /// reach this code from a real check either. Ignored by default: it downloads several
    /// megabytes over a real connection. Deliberately stops short of `install_now` and
    /// `launch_installer` -- the one remaining step needs a human at a UAC prompt, so this
    /// cannot install or launch anything; it proves only "the bytes matched the checksum".
    ///
    /// `WIRADESK_TEST_SETUP_URL=<url> WIRADESK_TEST_SETUP_SHA256=<sha> cargo test -p settings -- --ignored the_real_installer_downloads_and_verifies`
    #[test]
    #[ignore]
    fn the_real_installer_downloads_and_verifies() {
        let url = std::env::var("WIRADESK_TEST_SETUP_URL")
            .expect("set WIRADESK_TEST_SETUP_URL to a real releases/download asset first");
        let sha = std::env::var("WIRADESK_TEST_SETUP_SHA256")
            .expect("set WIRADESK_TEST_SETUP_SHA256 to that asset's real SHA-256 first");

        let dir = staging_dir().expect("staging dir");
        let dest = dir.join("probe.exe");
        let digest = download_to_file(&url, &dest, INSTALLER_LIMIT).expect("download failed");
        let matches = crate::sha256::matches_hex(&digest, &sha);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            matches,
            "downloaded bytes did not match the published checksum"
        );
    }
}
