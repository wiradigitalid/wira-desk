---
type: scn
id: SCN-01
component: window-management
attaches_to: UC-1
created: '2026-08-21'
---

# SCN-01 — VM and remote desktop passthrough bypass during cycle

## Where it branches

Leaves from **UC-1 (Cycle to the next window of the same app on this monitor)** at **Step 3** (Foreground window discovery).

## Condition

The active foreground window matches configured virtual machine or remote desktop client signatures (e.g. process names `mstsc.exe`, `vmconnect.exe`, `vmware.exe`, `VirtualBoxVM.exe`, or known remote desktop class names `TscShellContainerClass`, `VMPlayerFrame`).

## Flow

1. User is actively working inside a remote desktop session or virtual machine guest operating system.
2. User presses the configured cycling shortcut (e.g. `Win + \``) intended for the guest environment or remote desktop host.
3. System low-level keyboard hook callback intercepts the key event on the dedicated hook thread.
4. System queries foreground window handle and evaluates executable image name and window class name against the static non-blocking bypass registry.
5. System identifies match with VM/RDP bypass signature (conforming to invariant AD-6 and BR-5).
6. System immediately invokes `CallNextHookEx` to forward the raw keystroke downstream to the OS input pipeline without swallowing or modifying key states.
7. System skips ring-buffer command enqueuing and does not signal the worker thread.
8. Guest virtual machine or remote desktop session receives the keystroke untouched and handles internal application cycling or window management natively.

## Outcome

Keystroke passes cleanly through to the remote host or virtual guest without host-level interception or focus stealing. Zero ring-buffer capacity or worker CPU cycles are consumed on the host.

## Why it is not in the UC

Isolates complex multi-target virtual machine and remote desktop client filtering rules and passthrough latency guarantees without cluttering the primary same-application cycling flow.
