# Digest: Win32 HID & Interoperability (Dimension 2)

## Findings & Extracted Claims

1. **Win32 Low-Level Mouse Hook (`WH_MOUSE_LL`):**
   - Implemented via `SetWindowsHookExW(WH_MOUSE_LL, LowLevelMouseProc, hMod, 0)` in `user32.dll`.
   - References: Microsoft Learn [`LowLevelMouseProc`](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc) and [`MSLLHOOKSTRUCT`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct).
   - `MSLLHOOKSTRUCT.mouseData`:
     - On `WM_XBUTTONDOWN` / `WM_XBUTTONUP`: The high-order word of `mouseData` indicates the button: `XBUTTON1` (`0x0001`, lower thumb / back) or `XBUTTON2` (`0x0002`, upper thumb / forward). Returning `1` from the hook swallows the event to prevent unwanted browser navigation.
     - On `WM_MOUSEHWHEEL`: The high-order word carries the horizontal wheel tilt delta as a signed 16-bit integer (`GET_WHEEL_DELTA_WPARAM(mouseData)`). A negative delta indicates a tilt to the left; a positive delta indicates a tilt to the right. Returning `1` suppresses default horizontal scrolling.

2. **Cursor Motion & Polling Latency Invariant:**
   - Standard modern optical/laser mice poll at 125 Hz to 1000 Hz, sending `WM_MOUSEMOVE` messages for every cursor increment.
   - Low-level hook callbacks are synchronous within the message pump thread. Any locking, IPC, allocation, or non-trivial computation in `LowLevelMouseProc` during `WM_MOUSEMOVE` will cause perceptible micro-stutter and input lag.
   - Architectural invariant: The callback must immediately pass `WM_MOUSEMOVE` through via `CallNextHookEx(0, nCode, wParam, lParam)` without entering any lock or inspection path.

3. **Tilt Wheel Burst Throttling (Debounce):**
   - Physical tilt wheel switches deliver multiple rapid `WM_MOUSEHWHEEL` ticks per physical flick. Without a software debounce timer (recommended 150 ms – 200 ms), a single tilt flick can trigger 2–4 consecutive action activations.
   - Class: `technical/win32-input`. Confidence: High.
