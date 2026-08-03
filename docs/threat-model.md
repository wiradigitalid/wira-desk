# Threat Model

Wira Desk runs elevated, installs a global low-level keyboard hook, and can register a
logon task that starts it elevated without a prompt. Those are the three properties that
make a switching utility look, from the outside, indistinguishable from a keylogger. This
document exists so the difference is checkable rather than asserted: it states what the
daemon is allowed to do, why each privilege is needed, where the trust boundaries are, and
which risks remain after the mitigations.

Written against the source in this repository. Where a claim can be checked in code, the
file is named. Where a risk is not mitigated, it is listed under
[Residual risks](#residual-risks) rather than omitted.

## Components

| Component | Integrity | Holds |
| --- | --- | --- |
| `wiradesk.exe` (daemon) | High (elevated, required) | Keyboard hook, tray window, window placement |
| `wiradesk-settings.exe` (Settings) | Inherits its launcher | Configuration editing only; no hook, no window control |
| `%APPDATA%\WiraDesk\config.toml` | Medium (user-writable) | Shortcuts, VM/RDP bypass lists, layout, auto-start flag |
| `%APPDATA%\WiraDesk\wiradesk.log` | Medium (user-writable) | Tier-2 warnings; see [Logging](#logging) |
| Logon scheduled task (optional) | Runs the daemon at High | Auto-start |

Inside the daemon there are three threads with deliberately separated duties: the Hook
thread owns the hook and its configuration snapshot, the Worker (main) thread owns window
enumeration and placement, and a heartbeat thread only posts a timer tick. Configuration
crosses between them by explicit message passing, never shared mutable state.

## Privileges, and why each one is required

Requesting more privilege than a feature needs is the failure mode this section is meant to
make visible.

**Administrator (`requireAdministrator`).** Required for exactly one reason: activating and
moving windows that belong to processes at a higher integrity level than a normal user
process. Windows blocks that across integrity levels (UIPI), so a medium-integrity switcher
silently fails on any elevated window — including Task Manager and any admin console the
user has open. Elevation is not used to read other processes' memory, install drivers, or
write outside `%APPDATA%`.

The manifest is not the only check. `main.rs` re-queries the process token
(`OpenProcessToken` + `GetTokenInformation`) and refuses to start when not elevated, so a
binary rebuilt without the manifest fails loudly instead of running with half its
functionality.

**Global low-level keyboard hook (`WH_KEYBOARD_LL`).** Required because the shortcut must
work while any application has focus, which is what a system-wide hotkey means. The narrower
alternative, `RegisterHotKey`, cannot express the behaviour the product needs: it cannot
suppress the Start Menu that a lone `Win` press opens, and it fails when another application
has already claimed the combination.

**`PROCESS_QUERY_LIMITED_INFORMATION`.** The narrowest right that answers "what executable
is this window's process". Deliberately not `PROCESS_QUERY_INFORMATION`, and never
`PROCESS_VM_READ` — the daemon never reads another process's memory. Used in
`cycling/source.rs` and `context/vm_bypass.rs`.

**Task Scheduler (`/RL HIGHEST /SC ONLOGON`), only if the user enables auto-start.** An
elevated program cannot auto-start through the ordinary `Run` key without a UAC prompt at
every logon, so a logon task is the mechanism Windows provides. See
[Auto-start](#auto-start-is-an-elevation-path) for the consequence, which is real.

## What the keyboard hook does and does not do

The hook callback receives every key event on the desktop. What it does with them is
bounded, and the bound is enforced by design rather than by review:

- It reads the virtual-key code and the injected-input flag, updates a four-bool modifier
  state, and compares that against the two configured shortcuts.
- **No keystroke content is written anywhere.** Not to the log, not to the debug trace, not
  to memory that outlives the event. Checkable: no logging call in `crates/` takes a
  virtual-key value as an argument. The debug-only traces record latency, cycle outcomes,
  and counters — never key values.
- No heap allocation, no lock, no file I/O, and no logging on the callback path. Identity
  lookups for the VM/RDP bypass use fixed buffers owned by `HookIdentityCollector`
  (`[u16; 256]` for the class, `[u16; 260]` for the path), reused per event, compared
  directly against pre-normalised policy strings so no `String` is ever built.
- Synthetic input (`LLKHF_INJECTED`) is rejected. The daemon injects an unassigned key
  itself to suppress the Start Menu, so processing injected events would let the hook
  consume its own injection. A measurement seam can open that gate, and it is
  `#[cfg(debug_assertions)]`-gated — in release the function is a `const false` and the
  branch folds away.
- Matched shortcuts are swallowed; modifier releases are never swallowed. Swallowing a
  modifier release leaves the focused application believing the key is still held, which is
  a worse bug than an occasional Start Menu.
- There is no network path in the product. No socket, HTTP client, or telemetry call exists
  in `crates/`.

## Trust boundaries and attack surface

### Configuration is written at medium integrity and read at high

`config.toml` lives under `%APPDATA%`, so any process running as the user — including one at
medium integrity that cannot elevate — can rewrite it, and the elevated daemon then parses
and applies it. This is a genuine boundary crossing, and it is bounded rather than
eliminated:

- Every field is typed. Shortcuts must parse as shortcuts or the whole reload is rejected;
  the bypass lists are compared as strings against process basenames and window class names
  and are **never** executed or used as paths; layout fields are numbers that become window
  geometry.
- No configuration value reaches a command line or a filesystem path. The one privileged
  action configuration can trigger is auto-start registration, and `autostart::create_args`
  builds it from `current_exe_path()` and `%USERNAME%` — not from anything in the file. An
  attacker rewriting `config.toml` cannot point the scheduled task at another binary.
- Rejection is all-or-nothing (`daemon/config.rs`): an unreadable, malformed, or
  semantically invalid file leaves every actor on its last-known-good configuration and
  emits one Tier-2 warning. A partially applied reload would be worse, because the user
  could not tell which half took effect.

Realistic impact of a hostile config write: the shortcut stops working (add everything to
the bypass list), windows are placed oddly, or auto-start is toggled. Not code execution.

### Auto-start is an elevation path

The logon task runs the daemon with `/RL HIGHEST`, which means **elevated at every logon
with no UAC prompt**. That is what makes auto-start useful, and it has a direct consequence:
anyone who can overwrite the executable at the registered path gains a program that runs
elevated at logon without the user seeing anything.

The task stores an absolute path and sets no working directory, so the path itself cannot be
hijacked. The exposure is filesystem permissions on the install location, and that is the
user's to get right:

> Install Wira Desk in a directory only administrators can write — `%ProgramFiles%` or
> similar. Do not enable auto-start from a build sitting in `Downloads`, `Desktop`, or any
> other user-writable folder.

### Window-message IPC

Settings and the daemon communicate with `WM_APP` window messages, and **every one of them
carries zero in both `wParam` and `lParam`**. They are pure "look again" signals.
`WM_APP_RELOAD_CONFIG` is typical: the daemon re-reads the configuration file itself, so no
data crosses the process boundary and a pointer from another address space would be
meaningless anyway.

That uniformity is deliberate and was not always true. `WM_APP_CONFIG_SNAPSHOT` used to
carry a leaked `Box<HookSnapshot>` pointer in `lParam`, which the handler reconstructed with
`Box::from_raw`. It was sound for the intended sender, but the handler had no way to
distinguish a pointer the Worker had staged from an arbitrary integer — so a
`WM_APP_CONFIG_SNAPSHOT` originating anywhere else would have been a free of an
attacker-chosen address. Nothing could actually deliver one: the daemon runs at high
integrity, UIPI blocks lower-integrity processes from posting to it, and
`ChangeWindowMessageFilterEx` is called exactly once, for the `TaskbarCreated` broadcast
alone, never for a `WM_APP` message. But that is safety by unreachability, one refactor away
from not holding.

The snapshot now waits in a slot inside the process (`hook::PENDING_SNAPSHOT`, an
`AtomicPtr`) and the message is only a wake-up. Ownership is governed by a single rule —
whoever swaps a non-null pointer *out* of the slot owns it and frees it — which is what
keeps a superseded snapshot from either leaking or being freed twice. A spurious or
duplicated wake-up now makes the Hook thread collect whatever the Worker legitimately
staged, or nothing at all; neither outcome involves a pointer the daemon did not create.

The remaining trust boundary here is therefore just the message filter, and it stays
narrow: opening `WM_APP` messages to lower-integrity senders would let another process
trigger a configuration re-read, which is harmless, but there is no reason to do it.

### DLL search order

`SetDllDirectoryW` with an empty string is the first statement in `main`, before anything
else runs, which removes the current directory from the DLL search path. For an elevated
process this matters: a planted DLL beside the executable would otherwise load with the
daemon's privileges. The auto-start task also sets no working directory, for the same
reason.

### Single-instance mutex

The daemon claims `Global\...` at startup. A lower-privilege process can create that name
first to make the daemon exit — `ERROR_ACCESS_DENIED` is treated as "already running" and
the process exits 0. The effect is denial of a convenience feature, not privilege gain or
data exposure. Treating a squatted name as a reason to continue would be worse: two live
global keyboard hooks fighting over the same shortcut.

### Window and process enumeration

One `EnumWindows` sweep per accepted command, with no cache between commands. Only
non-blocking metadata is read — visibility, cloak state, class name, executable basename —
so a hung window answers as fast as a healthy one and cannot stall the daemon. Window
titles are not used for switching decisions.

### Dependencies

The daemon depends on `windows-sys` and a TOML parser; Settings additionally pulls a GUI
stack. The whole tree is checked with `cargo audit`; as of the last run the only advisories
were in a Linux-only subtree that never compiles for the Windows target. A parser flaw in a
dependency reachable from `config.toml` would be a real vector, which is why the dependency
gate is part of the release checklist rather than a one-off.

## Logging

Two separate paths, deliberately not merged:

- `wiradesk.log` — user-facing Tier-2 warnings, one timestamped line each, opened and closed
  per line so the file is never held. There is no log rotation in this version; the file
  grows until deleted.
- Debug trace — `#[cfg(debug_assertions)]` only, absent from release builds.

Neither records keystroke content. Both live under `%APPDATA%` at medium integrity, so treat
them as readable by anything running as the user.

## How the unsafe surface is held to a standard

The daemon is FFI-heavy by nature, so this is where mistakes would be both easy and
expensive. Every `unsafe` block in the workspace carries a `SAFETY:` comment stating the
precondition it relies on, and that is enforced by the compiler rather than by review:
`undocumented_unsafe_blocks` and `missing_safety_doc` are set to `deny` in the workspace
lints, so a new undocumented block is a build failure. The comments are expected to state
the actual obligation — buffer capacity against the size argument passed to Win32, which
component owns a handle and where it is released exactly once, which thread a call may run
on — not to restate what the function does.

## Residual risks

Stated because a threat model that lists only what it solved is not usable.

1. **Auto-start plus a user-writable install directory is an unprompted elevation path.**
   Not fixable in code; it depends on where the user installs the binary.
2. **A hostile `config.toml` write influences elevated behaviour.** Bounded to typed fields
   with no path or command among them, but not nil.
3. **Releases are unsigned.** See `SECURITY.md` for what can be verified today.
4. **No log rotation.** `wiradesk.log` grows without bound until deleted.
5. **The COM virtual-desktop path is minimally exercised.** `context/virtual_desktop.rs`
   declares the `IVirtualDesktopManager` vtable by hand because no binding ships for it.
   Slot order and both GUIDs are pinned by tests, and every failure returns "unknown" so the
   caller fails closed, but the interface layout remains an assumption about an undocumented
   shell interface.
6. **Denial of the shortcut is cheap.** Squatting the mutex name, or writing a bypass list
   that matches everything, disables the feature. Neither grants privilege.

Previously listed here and now closed: `WM_APP_CONFIG_SNAPSHOT` no longer carries a pointer,
so the handler cannot be handed an address it did not create. See
[Window-message IPC](#window-message-ipc) for what replaced it.

## Out of scope

- An attacker who is already an administrator on the machine. Every boundary here assumes
  the attacker is not.
- Physical access, malicious hardware, and firmware.
- Other software's own keyboard hooks. Hooks compose in an OS-defined chain; Wira Desk
  passes events it does not claim to the next hook and cannot police what else is installed.
- Windows' own handling of foreground rights, which the daemon requests but does not
  control.

## Reporting

See `SECURITY.md`.
