//! Deciding whether an update should be offered.
//!
//! This module is deliberately the whole decision and none of the plumbing: it parses a
//! release descriptor, compares it against the running version, and validates the thing it
//! is being told to download. No sockets, no files, no Win32 — so every rule below is
//! reachable from a unit test, which is the point. The failures this layer prevents are
//! silent ones, and a silent failure that only a live network can produce is a failure
//! nobody tests.
//!
//! The layers on top of it, added separately: fetching `latest.json` over HTTPS, hashing a
//! download, verifying an Authenticode signature once a certificate exists, and launching
//! the installer detached.
//!
//! # What `latest.json` is
//! Published by the release workflow as a release asset under a name that never changes, so
//! `releases/latest/download/latest.json` is a permanently stable URL. One file is read by
//! both this updater and the download page on the website, so the two cannot disagree about
//! what the current version is.

use serde::Deserialize;

/// The repository this build belongs to, taken from `Cargo.toml`.
///
/// Read with `env!` rather than written out here, and not only to avoid a duplicate that
/// could drift. `scripts/verify-public-export.ps1` permits the maintainer's handle in exactly
/// two places, one of them a crate manifest's `repository` line — so deriving the value keeps
/// the fact in its declared home and keeps this file free of a second copy. The gate caught
/// the first version of this module for exactly that.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Split `https://host/path` into its host and path. `None` for anything else.
///
/// Hand-parsed and narrow on purpose: a permissive URL parser here would be a parser whose
/// edge cases decide where an executable is downloaded from.
fn split_https(url: &str) -> Option<(&str, &str)> {
    let rest = url.strip_prefix("https://")?;
    Some(match rest.split_once('/') {
        Some((host, path)) => (host, path),
        None => (rest, ""),
    })
}

/// A release, as described by `latest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    pub version: String,
    #[serde(default)]
    pub released: String,
    pub setup_url: String,
    pub setup_sha256: String,
    #[serde(default)]
    pub notes_url: String,
}

/// Why a release descriptor was refused.
///
/// Each variant is a distinct thing that can be wrong, rather than one catch-all error,
/// because the updater has to say something true to the user and "update failed" is not it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejected {
    /// The descriptor could not be parsed at all.
    Unreadable,
    /// A version string was not three numeric fields.
    BadVersion(String),
    /// `setup_sha256` was not 64 hexadecimal characters.
    BadChecksum,
    /// `setup_url` was not an HTTPS URL on the expected repository.
    BadUrl,
    // There is deliberately no "not newer" variant. `decide` answers that case with
    // `Decision::UpToDate`, which is not a refusal -- being current is the normal outcome,
    // not an error. A variant existed here at first and `-D warnings` caught it as never
    // constructed once the blanket dead-code allow came off, which is the lint doing exactly
    // the job it is set to `deny` for: a variant the logic cannot produce is a lie told by
    // the type, and every `match` on it grows an arm that can never run.
}

/// The outcome of asking whether to offer an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Nothing to do; the running version is current or ahead.
    UpToDate,
    /// A newer, validated release is available.
    Available(Release),
    /// The descriptor was refused. Nothing is downloaded.
    Refused(Rejected),
}

/// Three numeric fields, compared field by field.
///
/// A hand-rolled comparison rather than a semver crate: the only versions this ever sees are
/// produced by `[workspace.package] version` and written into `latest.json` by the release
/// workflow, both of which are three plain integers. Accepting more than that would mean
/// accepting version strings the product cannot itself produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u64, u64, u64);

fn parse_version(raw: &str) -> Option<Version> {
    let trimmed = raw.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.');
    let mut next = || -> Option<u64> {
        let field = parts.next()?;
        if field.is_empty() || !field.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        field.parse().ok()
    };
    let v = Version(next()?, next()?, next()?);
    // A fourth field means this is not the shape we produce, and guessing what the extra
    // number means is how a comparison quietly becomes wrong.
    if parts.next().is_some() {
        return None;
    }
    Some(v)
}

