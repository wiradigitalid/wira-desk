; Wira Desk installer — NSIS.
;
; Build:
;   makensis /DVERSION=0.1.0 /DSTAGE_DIR=<abs path to staged files> packaging\wiradesk.nsi
;
; VERSION and STAGE_DIR are supplied by the workflow so that the crate version stays the
; single source of truth, and both are required — see the note above their definitions.
; STAGE_DIR holds everything the installer packages: the two executables, LICENSE, NOTICE,
; and the icon.
;
; ---------------------------------------------------------------------------------
; Why a hand-written script rather than a generator
;
; `cargo-packager` was the first choice — Rust-native, generates NSIS, and would have been
; less to own. It was rejected on one specific gap: its `NsisConfig` exposes
; `preinstall_section` but has no hook that runs during UNINSTALL. This application
; registers a Windows Scheduled Task that launches it ELEVATED AT EVERY LOGON WITH NO UAC
; PROMPT, and an uninstall that leaves that task behind leaves an elevated auto-start entry
; pointing at files that no longer exist. Removing it is not a nicety; it is the one thing
; the uninstaller most has to get right. Reaching it through `cargo-packager` would have
; meant supplying a full custom `.nsi` template anyway — at which point the generator adds a
; version-coupled dependency and gives nothing back.
;
; This application is two executables and a licence file. The script below is the whole
; packaging story, and it is shorter than the configuration would have been.
;
; ---------------------------------------------------------------------------------
; What this installer deliberately does NOT do
;
; * It does NOT create the auto-start scheduled task. The daemon owns that task
;   (`crates/daemon/src/autostart.rs`, business rule BR-4) and it is registered only when
;   the user asks for it. An installer that silently created an unprompted elevated logon
;   task would be making a security decision that belongs to the user.
;
; * It offers NO per-user install location. That is not an omission — `%ProgramFiles%` is
;   the point. The logon task runs the daemon elevated with no prompt, so anyone who can
;   overwrite the executable gains an unprompted elevated foothold; the file permissions
;   are the whole defence. An installer offering `%LOCALAPPDATA%` would be offering a
;   privilege-escalation route as a convenience. See `SECURITY.md`.
;
; * It does NOT delete `%APPDATA%\WiraDesk\` on uninstall, and this is deliberate twice
;   over. First, the uninstaller runs elevated, so `$APPDATA` resolves to the ADMINISTRATOR
;   who is uninstalling — which need not be the user whose settings those are. Deleting the
;   wrong profile's data is worse than leaving data behind. Second, that directory carries
;   migration semantics documented in `README.md`: removing it does not reset settings, it
;   re-imports from a legacy WinTick install. The uninstaller says where the folder is and
;   leaves the decision to a human.

Unicode true
ManifestDPIAware true

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "FileFunc.nsh"
!include "WinMessages.nsh"

; VERSION and STAGE_DIR are required rather than defaulted. A default for either would
; compile happily and produce a WRONG installer -- one labelled 0.0.0, or one built from a
; stale staging directory -- and an installer that is quietly wrong is worse than one that
; refuses to build. OUT_DIR defaults to the working directory, because being wrong about
; where the output landed costs nothing.
;
; STAGE_DIR must be ABSOLUTE, and every asset below is read from it so that this script
; holds no path into the repository at all. NSIS resolves a relative path against the
; directory holding the script, and the __FILEDIR__ variable is itself only as absolute as
; the path makensis was invoked with -- so a repo-relative asset path compiles when the
; script is named absolutely and fails when it is named relatively, which is exactly how
; the workflow invokes it.
!ifndef VERSION
  !error "VERSION is required: makensis /DVERSION=<x.y.z> ..."
!endif
!ifndef STAGE_DIR
  !error "STAGE_DIR is required and must be absolute: makensis /DSTAGE_DIR=<abs path> ..."
!endif
!ifndef OUT_DIR
  !define OUT_DIR "."
!endif

!define APP_NAME     "Wira Desk"
!define PUBLISHER    "Wira Digital Indonesia"
!define HOMEPAGE     "https://github.com/kodesh87/wira-desk"

; These four must match `crates/shared/src/constants.rs`. They are duplicated here because
; NSIS cannot read Rust, and they are load-bearing: the window class and title are how the
; installer asks a running daemon to shut down before its file is replaced, and the task
; name is what the uninstaller removes.
!define DAEMON_EXE   "wiradesk.exe"
!define SETTINGS_EXE "wiradesk-settings.exe"
!define DAEMON_CLASS "WiraDeskDaemonHiddenWindow"
!define DAEMON_TITLE "WiraDeskDaemon"
!define TASK_NAME    "WiraDesk"

!define UNINST_KEY   "Software\Microsoft\Windows\CurrentVersion\Uninstall\WiraDesk"

Name "${APP_NAME}"
OutFile "${OUT_DIR}\WiraDesk-${VERSION}-x64-setup.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"

; An upgrade must land on top of the existing install, never beside it. The registered
; logon task stores an absolute path, so a second copy at a different directory would leave
; Windows launching the old one at logon.
InstallDirRegKey HKLM "${UNINST_KEY}" "InstallLocation"

RequestExecutionLevel admin
SetCompressor /SOLID lzma

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName"     "${APP_NAME}"
VIAddVersionKey "CompanyName"     "${PUBLISHER}"
VIAddVersionKey "LegalCopyright"  "Copyright (c) 2026 ${PUBLISHER}"
VIAddVersionKey "FileDescription" "${APP_NAME} Setup"
VIAddVersionKey "FileVersion"     "${VERSION}"
VIAddVersionKey "ProductVersion"  "${VERSION}"

!define MUI_ABORTWARNING
!define MUI_ICON "${STAGE_DIR}\wiradesk.ico"
!define MUI_UNICON "${STAGE_DIR}\wiradesk.ico"

!insertmacro MUI_PAGE_LICENSE "${STAGE_DIR}\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\${DAEMON_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Start Wira Desk now"
!define MUI_FINISHPAGE_LINK "Wira Desk on GitHub"
!define MUI_FINISHPAGE_LINK_LOCATION "${HOMEPAGE}"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; ---------------------------------------------------------------------------------
; Ask a running daemon to exit, and wait for it.
;
; Replacing a running executable fails on Windows, so this has to succeed before any file
; is written. It is a real shutdown request rather than a kill: `WM_CLOSE` on the daemon's
; hidden window reaches `DefWindowProc`, which calls `DestroyWindow`, which runs the
; daemon's `WM_DESTROY` arm — unhooking `WH_KEYBOARD_LL` and releasing the tray icon before
; the process exits. `taskkill /F` would skip all of that and orphan the tray icon until
; Explorer next refreshed. It is kept only as the last resort after six seconds.
;
; The installer runs elevated and so does the daemon, so both sit at the same integrity
; level and UIPI does not block the message.
;
; Defined as a macro because NSIS compiles installer and uninstaller functions separately;
; the uninstaller needs the identical routine under the mandatory `un.` prefix.
!macro StopDaemonMacro un
Function ${un}StopDaemon
  Push $0
  Push $1
  StrCpy $1 0

  ${Do}
    FindWindow $0 "${DAEMON_CLASS}" "${DAEMON_TITLE}"
    ${If} $0 == 0
      ${ExitDo}
    ${EndIf}
    ${If} $1 >= 24
      DetailPrint "Daemon did not exit within six seconds; forcing."
      nsExec::ExecToLog 'taskkill /F /IM "${DAEMON_EXE}"'
      Pop $0
      Sleep 500
      ${ExitDo}
    ${EndIf}
    SendMessage $0 ${WM_CLOSE} 0 0 /TIMEOUT=3000
    Sleep 250
    IntOp $1 $1 + 1
  ${Loop}

  ; The settings window is bound to the daemon's lifetime and should already have gone.
  ; Asked politely and without `/F` so an unsaved edit is not destroyed; if it is already
  ; closed this is a no-op that reports failure, which is why the result is discarded.
  nsExec::ExecToLog 'taskkill /IM "${SETTINGS_EXE}"'
  Pop $0

  Pop $1
  Pop $0
FunctionEnd
!macroend
!insertmacro StopDaemonMacro ""
!insertmacro StopDaemonMacro "un."

Function .onInit
  ; The workspace builds for x86_64 only. Refusing here is friendlier than installing a
  ; binary that cannot start.
  ${IfNot} ${RunningX64}
    ${IfNot} ${Silent}
      MessageBox MB_ICONSTOP|MB_OK "Wira Desk requires 64-bit Windows."
    ${EndIf}
    SetErrorLevel 1
    Abort
  ${EndIf}
  ; Both the uninstall entry and `$PROGRAMFILES64` belong in the 64-bit registry view.
  SetRegView 64
FunctionEnd

Function un.onInit
  SetRegView 64
FunctionEnd

Section "Wira Desk" SecMain
  SectionIn RO

  Call StopDaemon

  SetOutPath "$INSTDIR"
  File "${STAGE_DIR}\${DAEMON_EXE}"
  File "${STAGE_DIR}\${SETTINGS_EXE}"
  File "/oname=LICENSE.txt" "${STAGE_DIR}\LICENSE"
  File "/oname=NOTICE.txt" "${STAGE_DIR}\NOTICE"

  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; The daemon is the entry point; the settings window is reached from its tray menu, and
  ; on its own it would exit immediately because its lifetime is bound to the daemon's.
  CreateShortcut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\${DAEMON_EXE}"

  WriteRegStr   HKLM "${UNINST_KEY}" "DisplayName"     "${APP_NAME}"
  WriteRegStr   HKLM "${UNINST_KEY}" "DisplayVersion"  "${VERSION}"
  WriteRegStr   HKLM "${UNINST_KEY}" "Publisher"       "${PUBLISHER}"
  WriteRegStr   HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr   HKLM "${UNINST_KEY}" "DisplayIcon"     "$INSTDIR\${DAEMON_EXE}"
  WriteRegStr   HKLM "${UNINST_KEY}" "URLInfoAbout"    "${HOMEPAGE}"
  WriteRegStr   HKLM "${UNINST_KEY}" "HelpLink"        "${HOMEPAGE}"
  WriteRegStr   HKLM "${UNINST_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  ; winget uninstalls non-interactively and reads this key to do it.
  WriteRegStr   HKLM "${UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoRepair" 1

  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "${UNINST_KEY}" "EstimatedSize" "$0"
SectionEnd

Section "Uninstall"
  Call un.StopDaemon

  ; Before the files go. An `ONLOGON` `/RL HIGHEST` task outliving the executable it names
  ; is the single worst thing this uninstaller could leave behind, so it is removed first
  ; and its failure is not allowed to matter — the task may simply never have been created,
  ; which `schtasks` reports as an error.
  DetailPrint "Removing the auto-start scheduled task, if one is registered."
  nsExec::ExecToLog 'schtasks /Delete /TN "${TASK_NAME}" /F'
  Pop $0

  Delete "$INSTDIR\${DAEMON_EXE}"
  Delete "$INSTDIR\${SETTINGS_EXE}"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\NOTICE.txt"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  DeleteRegKey HKLM "${UNINST_KEY}"

  ; Settings and logs are left on disk on purpose — see the header. Said out loud rather
  ; than left for the user to discover, and only when a human is watching.
  ${IfNot} ${Silent}
    MessageBox MB_ICONINFORMATION|MB_OK "Wira Desk has been removed.$\r$\n$\r$\nYour settings and log were left in %APPDATA%\WiraDesk. Delete that folder by hand if you want them gone."
  ${EndIf}
SectionEnd
