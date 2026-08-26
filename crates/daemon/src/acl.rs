//! Whether a filesystem path could be replaced by a principal that is not an
//! administrator.
//!
//! This exists for exactly one question, asked about exactly one file: the daemon's
//! own executable. The auto-start task runs that file at every logon with
//! `/RL HIGHEST` and no UAC prompt (`autostart`), so whoever can overwrite it gains
//! an unprompted elevated foothold. `SECURITY.md` states that as guidance to the
//! reader — "install where only administrators can write" — and guidance is what
//! this module turns into an observation the product can act on.
//!
//! **Why the DACL and not a list of known-good folders.** A whitelist of
//! `%ProgramFiles%`, `%ProgramFiles(x86)%`, and `%SystemRoot%` would be shorter and
//! need no `unsafe`, but it answers a different question — *is this one of the paths
//! we expect* rather than *is this path actually protected*. It would clear a
//! `C:\Program Files\` subfolder whose ACL an installer had loosened, and condemn a
//! correctly-locked `D:\Apps\WiraDesk`. The permission is the thing that protects the
//! file, so the permission is what gets read.
//!
//! **Verdicts are three, not two.** `Unknown` is a distinct answer from `AdminOnly`
//! and is never folded into it: a DACL that could not be read is not a DACL that
//! turned out to be safe. The caller decides what to do with not knowing.

use std::ffi::c_void;
use std::path::Path;

use windows_sys::Win32::Foundation::{LocalFree, ERROR_SUCCESS};
use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
use windows_sys::Win32::Security::{
    GetAce, ACCESS_ALLOWED_ACE, ACL, DACL_SECURITY_INFORMATION, INHERIT_ONLY_ACE,
    PSECURITY_DESCRIPTOR,
};

use crate::util::{debug_log, wide};

/// `ACE_HEADER::AceType` for an allow entry.
///
/// Defined here rather than imported: the symbol lives in
/// `Win32_System_SystemServices`, a windows-sys module of several thousand
/// unrelated constants that this crate would otherwise compile for one `0`. The
/// value is fixed by the on-disk ACE format and cannot drift.
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;

/// Bits that mean the same thing whichever kind of object carries them: the holder
/// can destroy it, or re-permission it and then do as they like.
const COMMON_REPLACE_RIGHTS: u32 = 0x0001_0000   // DELETE
    | 0x0004_0000                                 // WRITE_DAC   — could grant itself the rest
    | 0x0008_0000                                 // WRITE_OWNER — could take the object over
    | 0x1000_0000                                 // GENERIC_ALL
    | 0x4000_0000; // GENERIC_WRITE

/// Rights over the **executable file** that let a holder change what runs.
const FILE_REPLACE_RIGHTS: u32 = COMMON_REPLACE_RIGHTS
    | 0x0000_0002   // FILE_WRITE_DATA   — overwrite the image
    | 0x0000_0004; // FILE_APPEND_DATA  — modify the image

/// Rights over the **directory holding it** that let a holder change what runs.
///
/// `FILE_ADD_FILE` counts even without a delete right: the executable's own
/// directory is on the DLL search path, so planting a file there is enough. `main`
/// drops the *current* directory from that path with `SetDllDirectoryW`, which is a
/// different directory and does not cover this one.
///
/// **`FILE_APPEND_DATA` (`0x0004`) is deliberately absent here, and its absence is
/// the whole reason these two masks are separate.** The bit is shared: on a file it
/// means append, but on a directory it is `FILE_ADD_SUBDIRECTORY` — permission to
/// create a subdirectory, which cannot touch an existing file. Windows grants it on
/// `C:\` to `Authenticated Users`, so a single combined mask reports the drive root
/// as unsafe. That is not hypothetical: one mask was written first, and the test
/// against the real `%ProgramFiles%` DACL is what caught it.
const DIRECTORY_REPLACE_RIGHTS: u32 = COMMON_REPLACE_RIGHTS
    | 0x0000_0002   // FILE_ADD_FILE     — plant a DLL beside the executable
    | 0x0000_0040; // FILE_DELETE_CHILD — delete the executable, then replace it

