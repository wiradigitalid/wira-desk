; Wira Desk installer — Inno Setup.
;
; ============================================================================
; THIS IS THE INSTALLER. There is no second one: `packaging/wiradesk.nsi` was
; deleted when this file won, because keeping both is how a repository ends up
; shipping the one nobody has looked at in a year.
;
; The comparison that chose it is `docs/packaging-choice.md`, and the deciding
; reason is the `AppVersion` line below: the version is read out of the built
; binary, so the installer cannot disagree about its version with the program it
; installs. Two more rows followed from the same principle — `AppId` handles
; upgrade-in-place, and `[UninstallRun]` removes the scheduled task in one
; declarative line rather than a plugin call whose return value must be popped
; off a stack by hand.
;
; The decision was taken before the first tag on purpose. After a release, the
; old uninstaller and a new installer do not know about each other, and a user
; who upgrades ends up with two Add/Remove Programs entries, one of them broken.
; ============================================================================
;
; TWO INSTALL CHANNELS, ONE UPDATE MECHANISM, AND WHAT THAT DEMANDS OF THIS FILE.
;
; This installer is run three ways, and only the first shows a wizard:
;   1. A person double-clicks it.
;   2. winget runs it silently, for `winget install` and `winget upgrade`.
;   3. The application's own updater runs it silently, after downloading and
;      verifying it. That is the update path users are pointed at; winget is a
;      way to install, never a command a user is asked to type.
;
; So the silent path is not a corner case here — it is the path that carries
; every update. Anything that only works on the wizard path is broken for
; everyone who updates. See the `[Run]` section, which is exactly that defect.
;
; Build:
;   ISCC.exe /DSTAGE_DIR=<abs path to staged files> /DOUT_DIR=<abs path> packaging\wiradesk.iss
;
; TOOLCHAIN LICENCE, AND ONE OBLIGATION THAT IS ALREADY MET.
;
; Inno Setup's licence grants permission "to anyone to use this software for any
; purpose, including commercial applications". The "Non-commercial use only" line
; the compiler prints is a request to buy a commercial licence — added in 6.5.0 —
; and not a restriction the licence imposes. It is also compiler-only: it does not
; appear anywhere in the installer this file produces, which was checked by
; searching the compiled binary rather than assumed.
;
; The condition that does bind us is that binary redistributions retain the
; copyright notice and web addresses already in place. It is satisfied by
; construction: the produced Setup carries "This installation was built with Inno
; Setup." and a `jrsoftware.org` URL, both emitted by the engine. Nothing here
; strips them, and nothing should — that would be the one way to fall out of
; compliance, and it would look like tidying up.
;
; Pinned to the Inno Setup **6.x** line rather than 7.x on purpose. 6.7.3
; (2026-05-26) is mature and well past CVE-2025-15595, which affected 6.2.1 and
; earlier. Inno Setup 7.1.0 landed on 2026-08-12 — two weeks before this was
; written — and carries breaking changes (the `x64`/`x86` architecture
; identifiers deprecated, Pascal scripting removals, 64-bit installers). None of
; its new capabilities are needed here, so the newer line buys risk and nothing
; else. Revisit when 7.x has a few point releases behind it.

#ifndef STAGE_DIR
  #error STAGE_DIR is required and must be absolute: ISCC /DSTAGE_DIR=<abs path> ...
#endif
#ifndef OUT_DIR
  #define OUT_DIR "."
#endif

; The version is read out of the built executable's own version resource, which
; `crates/daemon/build.rs` fills from the crate version. That makes the binary
; the single source of truth and removes a whole class of mismatch: the
; installer cannot disagree with the program it installs, because it is asking
; the program. Note what this does NOT remove: the release workflow still checks
; that the git tag agrees with the crate version, because the tag names the
; GitHub release and could still contradict the binary inside it. What went away
; is having to pass the version in, not having to agree about it.
#define DaemonExe "wiradesk.exe"
#define SettingsExe "wiradesk-settings.exe"
#define AppVersion GetStringFileInfo(STAGE_DIR + "\" + DaemonExe, "FileVersion")
#define AppName "Wira Desk"
#define Publisher "Wira Digital Indonesia"
#define Homepage "https://github.com/kodesh87/wira-desk"

; Must match `crates/shared/src/constants.rs`. The window class is how a running
; daemon is found and asked to shut down; the task name is what uninstall removes.
#define DaemonWindowClass "WiraDeskDaemonHiddenWindow"
#define TaskName "WiraDesk"