fn checksum_looks_like_sha256(raw: &str) -> bool {
    raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The one prefix a download URL may begin with: this repository's release downloads.
fn required_download_prefix() -> Option<String> {
    let (_, repo_path) = split_https(REPOSITORY)?;
    Some(format!(
        "/{}/releases/download/",
        repo_path.trim_matches('/')
    ))
}

/// `true` when the URL is HTTPS and points into this project's own release downloads.
///
/// **This is a guard, not tidiness.** `latest.json` arrives over HTTPS, so its contents are
/// as trustworthy as the connection — but `setup_url` inside it is a download instruction,
/// and a descriptor tampered with anywhere (a compromised release, a mistaken edit, a host
/// migration done wrong) could point it at a server of someone else's choosing. Pinning the
/// host and the repository path means the worst a bad descriptor achieves is failing, rather
/// than fetching an executable nobody vetted.
fn url_is_acceptable(raw: &str) -> bool {
    let Some((repo_host, _)) = split_https(REPOSITORY) else {
        return false;
    };
    let Some(prefix) = required_download_prefix() else {
        return false;
    };
    let Some((authority, path)) = split_https(raw) else {
        return false;
    };

    // An exact host match, never a suffix or prefix one: `evil-github.com` ends with the
    // right letters and `github.com.evil.tld` begins with them. Comparing the whole
    // authority also rejects userinfo, since `github.com@evil.tld` is not equal to the host
    // even though it reads like it starts with it, and rejects a port for the same reason.
    if authority != repo_host {
        return false;
    }

    let path = format!("/{path}");
    // Any dot-segment means this is not the shape the release workflow emits, so it is
    // refused outright rather than normalised — normalising is where path guards go wrong.
    if path.contains("/../") || path.ends_with("/..") {
        return false;
    }
    // Longer than the prefix, so the bare prefix with no file after it does not pass.
    path.starts_with(&prefix) && path.len() > prefix.len()
}

/// Validate a descriptor and decide against the running version.
///
/// `running` is normally `env!("CARGO_PKG_VERSION")`; it is a parameter so the decision is
/// testable without rebuilding at a different version.
pub fn decide(running: &str, descriptor: &str) -> Decision {
    let release: Release = match serde_json::from_str(descriptor) {
        Ok(r) => r,
        Err(_) => return Decision::Refused(Rejected::Unreadable),
    };

    let Some(running_v) = parse_version(running) else {
        return Decision::Refused(Rejected::BadVersion(running.to_owned()));
    };
    let Some(offered_v) = parse_version(&release.version) else {
        return Decision::Refused(Rejected::BadVersion(release.version.clone()));
    };

    if !checksum_looks_like_sha256(&release.setup_sha256) {
        return Decision::Refused(Rejected::BadChecksum);
    }
    if !url_is_acceptable(&release.setup_url) {
        return Decision::Refused(Rejected::BadUrl);
    }

    // STRICTLY greater, never merely different. Accepting a different version would accept
    // an older one, which is how a replayed descriptor pushes a build whose flaw is already
    // fixed — and the offered installer would be correctly signed and correctly hashed,
    // because it really is a release of this product. Ordering is the only thing that
    // catches it.
    if offered_v > running_v {
        Decision::Available(release)
    } else {
        Decision::UpToDate
    }
}

// ── Orchestration ───────────────────────────────────────────────────────────────────
//
// The three layers joined: decide, download, launch. Everything below runs on a worker
// thread and reports through a channel, following `hookbridge`'s shape rather than
// inventing a second one — the UI thread drains the receiver on a timer, so every model
// mutation stays on the thread that owns the model.

use std::sync::mpsc::{channel, Receiver, Sender};

use crate::https::HttpError;

/// Where the release descriptor lives. The filename never changes, so this URL is stable
/// across every version — no API, no rate limit, no HTML to parse.
pub fn latest_json_url() -> String {
    format!("{REPOSITORY}/releases/latest/download/latest.json")
}

/// Ceiling for the descriptor. It is a few hundred bytes; anything approaching this is
/// either not our file or not worth reading.
const DESCRIPTOR_LIMIT: u64 = 64 * 1024;

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
        HttpError::Hash(e) => format!("The checksum could not be computed ({}).", e.call),
    }
}