/// Principals whose write access to a system location is *already* an
/// administrative privilege, so an ACE granting it tells us nothing new.
///
/// Each entry is `(identifier authority, sub-authorities)`. Everything not listed
/// counts as non-administrative — including `NT AUTHORITY\LOCAL SERVICE` and the
/// backup/server operator groups, which are deliberately absent. They are not
/// administrators, they practically never hold write access on an install
/// directory, and treating an unfamiliar principal as safe is the failure this
/// list must not have.
const ADMINISTRATIVE_SIDS: &[(u64, &[u32])] = &[
    // S-1-5-18 — NT AUTHORITY\SYSTEM
    (5, &[18]),
    // S-1-5-32-544 — BUILTIN\Administrators
    (5, &[32, 544]),
    // S-1-5-80-956008885-… — NT SERVICE\TrustedInstaller, which owns the servicing
    // stack's write access to `%ProgramFiles%` and `%SystemRoot%`. Matched in full
    // rather than as "any S-1-5-80-*": that prefix is every service account there
    // is, and most of them are not administrative.
    (
        5,
        &[
            80, 956008885, 3418522649, 1831038044, 1853292631, 2271478464,
        ],
    ),
];

/// What reading a path's DACL established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Every allow entry granting a replacement right belongs to an administrative
    /// principal.
    AdminOnly,
    /// At least one non-administrative principal can replace what is there.
    NonAdminWritable,
    /// The DACL could not be read, or could not be walked to the end. Not a
    /// synonym for `AdminOnly`.
    Unknown,
}

/// `true` when `granted` carries any of the `dangerous` rights.
fn mask_permits_replacement(granted: u32, dangerous: u32) -> bool {
    granted & dangerous != 0
}

/// Split a binary SID into its identifier authority and sub-authorities.
///
/// Layout, fixed by `SID`: revision byte, sub-authority count byte, a six-byte
/// **big-endian** identifier authority, then that many little-endian `u32`s. Both
/// endiannesses are correct and they differ — the authority is the one field in the
/// structure that is not little-endian.
///
/// `None` when `sid` is too short for the count it declares, which is how a
/// truncated or malformed entry is rejected instead of read past its end.
fn sid_authority_and_subs(sid: &[u8]) -> Option<(u64, Vec<u32>)> {
    if sid.len() < 8 || sid[0] != 1 {
        return None;
    }
    let count = sid[1] as usize;
    if sid.len() < 8 + count * 4 {
        return None;
    }
    let authority = sid[2..8]
        .iter()
        .fold(0u64, |acc, b| (acc << 8) | u64::from(*b));
    let subs = (0..count)
        .map(|i| {
            let o = 8 + i * 4;
            u32::from_le_bytes([sid[o], sid[o + 1], sid[o + 2], sid[o + 3]])
        })
        .collect();
    Some((authority, subs))
}

/// `true` when write access held by this SID is already administrative.
///
/// A SID that cannot be decoded returns `false` — unreadable is treated as
/// non-administrative, so a malformed entry produces a warning rather than
/// silence.
fn sid_is_administrative(sid: &[u8]) -> bool {
    match sid_authority_and_subs(sid) {
        Some((authority, subs)) => ADMINISTRATIVE_SIDS
            .iter()
            .any(|(a, s)| *a == authority && subs.as_slice() == *s),
        None => false,
    }
}

