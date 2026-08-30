---
type: sdd
component: settings
status: reviewed
created: 2026-08-21
updated: 2026-08-30
realizes: [UC-4, UC-5, UC-6]
binds: [AD-1, AD-5, AD-11, AD-11a, AD-12, AD-13]
reviewed:
  date: '2026-08-26'
  sha: '79277c2'
  lenses: [structure, prose, edge-case-hunter]
---

# SDD — settings

## Decision Summary

The `settings` component is built as an on-demand, episodic graphical configuration binary (`wiradesk-settings.exe`) isolated from the core background service. It is structured into three logical components: `LC-settings-shell` (rendering, theme detection, and navigation), `LC-config-writer` (staged validation, atomic persistence, Task Scheduler auto-start, and daemon IPC signalling), and `LC-shortcut-capturer` (live physical key capture and accessible announcement). It delivers an instant, accessible, keyboard-navigable configuration interface with zero background RAM footprint when closed and full assistive technology support via AccessKit.

The two most expensive architectural decisions reversed from naive desktop utility designs are:
1. **Out-of-process episodic UI lifecycle over integrated in-daemon rendering (AD-1, AD-11):** Embedding a UI toolkit (e.g. `slint`, WebView2, or WinUI) directly into the background daemon permanently inflates static memory footprint (>20–50 MB RAM), links heavyweight graphics/input runtimes, and risks stalling low-level keyboard hook threads during redraws. Packaging Settings as a standalone binary launched on demand via `ShellExecute` keeps the background daemon under 2 MB static RAM and isolates UI crashes completely from window cycling.
2. **Staged validation and atomic replace-before-signal IPC over file-watcher polling (AD-5, AD-12):** File system watchers (`ReadDirectoryChangesW`) suffer from partial write races, false triggers during temporary file creation, and unnecessary background thread wake-ups. The settings binary stages user edits in memory, validates every shortcut chord against canonical grammar before disk writes, executes an atomic temporary file rename (`config.toml.tmp` → `config.toml`), and emits an explicit one-way Win32 `WM_APP_RELOAD_CONFIG` message to the daemon's hidden window. The daemon reloads only on explicit signal, eliminating read races and file-polling overhead entirely.

## Structure

The three Logical Components (LCs) operate strictly within the `settings` container (`wiradesk-settings.exe`), consuming shared types and IPC constants from the `shared` crate.

| LC | type | Responsibility |
| --- | --- | --- |
| `LC-settings-shell` | ui-composite | Hosts the retained-mode `Slint` window (`ui/main_window.slint`, compiled by `slint-build`) and its pane components; manages frameless window shell (`no-frame: true`) and 5-pane routing (`General`, `Shortcuts`, `Layout & Snapping`, `VM & Exceptions`, `About`); detects OS theme (`AppsUseLightTheme`) once at startup and sets it on the `Palette` global for two-theme visuals with widened focus outlines; implements first-run onboarding wizard progression; renders save feedback and diagnostic typeface information. |
| `LC-config-writer` | service | Executes strict pre-persistence shortcut validation; performs atomic file writes (`Config::save`) to `%APPDATA%\WiraDesk\config.toml`; dispatches non-blocking `WM_APP_RELOAD_CONFIG` (0x8001) signals via `PostMessageW` to `WiraDeskDaemonHiddenWindow`. It records the auto-start *preference* only; the scheduled task itself is created and deleted by the daemon (`daemon::autostart`) when it reloads config. |
| `LC-shortcut-capturer` | control | Implements interactive key interception within the Settings window; while a field is `Listening`, the daemon's own hook report (drained from a channel by a 20ms Slint timer) is the source of truth for the chord — Slint's `key-pressed` text never arrives for a chord the Windows shell owns, so it survives only as a marked fallback for when no daemon is running (`DEC-004`); enforces modifier requirements; exposes live first-class `Listening` state and screen reader announcements via Slint's accessibility tree. |

### Dependency & Communication Direction

