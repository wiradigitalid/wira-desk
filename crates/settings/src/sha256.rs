//! SHA-256, over Windows' own CNG provider.
//!
//! Used for one thing: proving a downloaded installer is the file `latest.json` named.
//! **Until a code-signing certificate exists this is the only verification the updater
//! has**, which makes it a security control rather than a convenience, and is why it is
//! its own module with its own tests rather than a helper buried in the download code.
//!
//! No new dependency: `BCrypt*` comes from the `windows-sys` already in the tree. That
//! matters more here than elsewhere — a hash function pulled in from the ecosystem to
//! verify downloads would widen the supply chain of the thing doing the verifying.
//!
//! # Incremental on purpose
//! [`Sha256`] hashes as bytes arrive rather than over a finished buffer, so the download
//! path never has to hold a complete unverified installer in memory or leave one on disk
//! and come back to check it later. Hashing what was written, as it is written, is what
//! makes "verified before it is ever executed" true by construction.

use std::ptr;

use windows_sys::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
    BCRYPT_SHA256_ALGORITHM,
};

/// Length of a SHA-256 digest in bytes.
pub const DIGEST_LEN: usize = 32;

/// Anything CNG refused to do. The `NTSTATUS` is kept because a failure here is either a
/// broken platform or a bug in this file, and both need the number to diagnose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CngError {
    pub call: &'static str,
    pub status: i32,
}

/// An incremental SHA-256, owning its CNG algorithm and hash handles.
pub struct Sha256 {
    alg: BCRYPT_ALG_HANDLE,
    hash: BCRYPT_HASH_HANDLE,
}

impl Sha256 {
    pub fn new() -> Result<Self, CngError> {
        let mut alg: BCRYPT_ALG_HANDLE = ptr::null_mut();
        // SAFETY: `alg` is a live local written only on success; the all-null value it starts
        // from is what CNG documents as an unopened provider, so an early return leaves
        // nothing to close. `BCRYPT_SHA256_ALGORITHM` is a `w!` static NUL-terminated wide
        // string with `'static` lifetime, and a null implementation pointer is the documented
        // request for the default provider.
        let status = unsafe {
            BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_SHA256_ALGORITHM, ptr::null(), 0)
        };
        if status < 0 {
            return Err(CngError {
                call: "BCryptOpenAlgorithmProvider",
                status,
            });
        }

        let mut hash: BCRYPT_HASH_HANDLE = ptr::null_mut();
        // SAFETY: `alg` was just opened and is not null. A null object buffer with a zero
        // length asks CNG to allocate the hash object itself, which is the documented modern
        // form and the reason no buffer has to be kept alive beside the handle. A null secret
        // with zero length selects a plain hash rather than an HMAC.
        let status =
            unsafe { BCryptCreateHash(alg, &mut hash, ptr::null_mut(), 0, ptr::null(), 0, 0) };
        if status < 0 {
            // The provider opened, so it must be closed before returning the error, or this
            // failure path leaks it.
            // SAFETY: `alg` is a provider handle opened above and not yet closed; closing it
            // exactly once on this path is the whole point of doing it here.
            unsafe {
                BCryptCloseAlgorithmProvider(alg, 0);
            }
            return Err(CngError {
                call: "BCryptCreateHash",
                status,
            });
        }

        Ok(Sha256 { alg, hash })
    }

    /// Feed the next chunk.
    pub fn update(&mut self, bytes: &[u8]) -> Result<(), CngError> {
        // Chunked because the API takes a `u32` length and a download is not bounded by
        // `u32::MAX`. Silently truncating a hash input would produce a digest that is
        // confidently wrong, which is worse than any error this could return.
        for chunk in bytes.chunks(u32::MAX as usize) {
            // SAFETY: `self.hash` is a live handle owned by `self` and destroyed only in
            // `Drop`. `chunk` is a live slice for the duration of the call, and its length is
            // passed as the byte count, so CNG reads exactly the bytes that exist. An empty
            // slice yields a null-ish pointer with a zero count, which CNG accepts as a
            // no-op.
            let status =
                unsafe { BCryptHashData(self.hash, chunk.as_ptr(), chunk.len() as u32, 0) };
            if status < 0 {
                return Err(CngError {
                    call: "BCryptHashData",
                    status,
                });
            }
        }
        Ok(())
    }

    /// Finish and return the digest. Consumes the hasher: CNG's hash object is not reusable
    /// after finishing, and letting it be called twice would return a digest of nothing.
    pub fn finish(self) -> Result<[u8; DIGEST_LEN], CngError> {
        let mut digest = [0u8; DIGEST_LEN];
        // SAFETY: `self.hash` is live. `digest` is a stack array of exactly `DIGEST_LEN`
        // bytes and that same constant is passed as the buffer length, so the write cannot
        // pass its end — SHA-256's digest is 32 bytes, so the size is right for the algorithm
        // opened in `new`.
        let status =
            unsafe { BCryptFinishHash(self.hash, digest.as_mut_ptr(), DIGEST_LEN as u32, 0) };
        if status < 0 {
            return Err(CngError {
                call: "BCryptFinishHash",
                status,
            });
        }
        Ok(digest)
    }
}