[Setup]
; A fixed identity, and the reason upgrades work without being written: Inno
; matches AppId against the existing uninstall log, reuses the previous install
; directory, and keeps one uninstall entry instead of accumulating them.
;
; It also carries the whole two-channel story. winget reads the installed version
; from the Add/Remove Programs entry this AppId owns, and so does the in-app
; updater — one entry, one source of truth, so the two channels cannot disagree
; about what is installed. Changing this GUID orphans every existing install.
AppId={{7E4F9C21-6B3D-4A88-9F14-2C5E8D0A1B73}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#Publisher}
AppPublisherURL={#Homepage}
AppSupportURL={#Homepage}
AppUpdatesURL={#Homepage}
VersionInfoVersion={#AppVersion}
VersionInfoCompany={#Publisher}
VersionInfoDescription={#AppName} Setup

; Per-machine, and no per-user option. `PrivilegesRequiredOverridesAllowed` is
; deliberately NOT set: setting it would give the user a per-user install choice,
; and a user-writable directory holding an executable that Windows launches
; elevated at every logon with no prompt is a privilege-escalation route. See
; SECURITY.md. This is the one line in this file where a convenience directive
; would be a security regression.
PrivilegesRequired=admin
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UsePreviousAppDir=yes

; x64 only — the workspace targets x86_64-pc-windows-msvc and nothing else.
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

LicenseFile={#STAGE_DIR}\LICENSE
SetupIconFile={#STAGE_DIR}\wiradesk.ico
UninstallDisplayIcon={app}\{#DaemonExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
OutputDir={#OUT_DIR}
; The `-x64-setup` ending is load-bearing: `release.yml` hands winget an
; `installers-regex` of `-x64-setup\.exe$` so that the loose binaries published
; beside this file cannot be mistaken for installers. Renaming this breaks the
; winget channel silently — the manifest job simply matches nothing.
OutputBaseFilename=WiraDesk-{#AppVersion}-x64-setup

; Restart Manager is switched OFF on purpose, and this is a deliberate rejection
; of Inno's default rather than an oversight.
;
; `CloseApplications=yes` would let Windows Restart Manager decide HOW to close
; the running daemon. For a hidden WS_EX_TOOLWINDOW window with no
; WM_QUERYENDSESSION handler, the likely outcome is force-termination after a
; timeout — which skips the daemon's WM_DESTROY arm, the one place that unhooks
; WH_KEYBOARD_LL and removes the tray icon. `AppMutex` is not an alternative
; either: it only warns the user and waits for them to click OK, which is useless
; under `/VERYSILENT` where winget drives the install.
;
; So the shutdown is done explicitly in [Code] below. This is the one requirement
; Inno cannot meet declaratively, and it was known before choosing it — the choice
; was made on the version-from-binary property, not on this row.
CloseApplications=no

; When a certificate exists, signing is two directives plus a tool configured
; with ISCC's `--signtool`. Note the wrinkle: SignedUninstaller writes a
; persistent signed uninstaller into SignedUninstallerDir whose signature is
; reused on later compiles, so that directory becomes build state to preserve.
; NSIS's `!uninstfinalize` has no such artefact.
;   SignTool=mysigntool
;   SignedUninstaller=yes

[Files]
Source: "{#STAGE_DIR}\{#DaemonExe}";   DestDir: "{app}"; Flags: ignoreversion
Source: "{#STAGE_DIR}\{#SettingsExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#STAGE_DIR}\LICENSE";        DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "{#STAGE_DIR}\NOTICE";         DestDir: "{app}"; DestName: "NOTICE.txt";  Flags: ignoreversion

[Icons]
; The daemon is the entry point. The settings window is reached from its tray
; menu and would exit immediately on its own, because its lifetime is bound to
; the daemon's — so it gets no shortcut.
Name: "{group}\{#AppName}"; Filename: "{app}\{#DaemonExe}"

[Run]
; TWO ENTRIES, AND THE SECOND ONE IS A BUG FIX RATHER THAN A FEATURE.
;
; The first is the finish-page checkbox: `postinstall` makes it a checkbox rather
; than an unconditional launch, `nowait` so Setup does not sit waiting on a tray
; daemon, and `skipifsilent` because a silent install has no finish page to put a
; checkbox on.
;
; That flag is correct and it was also the whole defect. With only this entry,
; every silent install ends with the daemon not running — which is `winget
; install`, `winget upgrade`, and every application-driven update. The visible
; symptom is the worst kind: the user presses Update, watches a progress bar
; complete, and the application is simply gone. Nothing errors, and no manual
; test finds it, because a person testing by hand always takes the wizard path.
;
; So the second entry starts the daemon on exactly the path the first one skips.
; It is not a duplicate: the two `Check`/`Flags` conditions are complements, so
; precisely one of them runs. Even if both somehow fired, the daemon holds a
; single-instance mutex and the second process would exit immediately — but that
; is a backstop, not the design.
; `runascurrentuser` IS ON BOTH ENTRIES AND MUST STAY. It is the fix for a real failure,
; and it is written explicitly on both rather than left to a default, because the default
; is what caused the failure — it flips depending on whether `postinstall` is present:
;
;   without postinstall -> runascurrentuser  (inherits Setup's elevated token)
;   with    postinstall -> runasoriginaluser (the user's NON-elevated token)
;
; So the finish-page checkbox launched the daemon unelevated. The daemon's manifest
; demands requireAdministrator, and `CreateProcess` — which is what [Run] uses — cannot
; elevate; only ShellExecuteEx can. The result was not a UAC prompt but a hard failure:
;
;   Unable to execute file ... CreateProcess failed; code 740.
;   The requested operation requires elevation.
;
; Note which path broke. The silent entry has no `postinstall`, so it was already correct;
; the wizard path, the one a person actually takes, was the broken one. `shellexec` would
; also work by routing through ShellExecuteEx, but it would put a SECOND UAC prompt in
; front of a user who just consented to Setup's. Inheriting the token Setup already holds
; costs no prompt at all.
Filename: "{app}\{#DaemonExe}"; Description: "Start {#AppName} now"; Flags: postinstall nowait skipifsilent runascurrentuser
Filename: "{app}\{#DaemonExe}"; Flags: nowait runascurrentuser; Check: RunningSilently

[UninstallRun]
; The highest-consequence line in either installer script. An ONLOGON task with
; /RL HIGHEST that outlives the executable it names is the worst residue this
; uninstaller could leave. [UninstallRun] executes before files are removed, and
; Inno does not treat a non-zero exit code as an error — which is what we want,
; because the task may never have been registered and schtasks reports that as
; a failure.
;
; This is the row where Inno is plainly nicer: one declarative line against the
; NSIS equivalent, which is an nsExec plugin call whose return value must be
; popped off the stack or it corrupts the next one.
; `RunOnceId` is not decoration: without it the compiler warns, and the entry can
; execute more than once across repeated uninstall attempts. Harmless for an
; idempotent `schtasks /Delete`, but the warning is right and worth not carrying.
Filename: "{sys}\schtasks.exe"; Parameters: "/Delete /TN ""{#TaskName}"" /F"; Flags: runhidden; RunOnceId: "DeleteAutoStartTask"

; NOTE, and it is deliberate: there is no [UninstallDelete] for
; %APPDATA%\WiraDesk. An elevated uninstaller's {userappdata} resolves to the
; ADMINISTRATOR running it, who need not be the user whose settings those are —
; so deleting is as likely to remove the wrong profile's data as the right one.
; That folder also carries migration semantics documented in README.md: removing
; it does not reset settings, it re-imports from a legacy install. The user is
; told where it is and left to decide.

[Code]
const
  WM_CLOSE = $0010;
  StopPollIntervalMs = 250;
  StopMaxPolls = 24; { six seconds before forcing }

{ True when Setup was started with /SILENT or /VERYSILENT, which is every
  automated path: winget, and the application's own updater.

  A wrapper rather than putting `WizardSilent` straight into a `Check:`
  parameter, so that the [Run] entry reads as a stated intent and so there is one
  place to change if the condition ever needs to be narrower. }
function RunningSilently: Boolean;
begin
  Result := WizardSilent;
end;

{ Ask a running daemon to exit, and wait for it.

  Replacing a running executable fails on Windows, so this must succeed before
  any file is written. It is a request rather than a kill: WM_CLOSE on the
  daemon's hidden window reaches DefWindowProc, which calls DestroyWindow, which
  runs the daemon's WM_DESTROY arm — unhooking WH_KEYBOARD_LL and releasing the
  tray icon before the process exits. taskkill /F skips all of that and orphans
  the tray icon until Explorer next refreshes, so it is the last resort only.

  Setup runs elevated and so does the daemon, so both sit at the same integrity
  level and UIPI does not block the message. }
procedure StopDaemon;
var
  Wnd: HWND;
  Polls: Integer;
  ResultCode: Integer;
begin
  Polls := 0;
  while True do
  begin
    Wnd := FindWindowByClassName('{#DaemonWindowClass}');
    if Wnd = 0 then
      Break;

    if Polls >= StopMaxPolls then
    begin
      Log('Daemon did not exit within six seconds; forcing.');
      Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM "{#DaemonExe}"',
           '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
      Sleep(500);
      Break;
    end;

    PostMessage(Wnd, WM_CLOSE, 0, 0);
    Sleep(StopPollIntervalMs);
    Polls := Polls + 1;
  end;

  { The settings window is bound to the daemon's lifetime and should already be
    gone. Asked without /F so an unsaved edit is not destroyed; a failure here
    just means it had already closed, which is why the code is discarded.

    DO NOT REMOVE THIS TO FIX AN UPDATER THAT DIES MID-UPDATE. It has to stay:
    `[Files]` replaces `wiradesk-settings.exe`, and Windows cannot replace a
    running image. The obligation is the other way round — the updater lives
    inside Settings, so it MUST launch Setup detached and then exit, rather than
    waiting on a process whose first act is to kill it. That contract is the
    updater's to keep, and this comment exists because the tempting fix is here. }
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/IM "{#SettingsExe}"',
       '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

{ Runs after the wizard and before any file is installed — the correct hook for
  this, because it must happen before the executable is replaced. }
function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  StopDaemon;
  Result := '';
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir: String;
begin
  if CurUninstallStep = usUninstall then
    StopDaemon
  else if CurUninstallStep = usPostUninstall then
  begin
    // OFFER THE DELETION RATHER THAN DESCRIBING IT.
    //
    // These are `//` comments rather than the brace form used elsewhere in this file,
    // and deliberately: Inno's Pascal comments are delimited by `{` and `}`, so naming a
    // constant like {userappdata} inside one closes the comment early and the prose after
    // it is parsed as code. The compiler said "Identifier expected" at the closing brace
    // and was right. Anything discussing Inno constants belongs in a `//` comment.
    //
    // This used to print an unconditional notice saying the folder had been left behind
    // and the user could delete it by hand. Two things were wrong. It fired even when the
    // folder did not exist, telling people about nothing. And it arrived after the
    // decision, when the only available action was to go and find a directory in
    // %APPDATA% themselves -- which almost nobody does, so an honest description of that
    // behaviour is "we leave files on your machine and mention it".
    //
    // Deleting is now offered, only when there is something to delete, and No is the
    // default so the destructive answer is never what a hurried Enter selects.
    //
    // Why this is safe to offer, having previously been argued against here. The concern
    // was that an elevated uninstaller resolves the AppData constant against whoever is
    // running it, who need not be the user those settings belong to. That is true, and
    // the consequence is milder than it first looked: the wrong-profile case deletes the
    // RUNNING administrator's own folder, which is either absent or theirs to lose, and
    // it cannot reach another user's profile. The second objection -- that the folder
    // carries migration semantics, since its absence re-imports from a legacy
    // %APPDATA%\WinTick -- no longer applies to anyone but this project's own maintainer,
    // because WinTick was never publicly released and so no user has one.
    //
    // Silent uninstall never prompts and never deletes. Package managers uninstall
    // unattended, and destroying user data with nobody present to consent is not a
    // default any of them asked for.
    DataDir := ExpandConstant('{userappdata}\WiraDesk');

    if UninstallSilent then
      Exit;

    if not DirExists(DataDir) then
    begin
      MsgBox('{#AppName} has been removed.', mbInformation, MB_OK);
      Exit;
    end;

    if MsgBox('{#AppName} has been removed.' + #13#10 + #13#10 +
              'Also delete your settings and log?' + #13#10 + #13#10 +
              DataDir + #13#10 + #13#10 +
              'Choose No to keep them, which is what you want if you plan to ' +
              'reinstall.', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
    begin
      { True/True/True: delete the directory itself, its files, and its subdirectories.
        A failure is deliberately not reported. The program is already gone, the user has
        been told the path, and an error dialog at this point would be the last thing
        they see from a product they just removed. }
      DelTree(DataDir, True, True, True);
    end;
  end;
end;