```text
[User Keyboard & Mouse] ──> LC-settings-shell ──> LC-shortcut-capturer
                                 │                       │
                         (staged edits)          (canonical chord)
                                 ▼                       │
                         LC-config-writer <──────────────┘
                                 │
                 ┌───────────────┴────────────────┐
                 ▼                                ▼
       [Atomic File Write]              [Win32 PostMessageW]
    %APPDATA%\WiraDesk\config.toml    (WM_APP_RELOAD_CONFIG: 0x8001)
                 │                                │
                 │ (re-reads on signal)           ▼
                 └───────────────────> [Daemon Hidden Window]
                                    (WiraDeskDaemonHiddenWindow)

[Windows Task Scheduler] <──(schtasks.exe)── daemon::autostart  (NOT settings)
[Windows Registry]       <──(RegGetValueW)─── LC-settings-shell (Theme)
[Windows Fonts]          <──(segoeui.ttf)──── LC-settings-shell (Typography)
[Windows UI Automation]  <──(AccessKit)────── LC-settings-shell (Assistive Tech)
```

- `LC-settings-shell` owns the mutable in-memory `SettingsModel` draft. Edits never touch the disk until explicitly saved.
- `LC-shortcut-capturer` interacts with `LC-settings-shell` to capture raw key events when activated for a specific `ShortcutField`. Cancelling (Escape) or navigating panes immediately restores `CaptureState::Idle`.
- `LC-config-writer` validates all draft fields before performing an atomic temporary-file-and-rename write. Only after the file replacement succeeds does it invoke `signal_reload()`.
- IPC across the process boundary carries no memory pointers (`wParam = 0`, `lParam = 0`). The configuration payload travels exclusively through the completed TOML file on disk.

## Inherited Constraints

The following Architectural Decisions from `ARCHITECTURE-SPINE.md` bind the design and implementation of `settings`:

| AD | Quoted rule | How it lands here |
| --- | --- | --- |
| **AD-1** | "Each actor (hook thread, worker thread, settings process) owns its state exclusively. Cross-actor communication uses only: lock-free ring buffer (hook→worker), Win32 Window Messages (settings→daemon), TOML file (settings→daemon config), ShellExecute (daemon→settings launch)." | Settings is executed as an isolated process. It never shares memory, pointers, or mutexes with the daemon. IPC to daemon uses strictly `PostMessageW(hwnd, WM_APP_RELOAD_CONFIG, 0, 0)`. |
| **AD-5** | "The Settings binary writes `config.toml` to completion atomically via temp file rename, then sends a `WM_APP_RELOAD_CONFIG` Win32 message to the Daemon's hidden window. The Daemon reloads config only upon receiving this message — never via polling or file watching." | `crates/settings/src/persistence.rs` executes `validate_config(&cfg)` → `cfg.save(path)` (atomic temp write + rename) → `signal_reload()` via `FindWindowW` / `PostMessageW`. |
| **AD-11** | "`wiradesk-settings.exe` uses `Slint` for its GUI, with `ui/main_window.slint` compiled by `slint-build` in `build.rs` and rendered through `i-slint-backend-winit`. The Daemon launches it via `ShellExecute` (inheriting Administrator elevation). De-elevation is not required — the settings binary only edits `config.toml` in `%APPDATA%`.<br/>**First run:** when no `config.toml` exists, the Daemon launches the same binary with the frozen `--onboarding` flag (`shared::ONBOARDING_FLAG`). The flag lives in `shared` because both sides use it — a typo must be a compile error, not an onboarding screen that silently never appears." | `crates/settings/build.rs` compiles `ui/main_window.slint` via `slint_build::compile`. `crates/settings/src/main.rs` builds the window with `MainWindow::new()` over `i-slint-backend-winit`. `resolve_launch_intent` evaluates CLI args for `shared::ONBOARDING_FLAG` and checks disk existence via `launch_intent()`. |
| **AD-11a** | "`wiradesk-settings.exe` depends on `slint` with the `accessibility` feature enabled. Slint's own Windows adapter publishes the UI Automation tree — backed by AccessKit internally — and is the accepted accessibility mechanism. Each interactive element states its role and label explicitly via `accessible-role`/`accessible-label` in the `.slint` source, rather than relying on inferred roles.<br/>**Typography:** Segoe UI is loaded from system fonts (`C:\Windows\Fonts\segoeui.ttf`), falling back to Tahoma, then to `LoadedFont::Bundled` — the renderer's own default face — when neither is found.<br/>**Theme:** OS theme detection (`AppsUseLightTheme`) sets `is_dark` on the `Palette` global (`ui/theme.slint`) once at startup; every component reads its colors from that global, including focus-outline color, so the focus indicator stays consistent with the detected OS theme without per-style overrides. Theme is not re-read while the window is open — a mid-session OS theme change takes effect only after a restart." | `Cargo.toml` specifies `slint` with `default-features = false, features = ["backend-winit", "renderer-skia", "accessibility", "compat-1-2"]`. `crates/settings/ui/**/*.slint` declare `accessible-role`/`accessible-label` per element. `crates/settings/src/theme.rs` validates and loads font bytes from `%SystemRoot%\Fonts\segoeui.ttf` via `ttf-parser`. `main.rs` calls `main_window.global::<Palette>().set_is_dark(is_dark)` once, from `theme::detect_theme()`. |
| **AD-12** | "The project is a single Cargo Workspace with three crates: `daemon` (`wiradesk.exe`), `settings` (`wiradesk-settings.exe`), and `shared` (Config TOML types, `u8` command enum, constants, `%APPDATA%` path). Both binaries depend on `shared`." | Settings depends directly on `shared` for `Config`, `Shortcut`, `config_path()`, `migrate_appdata()`, and constants (`WM_APP_RELOAD_CONFIG`, `DAEMON_WINDOW_CLASS`, `DAEMON_WINDOW_TITLE`, `ONBOARDING_FLAG`, `TASK_NAME`). |
| **AD-13** | "Auto-start is registered as a Windows Scheduled Task (`schtasks`): trigger `ONLOGON`, run level `/RL HIGHEST`, run-as user `/RU "%USERNAME%"` (the specific active user, never SYSTEM — keeping `%APPDATA%` aligned between daemon and settings GUI). The task action (`/TR`) must use the absolute executable path and the `Start in` parameter must be left empty or point to the secure install directory, mitigating DLL Hijacking. The registry `Run`-key mechanism (`HKCU\...\CurrentVersion\Run`) is prohibited. Toggle (create/delete task) is exposed via the tray context menu and settings UI." | `General` pane auto-start toggle synchronizes with `shared::Config.general.auto_start` and Task Scheduler task `WiraDesk`. Daemon owns `schtasks` creation/deletion with `/SC ONLOGON /RL HIGHEST /RU "%USERNAME%" /TR "\"<exe_path>\"" /F`. |

