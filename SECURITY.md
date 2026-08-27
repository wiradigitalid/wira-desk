# Security Policy

Wira Desk runs elevated, installs a global low-level keyboard hook (`WH_KEYBOARD_LL`), can
register a logon task that starts it elevated without a prompt, and enumerates top-level
windows to implement switching and snapping. Those properties deserve scrutiny, so
`docs/threat-model.md` documents the trust boundaries, the reason for each privilege, and the
risks that remain after mitigation. Read that first if you are evaluating whether to trust
this software.

Two facts most people want up front:

- **No keystroke content is recorded.** The hook reads virtual-key codes to match the two
  configured shortcuts and writes none of them to disk, to the log, or to the debug trace.
  No logging call in the codebase takes a key value as an argument.
- **There is no network path.** No socket, HTTP client, update check, or telemetry exists in
  the source. Configuration and logs stay in `%APPDATA%\WiraDesk\`.

## Reporting a vulnerability

Use **GitHub Security Advisories** on this repository ("Report a vulnerability" under the
Security tab) so the report stays private until a fix exists. Please do not open a public
issue for a suspected vulnerability.

Helpful in a report: the Windows build, the Wira Desk version, what you did, what happened,
and â€” if you have one â€” a minimal reproduction. There is no bounty programme and no
guaranteed response time; this is a small project and honesty about that is more useful than
a promise it cannot keep.

In scope: anything that lets a non-administrator gain privileges, read data the daemon should
not expose, or make the elevated daemon act on input it should not trust.

Out of scope, and documented rather than fixed: an attacker who is already an administrator;
disabling the shortcut through resource squatting (denial of a convenience feature, no
privilege gain); and the residual risks listed in `docs/threat-model.md`, which are known
trade-offs rather than unreported bugs.

## Supported versions

Only the latest release receives fixes. This is version 0.1.0, the initial public source
release; there is no long-term support branch.

## Release integrity

**Binaries are not code-signed.** Two consequences, stated plainly because both are
user-visible:

- Windows SmartScreen will warn on first run, and because the daemon requires elevation the
  UAC prompt shows an unverified publisher. That is expected for an unsigned build and is not
  evidence of tampering â€” but it also means the prompt cannot help you tell the two apart.
- The strongest verification available today is **building from source in this repository**,
  which is why the full source is published rather than binaries alone. If a release
  publishes checksums, verify them with `Get-FileHash` before running; a checksum served from
  the same place as the download proves integrity of transfer, not of origin.

Signing is understood to be the real fix and is not yet in place. Until it is, treat any
"Wira Desk" binary from anywhere other than this repository's releases as untrusted.

## Hardening guidance

Both of these matter more than anything else on this page:

- **Install where only administrators can write** â€” `%ProgramFiles%` or similar. The
  auto-start task runs the daemon elevated at every logon with no prompt, so anyone able to
  overwrite the executable at that path gains an unprompted elevated foothold. The task
  stores an absolute path and sets no working directory, so the path itself cannot be
  hijacked; the file permissions are what protect it.
- **Do not enable auto-start from a build in `Downloads`, `Desktop`, or any other
  user-writable folder**, for the same reason.

Both points are now **checked rather than only asked for**, and it is worth being precise
about what that does and does not mean:

- The daemon reads the permissions of its own executable and of the directory holding it. If
  a principal that is not an administrator holds a right that would let it replace either,
  and auto-start is registered, you get a Tier-2 warning â€” a line in `wiradesk.log` and the
  warning dot on the tray icon. **It warns; it does not refuse.** Auto-start still turns on,
  because a check that blocked running from a build directory would be switched off rather
  than heeded, and the choice is yours to make knowingly. Silence here is not a clean bill of
  health for anything but this one question.
- The stored path no longer goes stale. Because the task's action is an absolute path frozen
  when auto-start was switched on, moving the executable used to leave the logon task aimed
  at the old location â€” so installing properly *after* first running from `Downloads` left
  the download as the thing Windows launched elevated. The daemon now re-points the task at
  itself on every start.

Neither is a substitute for installing in the right place. They tell you when you have not.

The installer is the third piece, and it is the one that makes the right place the default:
it requires Administrator, installs to `%ProgramFiles%\Wira Desk`, and offers **no per-user
install location**. An installer that offered `%LOCALAPPDATA%` would be offering the
escalation route above as a convenience, so it does not offer it. The installer also does
not enable auto-start; registering an unprompted elevated logon task is a decision that
stays with you.

Uninstalling removes the scheduled task. An `ONLOGON` task with `/RL HIGHEST` outliving the
executable it names is the worst thing an uninstaller could leave behind.

## Design notes

- Elevation exists for one purpose: activating and moving windows owned by higher-integrity
  processes, which Windows (UIPI) blocks otherwise. It is not used to read other processes'
  memory â€” the daemon opens processes with `PROCESS_QUERY_LIMITED_INFORMATION`, never
  `PROCESS_VM_READ`. The manifest is not the only check: the daemon re-queries its own token
  and refuses to start unelevated.
- The hook callback is bounded by construction: no heap allocation, no lock, no file I/O, and
  no logging on the callback path.
- `SetDllDirectoryW` runs as the first statement in `main` to drop the current directory from
  the DLL search order, so a planted DLL cannot load with the daemon's privileges.
- Configuration reload uses explicit `WM_APP` messages and is all-or-nothing: an unreadable,
  malformed, or invalid file leaves the previous configuration in force and emits one
  warning. No configuration value ever becomes a path or a command line.
- Every `unsafe` block carries a `SAFETY:` comment stating the precondition it relies on, and
  the compiler enforces that â€” `undocumented_unsafe_blocks` and `missing_safety_doc` are
  `deny` in the workspace lints, so an undocumented block fails the build.