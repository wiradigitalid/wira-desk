# Why NSIS, and what would make Inno Setup the right answer instead

Researched 2026-08-27. This document exists because the packaging choice was made from one
requirement (`cargo-packager` has no uninstall hook) without ever comparing the two tools
that were actually in contention. That is a thin basis for a decision this hard to reverse
once users have an installer, so this is the comparison, against **this product's**
requirements rather than a generic feature list.

**Verdict up front: stay on NSIS now, and the reason is not that NSIS is the better tool.**
Inno Setup is the more maintainable one and the honest expectation is that it wins later. What
keeps NSIS today is three concrete fits with how this repository already works, listed in the
matrix below and summarised at the end.

Two things in this document correct claims made earlier in the same week's work, and they are
marked where they appear. Both were stated with more confidence than the evidence supported.

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
| R10 | Official **`.zip`**, direct download, pinnable and checksummable, extracted with no administrator rights | Distributed as a **setup `.exe`**. CI must run it silently (needs administrator, which runners have), or use `innoextract`, a Docker image, or a third-party action | **NSIS**, and this one is concrete | `ci.yml` already states the convention: pinned version plus published SHA-256, bump the checksum in the same commit. NSIS satisfies it with one `curl` and one hash check. Inno needs an install step or a third-party dependency to reach the same place |
| R11 | `!uninstfinalize` — a compile-time hook over the generated uninstaller. No side files, no prompt, nothing to preserve between builds | `SignedUninstaller=yes` + `SignTool`. Works, but it writes a **persistent** uninstaller copy into `SignedUninstallerDir` whose signature is reused on later compiles, and it prompts on first compile unless the sign tool is configured on the command line | **NSIS** | **Correction.** Earlier in this week's work Inno's native uninstaller signing was listed as an advantage over NSIS. It is native, but for an automated pipeline it is the more awkward of the two — there is a stateful artefact to manage that NSIS does not have |
| R12 | **CVE-2023-37378** — uninstaller temporary directory writable by all users, DLL plant, local privilege escalation. CVSS 7.8. Affects ≤ 3.08, fixed in 3.09. NSIS 3.12 fixed a further elevated-temp-directory escalation | **CVE-2025-15595** — DLL hijacking privilege escalation during installer execution. CVSS 7.8. Affects ≤ 6.2.1. Inno 7 enables **RedirectionGuard** by default and adds ECDSA P-256 integrity verification | **Tie — and this is the most useful row in the table** | Both tools shipped a 7.8 local privilege escalation of the same class within two years. Neither is "the safe one." What matters is the version and whether anything in CI enforces a floor |

## Scoring, honestly

Counting rows gives Inno Setup four wins, NSIS three, and five ties — which would say switch, and
would be the wrong conclusion, because the rows are not equal weight.

**The three NSIS rows are the ones tied to how this repository already works:**

- **R2** is a correctness requirement, not a preference. The daemon holds a global keyboard hook
  and a tray icon; being force-terminated by Restart Manager leaves both to be cleaned up by
  Windows rather than by the code written to clean them up. Inno *can* match this with `[Code]`
  calling the same Win32 functions, but then the argument for switching — less code to own —
  has evaporated for the one part of the script that matters most.
- **R10** is the CI convention this repo has already written down and justified in a comment.
  Meeting it with NSIS is `curl` plus `Get-FileHash`. Meeting it with Inno means installing
  software onto the runner or taking a third-party action as a dependency.
- **R11** matters at the moment the certificate arrives, which is a known near-term step.

**The four Inno rows are all readability and defaults**, and they are real. `[UninstallRun]`,
`UsePreviousAppDir`, `ArchitecturesAllowed` and better-documented silent switches would each be
one fewer thing to get right by hand. That is exactly the axis on which NSIS scripts decay: ours
is already about 250 lines with a macro that has to be instantiated twice because NSIS compiles
installer and uninstaller functions separately.

## What would flip this

Written down now so the decision can be re-made on evidence rather than on fatigue.

1. **The script crosses roughly 400 lines, or someone has to re-read the whole thing to make a
   one-line change.** Readability is Inno's strongest advantage and the only reason it would
   have been the better first choice.
2. **A second installer behaviour appears** — a service, a driver, a firewall rule, a
   prerequisite chain. Each of those is declarative in Inno and hand-rolled in NSIS.
3. **Localisation of the installer UI becomes a requirement.** Inno's language files are
   markedly less work.
4. **NSIS stops being maintained.** Its release cadence is slow by comparison — 3.12 in April
   2026 against Inno's 7.1.0 in August 2026 — and slow is fine until it becomes stopped.

None of those is true today. If any becomes true, the migration is a rewrite of one file with
no user-visible change beyond the wizard's appearance, which is the cheapest kind of migration
there is — provided it happens **before** there are installed users to upgrade, because
switching installer engines after release means the old uninstaller and the new installer do not
know about each other.

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
