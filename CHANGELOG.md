# Changelog

## 0.1.0

Initial public source release of Wira Desk.

**Factory reset:** delete `%APPDATA%\WiraDesk\config.toml` only - not the folder. Migration re-runs if the legacy `%APPDATA%\WinTick\` directory still exists; that directory is intentionally preserved for rollback, so removing the entire `WiraDesk` folder is not a reset.