/// Read one path's DACL and decide who can replace what is at it.
///
/// `dangerous` is the right set that matters for this kind of object — see
/// `FILE_REPLACE_RIGHTS` and `DIRECTORY_REPLACE_RIGHTS`, which differ over a bit
/// whose meaning depends on it.
fn verdict_for(path: &Path, dangerous: u32) -> Verdict {
    let wide_path = wide(&path.to_string_lossy());
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();

    // SAFETY: `wide_path` is NUL-terminated by `wide` and is a local that outlives the
    // call. The four `null_mut()` arguments are the documented way to decline the owner,
    // group, and SACL outputs; only `ppdacl` and `ppsecuritydescriptor` receive writes,
    // and both point at locals of exactly the declared types. On anything other than
    // `ERROR_SUCCESS` the function writes neither output, which is why the early return
    // frees nothing — there is nothing yet to free. On success the descriptor is a single
    // `LocalAlloc` block that owns the DACL inline, so `dacl` stays valid exactly as long
    // as `descriptor` does and both are released by the one `LocalFree` below, on every
    // path out of this block.
    unsafe {
        let rc = GetNamedSecurityInfoW(
            wide_path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        );
        if rc != ERROR_SUCCESS {
            debug_log("Wira Desk: acl::verdict_for — GetNamedSecurityInfoW failed");
            return Verdict::Unknown;
        }

        // A NULL DACL is not an empty DACL: it grants everyone full access. Present
        // separately from the walk because the walk below would otherwise read it as
        // "no entries, therefore nobody" — the exact inversion of what it means.
        if dacl.is_null() {
            LocalFree(descriptor);
            return Verdict::NonAdminWritable;
        }

        let mut verdict = Verdict::AdminOnly;
        for index in 0..u32::from((*dacl).AceCount) {
            let mut ace: *mut c_void = std::ptr::null_mut();
            if GetAce(dacl, index, &mut ace) == 0 || ace.is_null() {
                verdict = Verdict::Unknown;
                break;
            }
            let entry = ace as *const ACCESS_ALLOWED_ACE;
            let header = (*entry).Header;

            // Deny entries are skipped rather than subtracted. This module answers
            // "does an allow entry hand a replacement right to a non-administrator",
            // and evaluating deny/allow precedence properly is what the kernel's own
            // access check does — reimplementing it here would be a second, weaker
            // copy of that logic. Skipping is the conservative direction: at worst
            // this warns about a path a deny entry had already secured.
            if header.AceType != ACCESS_ALLOWED_ACE_TYPE {
                continue;
            }

            // An inherit-only entry does not apply to the object carrying it, only to
            // children it is inherited into. `%ProgramFiles%` ships one — CREATOR OWNER
            // with full control, inherit-only — and reading it as effective would
            // condemn a correctly-installed directory on every Windows machine there is.
            if u32::from(header.AceFlags) & INHERIT_ONLY_ACE != 0 {
                continue;
            }

            if !mask_permits_replacement((*entry).Mask, dangerous) {
                continue;
            }

            // The SID is inline at the end of the ACE, starting at the `SidStart`
            // field — offset 8, past the 4-byte header and the 4-byte mask. `AceSize`
            // bounds it, so the slice cannot run past the entry even if the SID's own
            // declared length is wrong; `sid_authority_and_subs` then rejects a length
            // that does not match the count it declares.
            let sid_len = usize::from(header.AceSize).saturating_sub(8);
            if sid_len == 0 {
                continue;
            }
            let sid = std::slice::from_raw_parts((entry as *const u8).add(8), sid_len);

            if !sid_is_administrative(sid) {
                verdict = Verdict::NonAdminWritable;
                break;
            }
        }

        LocalFree(descriptor);
        verdict
    }
}

