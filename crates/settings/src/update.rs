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

// Nothing calls `decide` yet: this is the decision layer, and the fetch, hash, verify, and
// launch layers land on top of it separately. The allow is scoped to this module and is
// expected to go when the caller arrives; wiring a half-built updater into the UI just to
// satisfy a lint would be the worse trade.
#![allow(dead_code)]

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
    /// The offered version is not newer than the running one.
    NotNewer { running: String, offered: String },
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