/// Ask once, on a worker thread.
pub fn spawn_check(running: String) -> Receiver<Progress> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let msg = match crate::https::get_text(&latest_json_url(), DESCRIPTOR_LIMIT) {
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

    let digest = match crate::https::download_to_file(&release.setup_url, &dest, INSTALLER_LIMIT) {
        Ok(d) => d,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Progress::Failed(describe_http(&e));
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

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    /// A URL of the shape the release workflow actually emits, built from the same
    /// `Cargo.toml` value the guard reads. Derived rather than written out for the reason
    /// given at `REPOSITORY`, and with the useful side effect that a repository move keeps
    /// these tests meaningful instead of quietly testing a stale address.
    fn good_url() -> String {
        format!("{REPOSITORY}/releases/download/v0.2.0/WiraDesk-0.2.0-x64-setup.exe")
    }

    /// The host part of the repository URL, for building deliberately wrong variants.
    fn repo_host() -> String {
        split_https(REPOSITORY).unwrap().0.to_owned()
    }

    fn repo_path() -> String {
        split_https(REPOSITORY)
            .unwrap()
            .1
            .trim_matches('/')
            .to_owned()
    }

    fn descriptor(version: &str, url: &str, sha: &str) -> String {
        format!(
            r#"{{"version":"{version}","released":"2026-09-01","setup_url":"{url}",
               "setup_sha256":"{sha}","notes_url":"https://example.invalid/notes"}}"#
        )
    }

    fn good(version: &str) -> String {
        descriptor(version, &good_url(), GOOD_SHA)
    }

    #[test]
    fn a_newer_version_is_offered() {
        match decide("0.1.0", &good("0.2.0")) {
            Decision::Available(r) => {
                assert_eq!(r.version, "0.2.0");
                assert_eq!(r.setup_sha256, GOOD_SHA);
            }
            other => panic!("expected Available, got {other:?}"),
        }
    }

    #[test]
    fn the_same_version_is_up_to_date() {
        assert_eq!(decide("0.1.0", &good("0.1.0")), Decision::UpToDate);
    }

    /// The replay case, and the reason the comparison is `>` and not `!=`. A descriptor
    /// naming an older release would offer an installer that is genuinely this product's,
    /// correctly hashed and one day correctly signed — so ordering is the only thing that
    /// refuses it.
    #[test]
    fn an_older_version_is_never_offered() {
        assert_eq!(decide("0.2.0", &good("0.1.9")), Decision::UpToDate);
        assert_eq!(decide("1.0.0", &good("0.9.9")), Decision::UpToDate);
    }

    #[test]
    fn version_fields_are_compared_numerically_not_as_text() {
        // "0.10.0" sorts before "0.9.0" as a string and after it as a version.
        assert!(matches!(
            decide("0.9.0", &good("0.10.0")),
            Decision::Available(_)
        ));
        assert_eq!(decide("0.10.0", &good("0.9.0")), Decision::UpToDate);
    }

    #[test]
    fn a_leading_v_is_tolerated_on_either_side() {
        assert!(matches!(
            decide("v0.1.0", &good("v0.2.0")),
            Decision::Available(_)
        ));
    }

    #[test]
    fn unparseable_json_is_refused() {
        assert_eq!(
            decide("0.1.0", "not json at all"),
            Decision::Refused(Rejected::Unreadable)
        );
        // A well-formed document missing a required field is equally unusable.
        assert_eq!(
            decide("0.1.0", r#"{"version":"0.2.0"}"#),
            Decision::Refused(Rejected::Unreadable)
        );
    }

    #[test]
    fn a_version_that_is_not_three_numbers_is_refused() {
        for bad in ["", "0.2", "0.2.0.1", "0.2.x", "latest", "0..0", "0.2.-1"] {
            assert!(
                matches!(
                    decide("0.1.0", &good(bad)),
                    Decision::Refused(Rejected::BadVersion(_))
                ),
                "{bad:?} should have been refused"
            );
        }
    }

    #[test]
    fn a_checksum_that_is_not_sha256_is_refused() {
        for bad in [
            "",
            "abc",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde", // 63
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0", // 65
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg", // non-hex
        ] {
            assert_eq!(
                decide("0.1.0", &descriptor("0.2.0", &good_url(), bad)),
                Decision::Refused(Rejected::BadChecksum),
                "{bad:?} should have been refused"
            );
        }
    }

    /// Each of these is a way a tampered descriptor could try to send the download
    /// somewhere else while still looking plausible in a log line.
    #[test]
    fn a_download_url_off_this_repository_is_refused() {
        let host = repo_host();
        let path = repo_path();
        let tail = "releases/download/v0.2.0/x.exe";

        let bad_urls = vec![
            // Not HTTPS.
            format!("http://{host}/{path}/{tail}"),
            // A different host that ends with the right letters.
            format!("https://evil-{host}/{path}/{tail}"),
            // A different host that begins with them.
            format!("https://{host}.evil.tld/{path}/{tail}"),
            // The real host hidden behind userinfo.
            format!("https://{host}@evil.tld/{path}/{tail}"),
            // The real host with a port, which the workflow never emits.
            format!("https://{host}:8443/{path}/{tail}"),
            // Right host, someone else's project.
            format!("https://{host}/someone/else/{tail}"),
            // Right host and project, but not a release download.
            format!("https://{host}/{path}/raw/main/x.exe"),
            // Dot segments.
            format!("https://{host}/{path}/releases/download/../../../x.exe"),
            // The prefix and nothing after it.
            format!("https://{host}/{path}/releases/download/"),
            // No scheme at all.
            format!("{host}/{path}/{tail}"),
            // Host only.
            format!("https://{host}"),
        ];

        for url in bad_urls {
            assert_eq!(
                decide("0.1.0", &descriptor("0.2.0", &url, GOOD_SHA)),
                Decision::Refused(Rejected::BadUrl),
                "{url:?} should have been refused"
            );
        }
    }

    #[test]
    fn the_real_workflow_url_shape_is_accepted() {
        assert!(
            url_is_acceptable(&good_url()),
            "the guard must accept {:?}, which is what the release workflow writes",
            good_url()
        );
    }

    /// Validation runs before the comparison, so a descriptor that is both tampered with
    /// and older is reported as tampered with. The reverse order would hide an attack
    /// behind a reassuring "you are up to date", and the log would say nothing happened.
    #[test]
    fn validation_precedes_the_version_comparison() {
        assert_eq!(
            decide(
                "0.9.0",
                &descriptor("0.1.0", "http://evil.tld/x.exe", GOOD_SHA)
            ),
            Decision::Refused(Rejected::BadUrl),
            "an older descriptor with a foreign URL must report the URL, not up-to-date"
        );
        assert_eq!(
            decide("0.9.0", &descriptor("0.1.0", &good_url(), "nonsense")),
            Decision::Refused(Rejected::BadChecksum)
        );
    }
}