impl Drop for Sha256 {
    fn drop(&mut self) {
        // SAFETY: both handles were opened in `new` and are non-null for the whole life of
        // the value — there is no path that clears them — so each is destroyed exactly once,
        // here. Order matters: the hash object belongs to the provider, so it goes first.
        // Return values are discarded deliberately; a failure to close during teardown is
        // not actionable and must not panic in a `Drop`.
        unsafe {
            BCryptDestroyHash(self.hash);
            BCryptCloseAlgorithmProvider(self.alg, 0);
        }
    }
}

/// Lowercase hexadecimal, the form `latest.json` and `SHA256SUMS` both use.
pub fn to_hex(digest: &[u8; DIGEST_LEN]) -> String {
    let mut out = String::with_capacity(DIGEST_LEN * 2);
    for byte in digest {
        out.push(char::from_digit((byte >> 4) as u32, 16).unwrap());
        out.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap());
    }
    out
}

/// Constant-time-ish comparison of a computed digest against an expected hex string.
///
/// Case-insensitive on the expected side, because `SHA256SUMS` and `latest.json` are written
/// by different tools and there is no reason to fail a match over that. Compared byte by
/// byte with no early return, which is habit rather than necessity here — nothing about a
/// release checksum is secret — but a comparison that leaks where it stopped is not a
/// pattern worth having in a file whose whole job is verification.
pub fn matches_hex(digest: &[u8; DIGEST_LEN], expected_hex: &str) -> bool {
    let computed = to_hex(digest);
    if computed.len() != expected_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in computed.bytes().zip(expected_hex.bytes()) {
        diff |= a ^ b.to_ascii_lowercase();
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Digest of a complete slice. Lives here rather than beside `to_hex` because the
    /// download path hashes incrementally and never holds all the bytes at once, so this
    /// shape has only ever had test callers. Exporting it would invite the pattern the
    /// download layer exists to avoid.
    fn hex_of(bytes: &[u8]) -> Result<String, CngError> {
        let mut hasher = Sha256::new()?;
        hasher.update(bytes)?;
        Ok(to_hex(&hasher.finish()?))
    }

    /// The published NIST vectors. These are the reason this module is testable at all: the
    /// download layer above it cannot be exercised without a network, but the thing that
    /// decides whether a download is trusted can be checked against values no one in this
    /// project chose.
    const VECTORS: &[(&str, &str)] = &[
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ];

    #[test]
    fn matches_the_published_vectors() {
        for (input, expected) in VECTORS {
            assert_eq!(
                hex_of(input.as_bytes()).expect("CNG should hash"),
                *expected,
                "SHA-256 of {input:?}"
            );
        }
    }

    /// The property the download path depends on: bytes fed in pieces must produce the same
    /// digest as the same bytes fed at once. Without it, hashing while writing would give a
    /// different answer from hashing afterwards, and the verification would be meaningless.
    #[test]
    fn feeding_in_chunks_matches_feeding_at_once() {
        let data: Vec<u8> = (0u8..=255).cycle().take(10_000).collect();
        let whole = hex_of(&data).expect("CNG should hash");

        for chunk_size in [1usize, 7, 64, 1024, 9_999] {
            let mut hasher = Sha256::new().expect("CNG should open");
            for chunk in data.chunks(chunk_size) {
                hasher.update(chunk).expect("CNG should hash");
            }
            assert_eq!(
                to_hex(&hasher.finish().expect("CNG should finish")),
                whole,
                "chunked by {chunk_size} disagreed with the whole"
            );
        }
    }

    #[test]
    fn an_empty_update_does_not_disturb_the_digest() {
        let mut hasher = Sha256::new().expect("CNG should open");
        hasher.update(b"").expect("empty is a no-op");
        hasher.update(b"abc").expect("CNG should hash");
        hasher.update(b"").expect("empty is a no-op");
        assert_eq!(to_hex(&hasher.finish().unwrap()), VECTORS[1].1);
    }

    #[test]
    fn hex_is_lowercase_and_sixty_four_characters() {
        let hex = hex_of(b"anything").unwrap();
        assert_eq!(hex.len(), 64);
        assert!(hex.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(hex, hex.to_lowercase());
    }

    #[test]
    fn comparison_accepts_either_case_and_rejects_anything_else() {
        let mut hasher = Sha256::new().unwrap();
        hasher.update(b"abc").unwrap();
        let digest = hasher.finish().unwrap();

        assert!(matches_hex(&digest, VECTORS[1].1));
        assert!(matches_hex(&digest, &VECTORS[1].1.to_uppercase()));

        // One digit wrong, the same length.
        let mut wrong = VECTORS[1].1.to_owned();
        wrong.replace_range(0..1, "c");
        assert!(!matches_hex(&digest, &wrong));

        // Wrong length, and a prefix of the right answer -- which is the case a
        // `starts_with` implementation would have accepted.
        assert!(!matches_hex(&digest, &VECTORS[1].1[..63]));
        assert!(!matches_hex(&digest, ""));
    }
}
