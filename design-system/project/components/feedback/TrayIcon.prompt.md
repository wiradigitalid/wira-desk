# TrayIcon

WinTick's system-tray presence. Encodes the 3-tier error protocol (FR-11):

- **normal** — plain glyph. WinTick is responding.
- **warning** — small red dot. A non-fatal error was silently logged.
- **error** — large red cross. Keyboard hook is dead; pair with a Windows toast.

```jsx
<TrayIcon state="normal" />
<TrayIcon state="warning" />
<TrayIcon state="error" size={40} />
```

Set `onDark={false}` on a light taskbar.
