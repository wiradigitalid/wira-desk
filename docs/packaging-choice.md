# Why NSIS, and what would make Inno Setup the right answer instead

Researched 2026-08-27. This document exists because the packaging choice was made from one
requirement (`cargo-packager` has no uninstall hook) without ever comparing the two tools
that were actually in contention. That is a thin basis for a decision this hard to reverse
once users have an installer, so this is the comparison, against **this product's**
requirements rather than a generic feature list.

**Verdict, revised 2026-08-27 after building the Inno Setup equivalent:** the two options are
close enough that either is defensible, and the lean is now toward **Inno Setup** rather than
away from it. The first version of this document said "stay on NSIS" on the strength of three
arguments. Building a working `.iss` weakened two of them and disproved one outright — see
[Measured, not argued](#measured-not-argued), which is the section that should carry the most
weight here because it is the only one containing numbers rather than reading.

The decision window is **now**, before `v0.1.1`. After release the two engines cannot hand over
to each other: the old uninstaller and the new installer do not know about each other, and users
end up with both registered.

Three claims made earlier in the same week's work are corrected in this document and marked
where they appear. All three were stated with more confidence than the evidence supported, and
all three were caught by building the thing rather than by reading about it.

## The requirements this product actually has

Generic comparisons are useless here because most of what makes this installer hard comes from
one property: **the daemon runs elevated at every logon with no UAC prompt.** Everything below
follows from that or from a rule already written down elsewhere in the repository.

| # | Requirement | Where it comes from |
| --- | --- | --- |
| R1 | Per-machine install under `%ProgramFiles%`, with **no** per-user option offered | `SECURITY.md` hardening; the `acl` module exists to detect violations of it |
| R2 | Stop the running daemon **gracefully** before replacing its image — the hook unhooked and the tray icon released, not a kill | `tray.rs` `WM_DESTROY` arm |
| R3 | Delete the `WiraDesk` scheduled task on uninstall | `BR-7`; an `ONLOGON` `/RL HIGHEST` task outliving its binary is the worst residue possible |
| R4 | Must **not** create the scheduled task | `BR-4` — the daemon owns registration, and it is the user's decision |
| R5 | Must **not** delete `%APPDATA%\WiraDesk\` | An elevated uninstaller's `%APPDATA%` is the administrator's, and the folder carries migration semantics |
| R6 | Silent install and silent uninstall | winget drives both non-interactively |
| R7 | Install path stable across upgrades | The logon task stores an absolute path |
| R8 | x64 only, refusing to install elsewhere | The workspace targets `x86_64-pc-windows-msvc` only |
| R9 | Version taken from the crate, single source of truth | `verify-public-export.ps1` already enforces agreement across the tree |
| R10 | Toolchain pinnable and SHA-256 verifiable in CI | `ci.yml`'s existing convention for gitleaks and cargo-deny |
| R11 | Sign the installer **and the uninstaller** when a certificate exists | `SECURITY.md` release integrity |
| R12 | The toolchain must not itself be the vulnerability | Learned the hard way; see `3p.md` 2026-08-27 |

## The matrix

| # | NSIS | Inno Setup | Better fit | Why it matters here |
| --- | --- | --- | --- | --- |
| R1 | `RequestExecutionLevel admin` + `$PROGRAMFILES64`. Per-user is simply not offered because nothing offers it | `PrivilegesRequired=admin`, and `PrivilegesRequiredOverridesAllowed` must be left **unset** or the user gets a per-user choice | **Tie**, with a trap on the Inno side | Inno's per-user path is a directive away, so the safe configuration is the one you must not forget to leave alone |
| R2 | Hand-written: `FindWindow` on the daemon's known class and title, then `SendMessage WM_CLOSE`, poll, `taskkill /F` only after six seconds | `CloseApplications=yes` uses the **Windows Restart Manager**, which in silent mode closes and restarts automatically. But Restart Manager decides *how*, and a hidden `WS_EX_TOOLWINDOW` window with no `WM_QUERYENDSESSION` handler is likely to be force-terminated after its timeout rather than closed cleanly. Matching our behaviour needs custom `[Code]` | **NSIS** | **Correction.** Earlier this week Inno was described as able to close the application before replacing files "more tidily than the NSIS trick I used". That was wrong in the direction that matters: Restart Manager decides the method, and for a hidden tool window with no session handler the likely method is force-termination — which skips the `WM_DESTROY` arm that unhooks `WH_KEYBOARD_LL` and removes the tray icon. `AppMutex` is not the answer either: it only *warns*, and this product's single-instance mutex would produce a dialog rather than a shutdown |
| R3 | `Section "Uninstall"` is first-class; the task is removed with `nsExec::ExecToLog 'schtasks /Delete …'` | `[UninstallRun]` — declarative, one line, no plugin call. Or `CurUninstallStepChanged` for logic | **Inno**, on readability | Both work. Inno's is a line of configuration where ours is a plugin invocation whose return value must be popped off the stack or it corrupts later calls |
| R4 | Nothing to do — just do not write it | Nothing to do | **Tie** | Worth listing because the temptation to "helpfully" register autostart from the installer is exactly what neither tool should be asked to do |
| R5 | Nothing to do | Nothing to do, but Inno's `usPostUninstall` step and `[UninstallDelete]` make deleting app data easy enough that it happens by accident | **Tie**, with a trap on the Inno side | The safe behaviour is inaction, and inaction is easier to preserve in the tool that makes it harder to act |
| R6 | `/S` for install; `QuietUninstallString` written by hand into the uninstall key | `/VERYSILENT /SUPPRESSMSGBOXES`; the uninstaller takes the same switches. Better documented for package managers | **Inno**, marginally | Ours works, but we had to know to write `QuietUninstallString` ourselves. Nothing would have told us if we had not |
| R7 | `InstallDirRegKey` reads the previous `InstallLocation` | `UsePreviousAppDir=yes`, the default | **Inno**, marginally | Inno's correct behaviour is the default; ours is a directive we had to remember |
| R8 | `${RunningX64}` from `x64.nsh`, then abort | `ArchitecturesAllowed=x64compatible` — declarative. Note Inno **7 renamed these**: `x64` and `x86` are deprecated in favour of `x64compatible` and `x86compatible` | **Inno**, with a version caveat | Declarative beats an imperative guard, but the identifier depends on which major version you are on |
| R9 | `makensis /DVERSION=…` | `iscc /DVersion=…` with the preprocessor | **Tie** | Equivalent in practice |
| R10 | Official `.zip` from SourceForge, pinnable and checksummable, extracted with no administrator rights. Note SourceForge answers `Invoke-WebRequest` with an HTML interstitial and only `curl.exe` with the archive | Setup `.exe` from **GitHub Releases**, and its own installer supports **`/PORTABLE=1`** — so it installs into a scratch directory with **no administrator rights** and no registry writes. `ISCC.exe` then runs from there | **Tie, with a slight edge to Inno on host reliability** | **Correction, and this was the strongest of the three original NSIS arguments.** The first version of this document said Inno "needs an install step or a third-party dependency". It needs neither: one `curl` from GitHub Releases, one `Get-FileHash`, and one `/PORTABLE=1 /VERYSILENT` run — verified working in a non-elevated shell. GitHub Releases is also the more dependable host of the two, since the NSIS download requires knowing that one HTTP client works and another silently does not |
| R11 | `!uninstfinalize` — a compile-time hook over the generated uninstaller. No side files, no prompt, nothing to preserve between builds | `SignedUninstaller=yes` + `SignTool`. Works, but it writes a **persistent** uninstaller copy into `SignedUninstallerDir` whose signature is reused on later compiles, and it prompts on first compile unless the sign tool is configured on the command line | **NSIS** | **Correction.** Earlier in this week's work Inno's native uninstaller signing was listed as an advantage over NSIS. It is native, but for an automated pipeline it is the more awkward of the two — there is a stateful artefact to manage that NSIS does not have |
| R12 | **CVE-2023-37378** — uninstaller temporary directory writable by all users, DLL plant, local privilege escalation. CVSS 7.8. Affects ≤ 3.08, fixed in 3.09. NSIS 3.12 fixed a further elevated-temp-directory escalation | **CVE-2025-15595** — DLL hijacking privilege escalation during installer execution. CVSS 7.8. Affects ≤ 6.2.1. Inno 7 enables **RedirectionGuard** by default and adds ECDSA P-256 integrity verification | **Tie — and this is the most useful row in the table** | Both tools shipped a 7.8 local privilege escalation of the same class within two years. Neither is "the safe one." What matters is the version and whether anything in CI enforces a floor |

## Measured, not argued

`packaging/wiradesk.iss` is a working equivalent of the NSIS script, built and compiled on
2026-08-27 with Inno Setup 6.7.3. It implements all eight requirements, deliberately including
the awkward one, because a prototype that skipped the hard part would have flattered Inno rather
than tested it. Everything in this section is a number taken off that build.

| Measurement | NSIS 3.12 | Inno Setup 6.7.3 | Reading |
| --- | --- | --- | --- |
| Compiles clean | Yes, exit 0, no warnings | Yes, exit 0 — after adding `RunOnceId` to the `[UninstallRun]` entry, which the compiler correctly warned about | Tie. Inno's warning was a real one and worth having |
| Installer size, identical 17 MB payload | **6.35 MiB** | 8.13 MiB | **NSIS, by 28%.** Real, and irrelevant for a desktop utility at this scale |
| Script length, comments and blanks excluded | 136 lines | **113 lines** | Inno, by about 17%. Less of a landslide than "more readable" implied — the difference is in the *kind* of line, not the count |
| Version handling | Passed in as `/DVERSION=`, and the release workflow carries a step comparing the git tag against the crate version | **Read out of the built binary** with `GetStringFileInfo(exe, "FileVersion")` — proven: the output was named `WiraDesk-0.1.0-…` with no version supplied on the command line | **Inno.** It removes a mismatch class rather than guarding it |
| Graceful daemon shutdown | Hand-written `FindWindow` + `WM_CLOSE` loop | Hand-written `[Code]` doing the same, and `CloseApplications=no` set to **switch Restart Manager off** | **Closer to a tie than claimed.** Inno matched the requirement in comparable code. Its declarative advantage does not reach the hardest requirement |
| Toolchain acquisition, unelevated | `curl` + hash + `Expand-Archive` | `curl` + hash + `/PORTABLE=1 /VERYSILENT` — verified in a non-elevated shell | Tie |
| Installer manifest | `requireAdministrator` | **`asInvoker`** | **NSIS**, marginally. Inno's documentation promises Setup "will always run with administrative privileges"; the manifest measured on the compiled binary requests only `asInvoker`, so elevation happens at runtime by a mechanism this investigation did **not** determine. Practically the user sees the same UAC prompt |

Two things that did **not** change: NSIS still has the simpler uninstaller-signing path, and Inno
still has the declarative uninstall action, the automatic upgrade behaviour, and the systematic
message customisation.

## Scoring, honestly

The first version of this section counted four Inno wins against three for NSIS and concluded
that row weight beat row count. After building the prototype the count is roughly five to two,
and — more importantly — the weight has moved as well.

**What is left of the NSIS case, after measurement:**

- **Installer size**, 6.35 MiB against 8.13 MiB. Genuine and unarguable. It is also the least
  consequential advantage on the list for a desktop utility distributed over broadband, and it
  is worth saying plainly that a 28% smaller download is not a reason to keep a script.
- **Uninstaller signing** stays simpler: `!uninstfinalize` runs at compile time with no artefact,
  where `SignedUninstaller` leaves a persistent signed uninstaller whose state must be carried
  between builds. This matters at the moment the certificate arrives.
- **`requireAdministrator` in the manifest** rather than elevation obtained at runtime. Marginal,
  and invisible to the user.
- **It already exists, is verified, and is wired into CI.** Not an argument about the tools at
  all, and the honest weight to give it is "one hour of work", because that is what replacing it
  costs while there are no installed users.

**What is left of the Inno case:**

- **The version comes from the binary.** This is the one that changed my mind, because it does
  not make an existing job easier — it deletes the job. The release workflow currently carries a
  step whose entire purpose is catching a disagreement between the git tag and the crate version.
  A script that asks the executable what version it is cannot disagree with it.
- **Upgrade behaviour is correct by default** through `AppId`, rather than through a directive
  someone has to remember.
- **The uninstall action is one declarative line**, against an `nsExec` call whose return value
  must be popped off the stack or it corrupts the next one.
- **Message customisation is systematic** — `[Messages]` overrides any built-in string by name —
  against NSIS's per-`!define` approach.
- **Slightly less script**, 113 lines against 136, and more of it declarative.

The asymmetry that decides it: **three of Inno's advantages remove ongoing work permanently,
while NSIS's remaining advantages are either one-off or immaterial.** A smaller download does not
compound. A version read from the binary does.

## What is left to decide, and the cost of each answer

Both scripts exist and both compile. This is no longer a question about capability.

**Switching to Inno Setup** costs roughly an hour: point `ci.yml` and `release.yml` at `ISCC` and
the `/PORTABLE=1` install instead of the NSIS zip, drop the tag-versus-crate comparison step
because the version now comes from the binary, delete `wiradesk.nsi`, and re-run the gates. The
`.iss` is already written and already compiles clean.

**Staying with NSIS** costs nothing today and keeps a 28%-smaller installer, a simpler future
signing path, and a script that is verified in CI right now. It also keeps the version-agreement
step, the `nsExec` stack discipline, and a `!macro` instantiated twice because NSIS compiles
installer and uninstaller functions separately.

**What is not optional is choosing before release.** After `v0.1.1` ships, switching engines
means the previous NSIS uninstaller and the new Inno installer are unaware of each other, and a
user who upgrades ends up with two uninstall entries and one of them broken. Today that risk is
zero, which is the cheapest it will ever be.

**Whichever is chosen, delete the other file.** Two installer scripts where CI builds one is how
a repository ends up shipping the one nobody has read in a year.

**Recommendation: switch, if the hour is available.** Not because NSIS is bad — it demonstrably
works — but because the version-from-binary behaviour deletes a class of release bug rather than
guarding against it, and that compounds in a way a smaller download does not. If the hour is not
available, staying is defensible and nothing about it is a mistake; revisit at the next release,
not later, because the window closes at the first one.

## The conclusion worth keeping

The comparison produced a smaller answer than expected: **for this product, both tools work, and
the choice between them is worth less than four things that are true either way.**

1. **Per-machine, and no per-user option offered.** A user-writable location plus an unprompted
   elevated logon task is a local privilege escalation, whichever tool builds the installer.
2. **Uninstall removes the scheduled task.** The single highest-consequence line in the entire
   packaging story.
3. **Shutdown is a request, not a kill** — so the hook is released by the code that installed it.
4. **The toolchain version is pinned current and the floor is asserted in CI.** Both tools have
   shipped a 7.8 privilege escalation. The tool you picked will not protect you; the floor you
   asserted will.

Item 4 is the one that would have changed the outcome of this week's work if it had been written
down first, and it is the reason this document ends on it rather than on a winner.

## Sources

- CVE-2023-37378, NSIS uninstaller temporary directory —
  <https://www.fox-it.com/nl-en/technical-advisory-nullsoft-scriptable-installer-system-nsis-insecure-temporary-directory-usage/>
- CVE-2025-15595, Inno Setup privilege escalation via DLL hijacking —
  <https://www.sentinelone.com/vulnerability-database/cve-2025-15595/>
- NSIS changelog, including the 3.12 elevated temp-directory fix — <https://nsis.sourceforge.io/Docs/AppendixF.html>
- NSIS, signing an uninstaller — <https://nsis.sourceforge.io/Signing_an_Uninstaller>
- Inno Setup `AppMutex` — <https://jrsoftware.org/ishelp/topic_setup_appmutex.htm>
- Inno Setup `CloseApplications` — <https://jrsoftware.org/ishelp/topic_setup_closeapplications.htm>
- Inno Setup `SignedUninstaller` — <https://jrsoftware.org/ishelp/topic_setup_signeduninstaller.htm>
- Inno Setup `[Run]` and `[UninstallRun]` — <https://jrsoftware.org/ishelp/topic_runsection.htm>
- Inno Setup 7 revision history — <https://jrsoftware.org/files/is7-whatsnew.htm>
- Inno Setup compiler command line — <https://jrsoftware.org/ishelp/topic_compilercmdline.htm>