/// Whether a non-administrator could put different bytes where `exe` is now.
///
/// Both the file and the directory holding it are read, and the worse answer wins.
/// The directory is not redundant: `Modify` on a directory carries `FILE_ADD_FILE`
/// and `FILE_DELETE_CHILD`, which is enough to delete the executable and drop a
/// replacement in its place regardless of how tightly the file's own ACL is set.
/// The reverse case is real too — a loose file inside a locked directory — so
/// neither check substitutes for the other.
pub fn replaceable_by_non_admin(exe: &Path) -> Verdict {
    let mut verdicts = vec![verdict_for(exe, FILE_REPLACE_RIGHTS)];
    if let Some(dir) = exe.parent() {
        verdicts.push(verdict_for(dir, DIRECTORY_REPLACE_RIGHTS));
    }
    if verdicts.contains(&Verdict::NonAdminWritable) {
        Verdict::NonAdminWritable
    } else if verdicts.contains(&Verdict::Unknown) {
        Verdict::Unknown
    } else {
        Verdict::AdminOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a binary SID the way Windows lays one out, so the decoder is tested
    /// against the real format rather than against its own helper.
    fn sid(authority: u64, subs: &[u32]) -> Vec<u8> {
        let mut v = vec![1u8, subs.len() as u8];
        v.extend_from_slice(&authority.to_be_bytes()[2..8]);
        for s in subs {
            v.extend_from_slice(&s.to_le_bytes());
        }
        v
    }

    #[test]
    fn authority_is_big_endian_and_subs_are_little_endian() {
        // The one field in a SID that is not little-endian, pinned so a "tidy-up"
        // to uniform endianness cannot pass.
        let (authority, subs) = sid_authority_and_subs(&sid(5, &[32, 544])).expect("decodes");
        assert_eq!(authority, 5);
        assert_eq!(subs, vec![32, 544]);
    }

    #[test]
    fn administrators_and_system_and_trusted_installer_are_administrative() {
        assert!(sid_is_administrative(&sid(5, &[18])));
        assert!(sid_is_administrative(&sid(5, &[32, 544])));
        assert!(sid_is_administrative(&sid(
            5,
            &[80, 956008885, 3418522649, 1831038044, 1853292631, 2271478464]
        )));
    }

    #[test]
    fn ordinary_principals_are_not_administrative() {
        // S-1-5-11 Authenticated Users — the ACE that made this module necessary.
        assert!(!sid_is_administrative(&sid(5, &[11])));
        // S-1-5-32-545 BUILTIN\Users, S-1-1-0 Everyone, S-1-3-0 CREATOR OWNER.
        assert!(!sid_is_administrative(&sid(5, &[32, 545])));
        assert!(!sid_is_administrative(&sid(1, &[0])));
        assert!(!sid_is_administrative(&sid(3, &[0])));
        // A domain user: same authority as Administrators, different path.
        assert!(!sid_is_administrative(&sid(5, &[21, 1, 2, 3, 1001])));
    }

    #[test]
    fn a_truncated_sid_is_rejected_rather_than_read_past_its_end() {
        // Declares two sub-authorities, carries one.
        let mut truncated = sid(5, &[32, 544]);
        truncated.truncate(12);
        assert_eq!(sid_authority_and_subs(&truncated), None);
        // And an undecodable SID must not be waved through as administrative.
        assert!(!sid_is_administrative(&truncated));
        assert_eq!(sid_authority_and_subs(&[]), None);
        assert_eq!(sid_authority_and_subs(&[9, 0, 0, 0, 0, 0, 0, 0]), None);
    }

    #[test]
    fn write_and_delete_and_takeover_rights_all_permit_replacement() {
        assert!(mask_permits_replacement(0x0000_0002, FILE_REPLACE_RIGHTS)); // FILE_WRITE_DATA
        assert!(mask_permits_replacement(
            0x0000_0040,
            DIRECTORY_REPLACE_RIGHTS
        )); // FILE_DELETE_CHILD
        assert!(mask_permits_replacement(0x0001_0000, FILE_REPLACE_RIGHTS)); // DELETE
        assert!(mask_permits_replacement(0x0004_0000, FILE_REPLACE_RIGHTS)); // WRITE_DAC
        assert!(mask_permits_replacement(0x0008_0000, FILE_REPLACE_RIGHTS)); // WRITE_OWNER
        assert!(mask_permits_replacement(0x001F_01FF, FILE_REPLACE_RIGHTS)); // FILE_ALL_ACCESS
    }

    /// The two tests below read the DACLs Windows itself installed, and they are the
    /// only ones that exercise `verdict_for` — every other test here covers a pure
    /// helper. Without them the ACE walk is unproven: a wrong SID offset, a mishandled
    /// inherit-only flag, or a missing NULL-DACL case all leave the unit tests green.
    ///
    /// They read the real machine on purpose, which makes them the two tests in this
    /// module that could fail for an environmental reason rather than a code reason.
    /// Both anchors are chosen to make that unlikely: `%ProgramFiles%` and the user's
    /// own temp directory have the same shape on every Windows install and on the CI
    /// runner. A failure here is far more likely to be a real regression than a quirk
    /// of the host — treat it as one before dismissing it.
    #[test]
    fn a_system32_executable_reads_as_administrators_only() {
        // The false-positive anchor, and it is checked against a real executable rather
        // than a bare directory because the pair is what `replaceable_by_non_admin`
        // judges: `schtasks.exe` itself, plus `System32` around it. Both grant
        // BUILTIN\Users read-and-execute only, and both carry inherit-only entries with
        // full control. If any of that read as a replacement right, the product would
        // warn on every correctly-installed machine, and a warning that fires when
        // nothing is wrong is one nobody reads.
        //
        // `schtasks.exe` and not `notepad.exe`: this is the very binary `autostart`
        // shells out to, and unlike Notepad it has never been replaced by a Store app.
        let system_root = std::env::var("SystemRoot").expect("SystemRoot is always set");
        let exe = Path::new(&system_root)
            .join("System32")
            .join("schtasks.exe");
        assert!(exe.is_file(), "{} should exist", exe.display());
        assert_eq!(
            replaceable_by_non_admin(&exe),
            Verdict::AdminOnly,
            "a System32 executable must read as administrator-only, or the warning is noise"
        );
    }

    #[test]
    fn this_test_binary_reads_as_non_admin_writable() {
        // The true-positive anchor, and it is the exact shape that produced this module:
        // the running test executable lives under `target\`, which `cargo` and therefore
        // the logged-in user can write. That user's SID is not administrative however
        // many admin groups they belong to — which is the point, because an elevated
        // logon task pointed here would be replaceable without a prompt.
        let exe = std::env::current_exe().expect("the running test binary has a path");
        assert_eq!(
            replaceable_by_non_admin(&exe),
            Verdict::NonAdminWritable,
            "a build-output binary must read as replaceable by a non-administrator"
        );
    }

    #[test]
    fn the_shared_0x04_bit_is_read_differently_for_a_file_and_a_directory() {
        // The bug the real-DACL test above caught, pinned so it cannot come back. On a
        // file `0x04` is FILE_APPEND_DATA and it modifies the image; on a directory the
        // same bit is FILE_ADD_SUBDIRECTORY and it cannot touch an existing file.
        // Windows grants exactly this to Authenticated Users on `C:\`, so folding the
        // two masks back together reports the drive root as unsafe.
        assert!(mask_permits_replacement(0x0000_0004, FILE_REPLACE_RIGHTS));
        assert!(!mask_permits_replacement(
            0x0000_0004,
            DIRECTORY_REPLACE_RIGHTS
        ));
        // FILE_ADD_FILE, by contrast, matters on a directory: the executable's own
        // folder is on the DLL search path.
        assert!(mask_permits_replacement(
            0x0000_0002,
            DIRECTORY_REPLACE_RIGHTS
        ));
    }

    #[test]
    fn read_execute_and_attribute_rights_do_not() {
        // `%ProgramFiles%` grants BUILTIN\Users exactly this. If it ever counted as
        // replacement, the warning would fire on every correctly-installed machine.
        const FILE_GENERIC_READ_EXECUTE: u32 = 0x0012_00A9;
        assert!(!mask_permits_replacement(
            FILE_GENERIC_READ_EXECUTE,
            FILE_REPLACE_RIGHTS
        ));
        assert!(!mask_permits_replacement(
            FILE_GENERIC_READ_EXECUTE,
            DIRECTORY_REPLACE_RIGHTS
        ));
        assert!(!mask_permits_replacement(0x0000_0010, FILE_REPLACE_RIGHTS)); // FILE_WRITE_EA
        assert!(!mask_permits_replacement(0x0000_0100, FILE_REPLACE_RIGHTS)); // FILE_WRITE_ATTRIBUTES
        assert!(!mask_permits_replacement(0, FILE_REPLACE_RIGHTS));
    }
}
