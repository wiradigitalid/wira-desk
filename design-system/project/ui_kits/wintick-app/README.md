# UI Kit — WinTick App (native Windows-11 world)

Interactive recreation of WinTick's actual in-product surfaces. Uses the `.native` token
set + Segoe UI so it reads as a first-party Windows utility. **Brand saga-red never appears
here** — the only injected color is the mandated tray-alert red `#E81123`.

**`index.html`** simulates a Windows 11 desktop:
- **First-run onboarding** (`Onboarding.jsx`) launches over the desktop — practise `Win + \``
  on live dummy windows; "Skip Tutorial" available (FR-17).
- **System tray** — click / right-click the WinTick tray icon to open the context menu
  (`TrayMenu`), choose **Settings…** to open the window.
- **Settings window** (`SettingsWindow.jsx`) — feature-grouped rows built from `Toggle`
  (native), `ShortcutInput` (native), `Keycap`, `Badge`.
- **Demo bar (top)** toggles the 3-tier tray health states; **Error** fires a Windows toast
  with solutive microcopy (FR-11).

Composes design-system components: `TrayIcon`, `TrayMenu`, `Toggle`, `ShortcutInput`,
`Keycap`, `Button`, `Badge`. Load order: `_ds_bundle.js` → `Onboarding.jsx` →
`SettingsWindow.jsx` → app script.
