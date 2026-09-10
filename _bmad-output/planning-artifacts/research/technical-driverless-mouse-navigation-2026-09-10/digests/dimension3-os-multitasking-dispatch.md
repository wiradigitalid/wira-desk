# Digest: OS Multitasking Dispatch & Windows Integration (Dimension 3)

## Findings & Extracted Claims

1. **Virtual Desktop Switching Mechanism Evaluation:**
   - **COM Interface (`IVirtualDesktopManagerInternal`):** Undocumented, non-stable shell COM interface. Historical analysis of open-source utilities (e.g. `virtual-desktop-enhancer`, `MScholtes/VirtualDesktop`) demonstrates that Microsoft alters the GUID, method order, and parameter structures across major Windows 10 and Windows 11 updates (21H2, 22H2, 23H2, 24H2). Relying on this interface introduces critical regression risks on monthly OS patches.
   - **Synthetic Keystroke Injection (`SendInput`):** Standard `Ctrl + Win + Left/Right` shortcuts remain completely invariant across all Windows 10 and 11 builds.
   - Sources: [MScholtes/VirtualDesktop on GitHub](https://github.com/MScholtes/VirtualDesktop) and Windows release documentation.

2. **UIPI Bypass & Elevation Context:**
   - On Windows, `SendInput` from a medium-integrity process is blocked by User Interface Privilege Isolation (UIPI) when an elevated window (Administrator command prompt, Task Manager, IDE in run-as-admin mode) holds foreground focus.
   - Because Wira Desk's core daemon runs with elevated privileges (`requireAdministrator`), synthetic inputs dispatched by its Worker thread reliably penetrate UIPI barriers across all desktop targets.

3. **Decoupled Worker Architecture:**
   - To prevent input hook timeouts and OS unhooking (governed by `LowLevelHooksTimeout` in Windows registry), input interception (`WH_MOUSE_LL`) must be completely decoupled from action execution.
   - Intercepted events are pushed into the daemon's lock-free ring buffer, and the dedicated Worker thread drains the ring and issues commands or synthesizes keystrokes asynchronously.
   - Class: `architecture/os-integration`. Confidence: High.
