# Cross-Cutting Concerns

## Platform-Owned Entities

| ID | Kind | Owner | Description |
| --- | --- | --- | --- |
| `app-config` | Data | `_platform` | TOML configuration schema and on-disk file `%APPDATA%\WiraDesk\config.toml` shared by daemon and settings. |
| `ipc-reload-signal` | Control / Endpoint | `_platform` | Custom Win32 message `WM_APP_RELOAD_CONFIG` (`0x8001`) sent by settings to the daemon's hidden message-only window (`WiraDeskDaemonHiddenWindow`). |
| `runtime-paths` | Data | `_platform` | Standard paths for `%APPDATA%\WiraDesk` (config, log), executable locations, and legacy `%APPDATA%\WinTick` migration paths. |

## Platform-Owned Specifications

### 1. `app-config`

- **Schema Definition**: Shared Rust struct in `crates/shared/src/config.rs` with `#[derive(Serialize, Deserialize, Default)]`.
- **Serialization**: TOML format via `toml` crate.
- **Fail-Safe Policy**: Missing or malformed config falls back safely to internal defaults without crashing the daemon.
- **Atomic Persistence**:
  ```text
  1. Serialize Config struct to TOML string
  2. Ensure parent directory exists (%APPDATA%\WiraDesk\)
  3. Write string to temporary file: config.toml.tmp
  4. Atomic rename: config.toml.tmp -> config.toml
  5. Dispatch WM_APP_RELOAD_CONFIG to daemon hidden window
  ```

### 2. `ipc-reload-signal`

- **Sender**: Settings process (`crates/settings/src/persistence.rs`).
- **Receiver**: Daemon main thread hidden window (`crates/daemon/src/main.rs`).
- **Target Lookup**: `FindWindowW(class: "WiraDeskDaemonHiddenWindow", title: "WiraDeskDaemon")`.
- **Message Code**: `WM_APP_RELOAD_CONFIG = WM_APP + 1` (`0x8001`).
- **Action on Receipt**: Daemon re-reads `%APPDATA%\WiraDesk\config.toml` and posts an updated immutable snapshot to the Hook Thread via `WM_APP_CONFIG_SNAPSHOT`.

### 3. `runtime-paths`

- **App Subdirectory**: `%APPDATA%\WiraDesk`
- **Config Path**: `%APPDATA%\WiraDesk\config.toml`
- **Log Path**: `%APPDATA%\WiraDesk\wiradesk.log`
- **Settings Executable**: `wiradesk-settings.exe` located in the same directory as `wiradesk.exe`.
- **Legacy Migration Path**: `%APPDATA%\WinTick\config.toml` (one-time automatic copy to `%APPDATA%\WiraDesk\config.toml` if target does not exist).

## Error Protocol & Envelope

Wira Desk is an offline Windows desktop utility. It uses no HTTP/JSON error envelopes. Errors follow the strict 3-Tier Error Protocol (AD-7):

| Tier | Name | Trigger Condition | Visual Indicator | User Notification | Recovery Action |
| --- | --- | --- | --- | --- | --- |
| **Tier 1** | Startup Fatal | Missing required OS features, unrecoverable hook initialization failure after max retries (`HOOK_RETRY_MAX = 5`). | None (process terminates) | Exactly 1x Win32 `MessageBoxW` (Error icon). | Process exits immediately. No retry loop. |
| **Tier 2** | Runtime Warning | Non-blocking Win32 API failure, single window focus error, minor config parse warning. | Tray icon displays **Red Dot** overlay ("unread warning log"). | Silent. No modal popup, no toast. | Log entry appended to `wiradesk.log`. Normal operation continues. Clicking "View Logs" clears tray dot. |
| **Tier 3** | Runtime Critical | Keyboard hook died and could not be recovered after `HOOK_CHECK_FAIL_THRESHOLD = 3` consecutive heartbeat ticks (30s). | Tray icon displays **Red X** overlay ("stopped / degraded"). | Exactly 1x Windows Toast Notification alerting that cycling has paused. | Hook thread stops interception. User can restart or inspect logs. |

## IPC Command Protocol

- **Transport**: In-process lock-free static ring buffer (`16` slots). Zero heap allocation on hot input path.
- **Payload**: Raw `u8` command enum defined in `crates/shared/src/commands.rs`:
  - `0`: `Nop`
  - `1`: `Cycle` (Win+Backtick)
  - `2`: `SnapLeft` (Ctrl+Win+Left)
  - `3`: `SnapRight` (Ctrl+Win+Right)
  - `4`: `SnapMaximize` (Ctrl+Win+Enter)
  - `5`: `OverlappingStack` (Ctrl+Win+Down)
- **Queue Overflow Policy**: If the ring buffer is full (e.g. extreme spam), new incoming commands are dropped immediately by the Hook Thread.

## Security & Elevation

- **UIPI Bypass**: Daemon embeds an application manifest specifying `requestedExecutionLevel level="requireAdministrator"`. This permits focusing elevated applications (e.g. Task Manager, elevated cmd/powershell).
- **DLL Hijacking Mitigation**: Both `wiradesk.exe` and `wiradesk-settings.exe` invoke `SetDllDirectoryW(L"")` immediately upon startup.
- **Logon Task Hardening**: Task Scheduler entry created with `/RL HIGHEST` and `/RU "%USERNAME%"`. `Start in` is left empty or explicitly pointing to the application directory.
