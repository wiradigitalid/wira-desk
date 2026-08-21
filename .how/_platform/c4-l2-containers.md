# C4 L2 — Containers

| Container | Binary | Product Components Served | Technology | Responsibilities |
| --- | --- | --- | --- | --- |
| **daemon** | `wiradesk.exe` | `window-management` | Rust, `windows-sys`, Win32 API | Installs global `WH_KEYBOARD_LL` hook, maintains lock-free command ring buffer, executes stateless Z-order window cycling and DPI-aware snapping, manages tray icon / context menu, monitors hook health (10s heartbeat). Runs elevated (`requireAdministrator`). |
| **settings** | `wiradesk-settings.exe` | `settings` | Rust, `egui`, `eframe` + `accesskit` | Provides accessible GUI for editing shortcut bindings, VM bypass list, snapping preferences, and auto-start. Hosts first-run onboarding tutorial. Writes `config.toml` atomically and dispatches `WM_APP_RELOAD_CONFIG` to daemon. |
| **shared** | (library crate) | `_platform` | Rust, `serde`, `toml` | Single source of truth for config schema, default bindings, `u8` command enum, IPC message IDs, APPDATA paths, and legacy WinTick migration logic. |

```mermaid
graph TB
    subgraph WiraDeskApp["Wira Desk Application"]
        subgraph DaemonContainer["daemon (wiradesk.exe) [Container: Rust / Win32]"]
            HookThread["Hook Thread<br/>(Interception, Throttle, VM Bypass)"]
            RingBuf["Lock-Free Ring Buffer<br/>(16 slots, u8 commands)"]
            WorkerThread["Worker Thread<br/>(Z-Order, Focus, Snapping)"]
            MainThread["Main Thread & Message Window<br/>(Tray Icon, Health Monitor, IPC WndProc)"]
            
            HookThread -- "Push u8 command" --> RingBuf
            RingBuf -- "Pop command" --> WorkerThread
            MainThread -. "Supervise hook / Heartbeat" .-> HookThread
        end

        subgraph SettingsContainer["settings (wiradesk-settings.exe) [Container: Rust / egui]"]
            SettingsUI["Settings GUI & Accessibility<br/>(General, Switcher, Snap, Bypass, About)"]
            OnboardingUI["First-Run Onboarding<br/>(--onboarding simulation)"]
            ConfigWriter["Config Writer<br/>(Atomic save to temp + rename)"]
            
            SettingsUI --> ConfigWriter
            OnboardingUI --> ConfigWriter
        end

        subgraph SharedCrate["shared [Library Crate: Rust]"]
            ConfigSchema["Config TOML Model & Defaults"]
            CommandEnum["Command u8 Enum & Constants"]
            PathConstants["Paths & Win32 IPC Message IDs"]
        end
    end

    subgraph FileSystem["Local Storage"]
        ConfigFile[("%APPDATA%/WiraDesk/config.toml")]
        LogFile[("%APPDATA%/WiraDesk/wiradesk.log")]
    end

    DaemonContainer --> SharedCrate
    SettingsContainer --> SharedCrate

    MainThread -- "ShellExecute (launch on demand)" --> SettingsContainer
    ConfigWriter -- "Atomically writes" --> ConfigFile
    ConfigWriter -- "WM_APP_RELOAD_CONFIG (Win32 IPC)" --> MainThread
    MainThread -- "Reloads on signal" --> ConfigFile
    MainThread -- "Appends diagnostics" --> LogFile
```

## Container Interaction Matrix

| Initiator | Target | Channel / Protocol | Purpose |
| --- | --- | --- | --- |
| `daemon` (tray menu) | `settings` | `ShellExecute` (`wiradesk-settings.exe`) | Open Settings GUI (or onboarding via `--onboarding` on first run). Inherits Administrator elevation. |
| `settings` | `daemon` | `WM_APP_RELOAD_CONFIG` (Win32 `PostMessageW` / `SendMessageW` to hidden window) | Notify daemon that `config.toml` was atomically written to disk and needs immediate reload. |
| `Hook Thread` | `Worker Thread` | In-process lock-free static ring buffer (`16` slots of `u8`) | Pass validated, throttled shortcut commands with zero heap allocation. |
| `health.rs` | `Hook Thread` / Main | `WM_APP_HOOK_CHECK`, `WM_APP_HOOK_DEAD` | 10-second heartbeat check and Tier-3 critical escalation. |
| `daemon` | Filesystem | Direct I/O (`%APPDATA%\WiraDesk\config.toml`, `wiradesk.log`) | Read config on boot/signal; append diagnostic warnings/errors. |
| `settings` | Filesystem | Direct I/O (`config.toml.tmp` -> `config.toml`) | Atomically persist user changes without partial-read races. |
