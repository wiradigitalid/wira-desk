# C4 L1 — System Context

```mermaid
graph TD
    User["Power User / Developer<br/>[Person]"]
    
    subgraph WiraDeskSystem["Wira Desk<br/>[Software System]"]
        WD["Wira Desk Utility<br/>(Daemon + Settings GUI)"]
    end
    
    subgraph WindowsOS["Windows Desktop Operating System<br/>[External System]"]
        WinHook["Win32 Low-Level Keyboard Hooks<br/>(WH_KEYBOARD_LL)"]
        WinWM["Windows Window Manager / DWM<br/>(Z-Order, Focus, Snapping)"]
        WinVD["Virtual Desktop Manager<br/>(IVirtualDesktopManager)"]
        WinTray["System Tray & Shell<br/>(Shell_NotifyIcon, Toast)"]
        WinTS["Windows Task Scheduler<br/>(Logon Auto-Start)"]
    end
    
    subgraph Storage["Local Storage<br/>[Filesystem]"]
        AppData[("%APPDATA% / WiraDesk<br/>config.toml & wiradesk.log")]
    end

    User -- "Global key shortcuts<br/>(Win+`, Ctrl+Win+Arrows)" --> WinHook
    WinHook -- "Hook events" --> WD
    User -- "Tray menu click / Configuration" --> WD
    
    WD -- "EnumWindows, SetForegroundWindow, SetWindowPos" --> WinWM
    WD -- "IsWindowOnCurrentVirtualDesktop" --> WinVD
    WD -- "Shell_NotifyIcon, TaskbarCreated listener" --> WinTray
    WD -- "schtasks /Create, /Query, /Delete" --> WinTS
    WD -- "Read / Write configuration & log diagnostics" --> AppData
```

### System Boundaries & Description

- **Wira Desk**: Lightweight, local Windows utility that delivers instant macOS-style same-application window cycling and DPI-aware snapping.
- **External Systems**:
  - **Windows Low-Level Hooks**: Intercepts physical key combinations globally before target applications receive them.
  - **Windows Window Manager / DWM**: Live Z-order enumeration, active window focus transitions, DPI-aware bounds retrieval, and window positioning.
  - **Virtual Desktop Manager**: Official COM interface (`IVirtualDesktopManager`) isolating cycling within the active virtual desktop.
  - **System Tray & Toast**: Native Win32 notification icon, context menu, and critical error toast notifications.
  - **Windows Task Scheduler**: Elevation-preserving logon trigger executing silently without repetitive UAC prompts.
  - **Local Filesystem**: Non-volatile storage for user configuration (`%APPDATA%\WiraDesk\config.toml`) and append-only diagnostic log (`%APPDATA%\WiraDesk\wiradesk.log`).
- **Network Boundaries**: None. Wira Desk operates 100% offline with zero outbound/inbound network traffic.
