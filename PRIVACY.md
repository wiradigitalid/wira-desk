# Privacy

Wira Desk does not send telemetry or network traffic. There is no socket, HTTP client, or
update check in the source. Configuration and optional log files are stored locally under
`%APPDATA%\WiraDesk\`.

## Keystrokes

The daemon installs a global keyboard hook, so it observes every key event on the desktop.
It reads the virtual-key code and the injected-input flag in order to match the two
configured shortcuts, and **records none of it** â€” not to the log file, not to the debug
trace, not anywhere that outlives the event. No logging call in the codebase takes a
virtual-key value as an argument.

## Window metadata

To choose a switching or arrangement target the daemon reads window class names, visibility
and cloak state, and the executable basename of the owning process. That is read at the
moment a shortcut fires, kept only for the duration of that operation, and never uploaded.
Window titles are not used for switching decisions.

## What is stored on disk

| Path | Contents | Retention |
| --- | --- | --- |
| `%APPDATA%\WiraDesk\config.toml` | Your shortcuts, bypass lists, layout, auto-start flag | Until you delete it |
| `%APPDATA%\WiraDesk\wiradesk.log` | Timestamped warning lines | Until you delete it â€” **there is no log rotation in this version**, so the file grows unbounded |

Both live in your user profile at normal user permissions, so treat them as readable by
anything else running as you. To reset configuration, delete `config.toml` only â€” see
`CHANGELOG.md`, because deleting the whole folder is not a reset.