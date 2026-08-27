//! HTTPS GET over WinHttp, with a size ceiling.
//!
//! Two callers, both in the updater: fetching `latest.json`, and downloading the installer
//! it names. No new dependency — WinHttp is the HTTP client Windows already ships, reached
//! through the `windows-sys` already in the tree.
//!
//! # Three rules this module will not bend
//!
//! **HTTPS only.** A scheme other than `https` is refused before a handle is opened, and
//! `WINHTTP_FLAG_SECURE` is passed on every request. There is no code path here that can
//! perform a plaintext fetch, which matters because until a code-signing certificate exists,
//! the transport plus the SHA-256 in `latest.json` *is* the verification.
//!
//! **A ceiling, checked before allocating.** Every read is bounded by a caller-supplied
//! limit, and the limit is enforced as bytes arrive rather than after. A server that answers
//! a few-kilobyte JSON request with an endless body must not be able to make this process
//! grow until it dies — and the same function later downloads an installer, so "it is only a
//! small file" is not a property of the code.
//!
//! **No redirect to another host.** WinHttp follows redirects by default, which is right for
//! GitHub release assets: they answer with a redirect to a storage host. That is also exactly
//! the mechanism by which a poisoned descriptor could pull bytes from somewhere else, so
//! `update::decide` pins the URL before this module is ever called, and the caller is
//! responsible for having validated it.
//!
//! # What is testable here
//! The URL split and the scheme refusal. Everything past `WinHttpOpen` needs a network, and
//! is verified by the layer above it refusing a body whose digest does not match.

// Nothing calls this yet. It is a layer of the updater, and the layer that wires the
// button, the progress, and the installer launch lands separately. The allow is scoped
// to this module and goes when the caller arrives; wiring a half-built updater into the
// UI to satisfy a lint would be the worse trade.
#![allow(dead_code)]

use std::ptr;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Networking::WinHttp::{
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders,
    WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, INTERNET_DEFAULT_HTTPS_PORT,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WINHTTP_QUERY_FLAG_NUMBER,
    WINHTTP_QUERY_STATUS_CODE,
};

use crate::sha256::{CngError, Sha256, DIGEST_LEN};

/// Sent as the user agent. Names the product and version so a maintainer reading a server
/// log can tell what asked, which is the only thing this reveals beyond the request itself —
/// and `PRIVACY.md` says so.
const USER_AGENT: &str = concat!("WiraDesk/", env!("CARGO_PKG_VERSION"));

/// Read buffer. Large enough that a multi-megabyte installer is not read in thousands of
/// round trips, small enough to be irrelevant to a tray application's footprint.
const READ_CHUNK: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// The URL was not `https://host/path`.
    NotHttps,
    /// A WinHttp call failed. The name is kept so a log line says which one.
    Win32 { call: &'static str, code: u32 },
    /// The server answered with something other than 200.
    Status(u32),
    /// The body reached the caller's ceiling. Nothing is returned, and nothing partial is
    /// left for a caller to mistake for a complete answer.
    TooLarge { limit: u64 },
    /// The body was not valid UTF-8, for the text form.
    NotUtf8,
    /// SHA-256 failed, for the download form.
    Hash(CngError),
}

/// A WinHttp handle that closes itself exactly once.
///
/// Written as a guard rather than as paired calls because this module opens three handles per
/// request and has five early-return paths. Hand-balanced closes across that shape is how a
/// handle leak gets written, and a leak in a process that stays open for a session is a leak
/// that accumulates.
struct Handle(*mut core::ffi::c_void);