## Failure Behaviour

Failure modes across all internal and external Win32 / OS / IPC boundaries:

| Boundary | Slow | Absent | Lying | What the user sees | What is logged |
| --- | --- | --- | --- | --- | --- |
| **Daemon Hidden Window IPC** (`FindWindowW`, `PostMessageW`) | Daemon message queue congested; message handled after brief delay. | Daemon not running (`FindWindowW` returns `0`). | `PostMessageW` returns non-zero but daemon fails to parse on-disk file. | Settings saves successfully; status label states: *"Settings saved. They apply the next time Wira Desk starts."* | `warn!` in Settings diagnostics (no unhandled panic); daemon logs warning on reload parse error. |
| **Filesystem Persistence** (`config.toml` atomic write / rename) | Disk I/O stalls during flush. | `%APPDATA%\WiraDesk` directory missing or read-only; disk full (`EACCES`/`ENOSPC`). | OS reports file renamed but disk buffer uncommitted prior to power loss. | In-line red error banner: *"Could not save settings: &lt;os_error&gt;"*. In-memory draft retained; prior file on disk left intact. | `error!` in Settings with target path and OS error string. |
| **Windows Task Scheduler** (`schtasks.exe` via `std::process`) | `schtasks.exe` process spawn delayed by OS process creation (>1 s). | `schtasks.exe` binary missing or blocked by Group Policy; access denied (`EPERM`). | Process exits with code 0 but task was deleted by external administrative policy. | Toggle switch reverts to prior state on next reload or shows inline warning; auto-start does not activate on boot. | `warn!` recording `schtasks` non-zero exit code or spawn failure. |
| **OS Theme Registry Query** (`RegGetValueW` on `AppsUseLightTheme`) | Registry key query takes >10 ms. | `Personalize` subkey or `AppsUseLightTheme` value missing (custom Windows build/Lite SKU). | Value corrupted (non-DWORD type returned). | Graceful fallback to Light theme (`ThemeMode::Light`); UI remains fully readable with high contrast. | Silent fallback to default theme mode without crash. |
| **System Typography Loader** (`%SystemRoot%\Fonts\segoeui.ttf`) | Disk read of font file takes >50 ms at startup. | `segoeui.ttf` and `tahoma.ttf` absent from disk (minimal container/Windows Server SKU). | TTF file exists but contains corrupted header or invalid glyph table bytes. | `ttf-parser` rejects corrupt bytes; system falls back to Tahoma, then to `LoadedFont::Bundled` (the renderer's own default face). About pane displays actual typeface. | `LoadedFont::Bundled` surfaced in About tab; no crash or panic during atlas generation. |
| **UI Automation** (Slint's Windows accessibility backend, AccessKit internally) | Accessibility tree generation takes several milliseconds on dense frame. | Assistive client (Narrator) not running. | Screen reader queries unmapped node ID during rapid pane switching. | Visual rendering completely unaffected; screen reader receives clean node tree updates or graceful null node responses. | Handled internally by Slint's accessibility backend. |
| **Shortcut Input Validation** (`validate_shortcut`) | User types keys with high latency. | User submits empty string or modifier-only chord. | User inputs conflicting or multiple main keys (`ctrl+a+b`). | Field rejected with precise inline explanation (e.g. *"Switch windows... needs a main key in addition to modifiers"*); draft preserved for correction. | Validation error formatted via `describe()` and rendered in `SaveFeedback::Error`. |
| **Shortcut Collision Check** (`validate_config`, at submission) | No slow path: the comparison runs over the nine in-memory shortcut fields. | Not reachable — a field the grammar check already refused never arrives here. | Two actions carry the same canonical chord, each perfectly legal on its own, so nothing upstream has grounds to refuse either. | Both actions are marked while the draft stands, each naming the other. The submit action stays available throughout; on submission, the draft is refused with a message naming both (SCN-03, LBR-ST-8, LBR-ST-9). | Refusal reason recorded with both field names, via `describe()` as `SaveFeedback::Error`. |

Chord ownership outside this process is not a boundary this table can carry a row for: nothing is
queried, so there is no call to time out, find absent, or catch lying (`DEC-002`). Windows exposes
no way to ask which application a chord will actually reach, and a trial registration would answer
wrong in both directions — refusing a chord the daemon's low-level hook still wins regardless, and
clearing one an earlier third-party hook silently consumes instead. The honest state is silence:
nothing is logged at the moment of editing, and nothing is logged at the moment of a press that an
earlier hook swallows either, since the daemon never observes a keystroke it was never delivered.
Two of the three sources of that silence have since been closed, and the boundary is now drawn where
the knowledge actually is. Chords the **Windows shell** owns are refused from a curated catalogue
carried as data in `shared` — written, reviewed, and versioned knowledge rather than a trial
registration, so it sits inside `DEC-002` rather than against it — and each entry declares whether
Windows keeps the chord regardless of any hook or whether Wira Desk could have taken it and will not
(`DEC-003`). The catalogue replaced `is_reserved_system_shortcut`'s five combinations, under which a
chord such as `Win+Shift+S` or `Win+V` was accepted and then permanently unreachable. Chords an
**external application** holds are still never predicted; what exists now is an observation after the
fact, correlating the daemon's report of what its hook saw against what this window received
(`DEC-005`). What remains genuinely silent is narrower than before and MUST NOT be described as
closed: a chord absent from the catalogue, and a chord nobody presses while the key check is on
screen.

## Robustness Analysis (ABCE)

The Robustness Analysis classifies the technical design for all realized use cases (`UC-4`, `UC-5`, `UC-6`) and edge-case scenarios into Boundary, Control, Entity, and Behaviour.

### 1. Boundary Objects

- **`B-SettingsUI` (Retained-Mode Graphic Surface):** Top-level `Slint` window (`ui/main_window.slint`) declaring panes, buttons, checkboxes, sliders, and feedback banners as components bound to Rust-side model properties.
- **`B-ConfigFile` (TOML Storage on Disk):** Atomic file storage endpoint at `%APPDATA%\WiraDesk\config.toml` (and `.tmp` staging file).
- **`B-DaemonWindow` (Win32 IPC Target):** Top-level message-only window `WiraDeskDaemonHiddenWindow` receiving `WM_APP_RELOAD_CONFIG` (0x8001) and `WM_APP_CAPTURE_LEASE`. The lease message carries its level in `wParam` (0 none, 1 observe, 2 record) and this process's id in `lParam` — a process id and not a window handle, because the hook's comparison is against a foreground process id and a handle would have to be converted on the daemon side (`DEC-004`, `DEF-3`). **This boundary crosses an integrity level.** The daemon is elevated; this process is elevated only when the tray launched it, and runs at medium integrity when a user starts it from Explorer. Both messages therefore depend on the daemon admitting them through UIPI with `ChangeWindowMessageFilterEx` — without that, every post from the non-elevated path is discarded by Windows and reported here as an ordinary failed post.
- **`B-ChordReport` (Win32 IPC Source):** This process's own window, receiving the daemon's report of a chord its hook observed: a virtual-key code and a modifier set, posted never sent. The first channel in this product that runs daemon→settings, which is why `AD-1` names it explicitly.
- **`B-ThemeRegistry` (Windows Personalization Key):** Win32 Registry endpoint `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` (`AppsUseLightTheme`).
- **`B-SystemFonts` (Windows Font Subsystem):** Physical file storage at `%SystemRoot%\Fonts\segoeui.ttf` and `%SystemRoot%\Fonts\tahoma.ttf`.
- **`B-TaskScheduler` (Windows CLI Endpoint):** Windows Task Scheduler CLI interface (`schtasks.exe`).
- **`B-AccessKitAdapter` (Assistive Technology Bridge):** AccessKit Windows UIA tree publisher binding UI controls to screen readers (Windows Narrator).

### 2. Control Objects

- **`C-ShellController` (`LC-settings-shell`):** Coordinates application lifecycle, one-shot theme detection at startup, pane routing, and deterministic keyboard focus sequence. Slint's retained-mode renderer redraws on property change, so nothing here polls for a repaint the way an immediate-mode toolkit would.
- **`C-CaptureManager` (`LC-shortcut-capturer`):** Manages `CaptureState` transitions (`Idle` ↔ `Listening`), builds the candidate chord from the daemon's reported virtual-key code rather than from window-system text, filters modifier-only states, and emits screen reader announcements. Reading the raw code is what makes `Alt+Backtick` — this product's own fallback default, which yields no character at all — recordable, and it retires three heuristics that guessed at the same answer. The text-derived path survives only as a marked fallback for when no daemon is running (`DEC-004`).
- **`C-LeaseArbiter` (`LC-shortcut-capturer`):** Derives the lease level from `(pane, capture)` and posts it only when it changes, so arming and disarming have one owning place each rather than one per exit path. Fails closed: the daemon additionally requires this process to hold the foreground window, and a lease left armed against a dead process is reaped on the daemon's existing heartbeat (residual risk of process-id reuse: `OQ-17`).
- **`C-KeyCheck` (`LC-shortcut-capturer`):** Correlates the daemon's report against what this window received and renders one of four verdicts, stating only what was observed. Reports nothing about a chord nobody pressed, and declines to diagnose at all when no daemon report is available (`DEC-005`).
- **`C-PersistenceManager` (`LC-config-writer`):** Orchestrates configuration validation (`validate_config`), atomic disk serialization (`Config::save`), and IPC signal emission (`signal_reload`).
- **`C-OnboardingWizard` (`LC-settings-shell`):** Drives the 3-step first-run tutorial state machine (`Welcome` → `TrySwitching` → `Done`) and writes initial configuration on completion or skip.
- **`C-AutoStartController` (`daemon::autostart`):** Evaluates `schtasks.exe` arguments (`/Create`, `/Query`, `/Delete`) and synchronizes configuration state. It lives in the daemon, not in this component; `settings` reaches it only by writing `general.auto_start` and signalling reload.

### 3. Entity Objects

- **`E-SettingsDraft` (`SettingsModel.draft`):** Working in-memory copy of `shared::Config` holding staged user modifications prior to validation and persistence.
- **`E-SavedConfig` (`SettingsModel.saved`):** Immutable baseline copy of `shared::Config` reflecting active on-disk configuration; used for dirty-state tracking (`is_dirty()`) and revert operations.
- **`E-CaptureContext` (`CaptureState`):** First-class state machine entity tracking whether input capture is `Idle` or `Listening(ShortcutField)`.
- **`E-SaveOutcome` (`SaveOutcome` / `SaveFeedback`):** Ephemeral outcome entity representing `Saved { reload_signalled: bool }`, `Rejected(&'static str, ShortcutError)`, or `WriteFailed(String)`.
- **`E-OnboardingState` (`OnboardingStep`):** Current progression step of the first-run experience (`Welcome`, `TrySwitching`, `Done`).
- **`E-ControlSemantics` (`ControlSemantics`):** Declared accessibility metadata (accessible name, description, listening announcement) associated with each interactive widget.

### 4. Behaviour & Collaborations

#### UC-4: Change a Keyboard Shortcut in Settings
1. User navigates to the **Shortcuts** pane via mouse click or keyboard navigation (Tab / Arrow keys).
2. `C-ShellController` updates `model.pane = Pane::Shortcuts` and ensures `CaptureState::Idle`.
3. User clicks the button for a shortcut field (e.g. `ShortcutField::Switcher`) or presses Space/Enter on the focused button.
4. `C-CaptureManager` transitions `model.capture` to `CaptureState::Listening(ShortcutField::Switcher)`.
5. `B-AccessKitAdapter` announces: *"Listening for a key combination. Press Escape to cancel."*
6. User presses physical keys (e.g. `Ctrl + Shift + A`).
7. `C-CaptureManager` intercepts the input event, validates that at least one modifier and exactly one main key are present, formats the canonical string `"ctrl+shift+a"`, and updates `model.draft.switcher.shortcut`.
8. `C-CaptureManager` transitions `model.capture` back to `CaptureState::Idle`.
9. User clicks **Save** (or focuses Save and presses Enter).
10. `C-PersistenceManager` calls `validate_config(&model.draft)`:
    - If validation fails: `model.feedback` is set to `SaveFeedback::Error(...)`, displaying the specific invalid field; on-disk file remains untouched.
    - If validation passes: `Config::save()` writes to `config.toml.tmp` and renames to `config.toml`.
11. `C-PersistenceManager` calls `signal_reload()`:
    - `FindWindowW` locates `WiraDeskDaemonHiddenWindow`.
    - `PostMessageW(hwnd, WM_APP_RELOAD_CONFIG, 0, 0)` is dispatched to the daemon.
12. `C-ShellController` promotes `model.draft` to `model.saved`, resets dirty flag, and updates UI banner to *"Settings saved and applied."*

#### UC-5: Complete or Skip First-Run Tutorial
1. Daemon detects no `config.toml` on startup and launches `wiradesk-settings.exe --onboarding` via `ShellExecute`.
2. `C-ShellController` parses arguments via `resolve_launch_intent()`, detects `ONBOARDING_FLAG`, and initializes `model.onboarding = Some(OnboardingStep::Welcome)`.
3. `B-SettingsUI` renders Step 1 (Welcome heading and spatial isolation philosophy contrasting with Alt+Tab).
4. User clicks **Next**: `C-OnboardingWizard` advances to Step 2 (`TrySwitching`), displaying live same-app cycling guidance.
5. User clicks **Next**: `C-OnboardingWizard` advances to Step 3 (`Done`).
6. User clicks **Finish** (or user clicks **Skip Tutorial** at any step):
7. `C-OnboardingWizard` invokes `model.save(&config_path())`, persisting default `shared::Config` to `%APPDATA%\WiraDesk\config.toml` and signalling the daemon.
8. `model.onboarding` is cleared to `None`, transitioning the interface into the standard Settings panes.

#### UC-6: Turn Auto-Start on Boot On or Off
1. User navigates to the **General** pane and toggles the **"Start Wira Desk with Windows"** checkbox.
2. `model.draft.general.auto_start` is modified, marking `model.is_dirty() = true`.
3. User clicks **Save**.
4. `C-PersistenceManager` validates configuration and atomically commits `config.toml`.
5. `C-PersistenceManager` signals `WM_APP_RELOAD_CONFIG` to the daemon hidden window.
6. The daemon reloads config and reconciles the task: if `auto_start` was enabled, `C-AutoStartController` invokes `schtasks.exe /Create /TN WiraDesk /TR "\"<exe_path>\"" /SC ONLOGON /RL HIGHEST /RU "%USERNAME%" /F`; if disabled, `schtasks.exe /Delete /TN WiraDesk /F`.
7. The tray menu checkmark is read back from `schtasks /Query`, never from the config value.

#### SCN-01: Invalid Shortcut Combination Rejected
1. User enters listening mode on a shortcut field and presses a bare key without modifiers (e.g. `Tab` or `A`) or a multi-key chord (`Ctrl + A + B`).
2. `C-CaptureManager` attempts validation via `validate_shortcut()`:
   - Bare key returns `Err(ShortcutError::NoModifier)`.
   - Multi-key chord returns `Err(ShortcutError::MultipleMainKeys)`.
3. `C-CaptureManager` preserves `model.draft` with the prior valid shortcut, sets `model.feedback = SaveFeedback::Error(...)`, and remains in `Listening` state.
4. UI displays red inline error description; screen reader announces error reason.
5. User presses `Escape`: `C-CaptureManager` restores `CaptureState::Idle` without altering stored configuration.

#### SCN-02: Auto-Start Task Creation Fails
1. User enables Auto-Start and clicks Save, but system security policy or permissions prevent Scheduled Task creation.
2. `schtasks.exe /Create` exits with a non-zero error code.
3. `C-AutoStartController` logs a warning diagnostic; `General` pane UI reflects Task Scheduler failure.
4. Configuration file retains user preference; daemon checks task existence via authoritative `schtasks /Query` before updating context menu checkmarks.

## Evidence

Every architectural claim, constraint, and boundary behaviour is verified against source code in the repository:

| Claim | Label | Read to decide | Disposition |
| --- | --- | --- | --- |
| Atomic temp-file write before IPC reload signal | Verified | `crates/settings/src/persistence.rs` (`save_and_notify`), `crates/shared/src/config.rs` (`Config::save`) | Verified: `Config::save` writes to `.tmp` file and executes atomic rename before `signal_reload()` calls `PostMessageW`. |
| Out-of-process `ShellExecute` launch with `--onboarding` | Verified | `crates/daemon/src/main.rs`, `crates/settings/src/persistence.rs` (`resolve_launch_intent`), `crates/shared/src/constants.rs` | Verified: Daemon spawns `wiradesk-settings.exe` via `ShellExecuteW`; `--onboarding` flag shared in `shared::ONBOARDING_FLAG`. |
| Slint accessibility feature and UI Automation integration | Verified | `crates/settings/Cargo.toml`, `crates/settings/ui/**/*.slint` | Verified: `slint` is declared with `default-features = false` plus an explicit `"accessibility"` feature; interactive elements declare `accessible-role`/`accessible-label` (e.g. `ui/components/key_check.slint`, `ui/panes/layout_pane.slint`). |
| Segoe UI system font loading with TTF verification & fallback | Verified | `crates/settings/src/theme.rs` (`detect_ui_font`, `LoadedFont`), `crates/settings/Cargo.toml` (`ttf-parser`) | Verified: Checks `%SystemRoot%\Fonts\segoeui.ttf` and `tahoma.ttf` with `ttf_parser::Face::parse` before installing; falls back to `LoadedFont::Bundled`. |
| OS Theme detection & dual-theme focus outline styling | Verified | `crates/settings/src/theme.rs` (`detect_theme`), `crates/settings/src/main.rs`, `crates/settings/ui/theme.slint` | Verified: Reads `AppsUseLightTheme` DWORD via `RegGetValueW`; `main.rs` calls `main_window.global::<Palette>().set_is_dark(is_dark)` once at startup, and every component reads colors from that global. |
| Strict shortcut validation & canonicalization | Verified | `crates/settings/src/persistence.rs` (`validate_shortcut`, `classify_parse_failure`) | Verified: Rejects bare keys (`NoModifier`), multiple main keys (`MultipleMainKeys`), and unsupported tokens; returns canonical lowercase strings. |
| Deterministic focus order verification in debug builds | Verified | `crates/settings/src/app.rs` (`focus_order`, `focus_order_mismatch`) | Verified: Explicit focus stop arrays for each pane; debug assertion validates drawn UI sequence against declared contract. |
| Windows Task Scheduler `ONLOGON` elevation contract | Verified | `crates/daemon/src/autostart.rs` (`create_args`, `is_registered`), `crates/shared/src/constants.rs` | Verified: Creates task `WiraDesk` with `/SC ONLOGON /RL HIGHEST /RU "%USERNAME%" /TR "\"<exe_path>\"" /F` without console window. |
| Three-crate Cargo workspace dependency hierarchy | Verified | `Cargo.toml`, `crates/settings/Cargo.toml`, `crates/daemon/Cargo.toml`, `crates/shared/Cargo.toml` | Verified: Shared crate owns `Config`, `Shortcut`, `constants`, and migration logic; `daemon` and `settings` depend on `shared`. |

## Open Items

None. All technical mechanisms, invariants (AD-1, AD-5, AD-11, AD-11a, AD-12, AD-13), and boundary failure modes are verified against the codebase and ratified.
