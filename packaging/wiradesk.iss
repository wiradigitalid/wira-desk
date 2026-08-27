; Wira Desk installer — Inno Setup.
;
; ============================================================================
; THIS IS AN EVALUATION PROTOTYPE. IT IS NOT BUILT BY CI AND NOTHING SHIPS FROM
; IT. `packaging/wiradesk.nsi` is the installer this product currently uses.
;
; It exists so the choice recorded in `docs/packaging-choice.md` can be compared
; against a working equivalent rather than against documentation. It implements
; the same eight requirements as the NSIS script, deliberately including the
; awkward one (graceful daemon shutdown), because a prototype that skipped the
; hard part would flatter Inno Setup rather than test it.
;
; Exactly one of these two files should survive the decision. Keeping both is
; how a repository ends up shipping the one nobody has looked at in a year.
; ============================================================================
;
; Build:
;   ISCC.exe /DSTAGE_DIR=<abs path to staged files> /DOUT_DIR=<abs path> packaging\wiradesk.iss
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
; the program. The NSIS script takes the version as a command-line define
; instead, which works but is one more thing for a release workflow to get right.
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
; directory, and keeps one uninstall entry instead of accumulating them. The NSIS
; script reaches the same place through an explicit `InstallDirRegKey`.
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
OutputBaseFilename=WiraDesk-{#AppVersion}-x64-setup-inno

; Restart Manager is switched OFF on purpose, and this is the substantive
; difference from the NSIS script rather than a style choice.
;
; `CloseApplications=yes` would let Windows Restart Manager decide HOW to close
; the running daemon. For a hidden WS_EX_TOOLWINDOW window with no
; WM_QUERYENDSESSION handler, the likely outcome is force-termination after a
; timeout — which skips the daemon's WM_DESTROY arm, the one place that unhooks
; WH_KEYBOARD_LL and removes the tray icon. `AppMutex` is not an alternative
; either: it only warns the user and waits for them to click OK, which is useless
; under `/VERYSILENT` where winget drives the install.
;
; So the shutdown is done explicitly in [Code] below, exactly as the NSIS script
; does it. Inno CAN match the requirement — it just cannot match it declaratively,
; which is worth knowing before choosing it for this reason.
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
; The finish-page checkbox. `postinstall` makes it a checkbox rather than an
; unconditional launch; `nowait` so Setup does not sit waiting on a tray daemon.
Filename: "{app}\{#DaemonExe}"; Description: "Start {#AppName} now"; Flags: postinstall nowait skipifsilent

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
    just means it had already closed, which is why the code is discarded. }
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
begin
  if CurUninstallStep = usUninstall then
    StopDaemon
  else if CurUninstallStep = usPostUninstall then
  begin
    if not UninstallSilent then
      MsgBox('{#AppName} has been removed.' + #13#10 + #13#10 +
             'Your settings and log were left in %APPDATA%\WiraDesk. Delete that ' +
             'folder by hand if you want them gone.', mbInformation, MB_OK);
  end;
end;