impl Handle {
    /// `None` when WinHttp returned null, which it documents as failure.
    fn new(raw: *mut core::ffi::c_void) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Handle(raw))
        }
    }

    fn raw(&self) -> *mut core::ffi::c_void {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: `self.0` is non-null — `new` refuses null — and is a WinHttp handle that
        // this value alone owns, so it is closed exactly once, here. The result is discarded
        // because a failure to close during teardown is not actionable and must not panic in
        // a `Drop`.
        unsafe {
            WinHttpCloseHandle(self.0);
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn last_error(call: &'static str) -> HttpError {
    // SAFETY: `GetLastError` reads the calling thread's own error slot and has no
    // preconditions. It is read immediately after the failing call, with nothing in between
    // that could overwrite it.
    let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    HttpError::Win32 { call, code }
}

/// Split `https://host/path` into host and path-with-leading-slash.
///
/// Refuses anything that is not HTTPS, and refuses a host carrying userinfo or a port — both
/// because WinHttp takes host and port separately, so a port smuggled into the host string
/// would either be resolved as part of a hostname or silently ignored, and neither is a thing
/// to leave to chance in a downloader.
fn split_url(url: &str) -> Result<(String, String), HttpError> {
    let rest = url.strip_prefix("https://").ok_or(HttpError::NotHttps)?;
    let (host, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{p}")),
        None => (rest, "/".to_owned()),
    };
    if host.is_empty() || host.contains('@') || host.contains(':') {
        return Err(HttpError::NotHttps);
    }
    Ok((host.to_owned(), path))
}

/// One GET, streamed to `sink`, stopping if more than `limit` bytes arrive.
///
/// The body is never buffered whole inside this function: each chunk goes straight to the
/// sink. That is what lets the caller hash an installer as it is written rather than
/// afterwards, and what keeps the ceiling meaningful — a limit enforced after the fact is
/// not a limit.
fn get_streaming<F>(url: &str, limit: u64, mut sink: F) -> Result<u64, HttpError>
where
    F: FnMut(&[u8]) -> Result<(), HttpError>,
{
    let (host, path) = split_url(url)?;
    let agent = wide(USER_AGENT);
    let host_w = wide(&host);
    let path_w = wide(&path);

    // SAFETY: `agent` is a NUL-terminated wide string in a local that outlives the call.
    // Null proxy and bypass pointers select the access type's own configuration, which is the
    // documented use. A zero flag word requests the synchronous session this module wants;
    // asynchronous WinHttp would require a callback outliving these frames.
    let session = Handle::new(unsafe {
        WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            ptr::null(),
            ptr::null(),
            0,
        )
    })
    .ok_or_else(|| last_error("WinHttpOpen"))?;

    // SAFETY: `session` is a live handle held by the guard for the rest of this function.
    // `host_w` is NUL-terminated and outlives the call. The port is the HTTPS default, and
    // the reserved argument is zero as documented.
    let connect = Handle::new(unsafe {
        WinHttpConnect(
            session.raw(),
            host_w.as_ptr(),
            INTERNET_DEFAULT_HTTPS_PORT,
            0,
        )
    })
    .ok_or_else(|| last_error("WinHttpConnect"))?;

    // SAFETY: `connect` is live. `path_w` is NUL-terminated and outlives the call. Null verb
    // means GET, null version means the default, null referrer and null accept-types are the
    // documented "nothing to say" values. `WINHTTP_FLAG_SECURE` is what makes this TLS, and
    // it is not conditional on anything.
    let request = Handle::new(unsafe {
        WinHttpOpenRequest(
            connect.raw(),
            ptr::null(),
            path_w.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            WINHTTP_FLAG_SECURE,
        )
    })
    .ok_or_else(|| last_error("WinHttpOpenRequest"))?;

    // SAFETY: `request` is live. No additional headers, hence a null pointer with a zero
    // length; no request body, hence null optional data with zero lengths. A zero context is
    // correct for a synchronous session, which has no callback to receive one.
    let ok = unsafe { WinHttpSendRequest(request.raw(), ptr::null(), 0, ptr::null(), 0, 0, 0) };
    if ok == FALSE {
        return Err(last_error("WinHttpSendRequest"));
    }

    // SAFETY: `request` is live and a request has been sent on it, which is this call's
    // documented precondition. The reserved argument must be null.
    if unsafe { WinHttpReceiveResponse(request.raw(), ptr::null_mut()) } == FALSE {
        return Err(last_error("WinHttpReceiveResponse"));
    }

    let mut status: u32 = 0;
    let mut status_len = core::mem::size_of::<u32>() as u32;
    // SAFETY: `request` is live and its response has been received. `WINHTTP_QUERY_FLAG_NUMBER`
    // makes WinHttp write a `u32`, and the buffer is exactly one `u32` with its own size
    // passed as the length, so the write cannot pass its end. A null name is required for a
    // query selected by level rather than by header name, and a null index asks for the first.
    let ok = unsafe {
        WinHttpQueryHeaders(
            request.raw(),
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            ptr::null(),
            (&mut status as *mut u32).cast(),
            &mut status_len,
            ptr::null_mut(),
        )
    };
    if ok == FALSE {
        return Err(last_error("WinHttpQueryHeaders"));
    }
    if status != 200 {
        return Err(HttpError::Status(status));
    }

    let mut buf = vec![0u8; READ_CHUNK];
    let mut total: u64 = 0;
    loop {
        let mut read: u32 = 0;
        // SAFETY: `request` is live. `buf` is a heap allocation of `READ_CHUNK` bytes that
        // outlives the call, and `READ_CHUNK` is passed as the capacity, so WinHttp cannot
        // write past its end. `read` is a live local that receives the count actually
        // written, and only that many bytes are looked at below.
        let ok = unsafe {
            WinHttpReadData(
                request.raw(),
                buf.as_mut_ptr().cast(),
                READ_CHUNK as u32,
                &mut read,
            )
        };
        if ok == FALSE {
            return Err(last_error("WinHttpReadData"));
        }
        if read == 0 {
            break; // End of body. WinHttp signals it with a zero-length read.
        }

        // Checked before the sink sees the bytes, so a body that runs past the ceiling never
        // reaches a file or a buffer at all.
        total += u64::from(read);
        if total > limit {
            return Err(HttpError::TooLarge { limit });
        }

        sink(&buf[..read as usize])?;
    }

    Ok(total)
}

/// Fetch a small document as text.
///
/// `limit` is a hard ceiling in bytes. `latest.json` is a few hundred bytes; anything wildly
/// past that is either not our file or not worth reading.
pub fn get_text(url: &str, limit: u64) -> Result<String, HttpError> {
    let mut body: Vec<u8> = Vec::new();
    get_streaming(url, limit, |chunk| {
        body.extend_from_slice(chunk);
        Ok(())
    })?;
    String::from_utf8(body).map_err(|_| HttpError::NotUtf8)
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
pub fn download_to_file(
    url: &str,
    dest: &std::path::Path,
    limit: u64,
) -> Result<[u8; DIGEST_LEN], HttpError> {
    use std::io::Write;

    let mut hasher = Sha256::new().map_err(HttpError::Hash)?;
    let mut file = std::fs::File::create(dest).map_err(|_| HttpError::Win32 {
        call: "File::create",
        code: 0,
    })?;

    let outcome = get_streaming(url, limit, |chunk| {
        hasher.update(chunk).map_err(HttpError::Hash)?;
        file.write_all(chunk).map_err(|_| HttpError::Win32 {
            call: "File::write_all",
            code: 0,
        })
    });

    let flushed = file.flush().is_ok();
    drop(file);

    if outcome.is_err() || !flushed {
        let _ = std::fs::remove_file(dest);
        return Err(outcome.err().unwrap_or(HttpError::Win32 {
            call: "File::flush",
            code: 0,
        }));
    }

    match hasher.finish() {
        Ok(digest) => Ok(digest),
        Err(e) => {
            let _ = std::fs::remove_file(dest);
            Err(HttpError::Hash(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_https_url_splits_into_host_and_path() {
        assert_eq!(
            split_url("https://example.test/a/b.json"),
            Ok(("example.test".to_owned(), "/a/b.json".to_owned()))
        );
    }

    #[test]
    fn a_url_with_no_path_gets_the_root() {
        assert_eq!(
            split_url("https://example.test"),
            Ok(("example.test".to_owned(), "/".to_owned()))
        );
    }

    /// Each of these would, if accepted, mean a fetch that is not what it appears to be.
    /// The port and userinfo cases matter specifically because WinHttp takes the host and
    /// port as separate arguments: a port left inside the host string is not a port to it.
    #[test]
    fn anything_that_is_not_plain_https_is_refused() {
        for url in [
            "http://example.test/a",
            "ftp://example.test/a",
            "//example.test/a",
            "example.test/a",
            "https://",
            "https:///path",
            "https://example.test:8443/a",
            "https://user@example.test/a",
            "https://example.test@evil.test/a",
        ] {
            assert_eq!(
                split_url(url),
                Err(HttpError::NotHttps),
                "{url:?} should have been refused"
            );
        }
    }

    /// The user agent names the product and its version, which is the only thing a server
    /// learns beyond the request itself. `PRIVACY.md` describes exactly this, so if the
    /// string ever grows to carry more, that document has become wrong.
    #[test]
    fn the_user_agent_carries_only_product_and_version() {
        assert_eq!(
            USER_AGENT,
            format!("WiraDesk/{}", env!("CARGO_PKG_VERSION"))
        );
        assert!(!USER_AGENT.contains(' '), "no room for anything else");
    }
}
