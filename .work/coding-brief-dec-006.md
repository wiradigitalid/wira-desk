# Coding brief — DEC-006, arrangement must never target a Wira Desk window

Scratch, and a convenience copy. The authority is
`.control/decisions/DEC-006-arrangement-never-targets-a-wira-desk-window.md`, which is `status: applied`
as of 2026-08-26, together with `LBR-WM-6` in `.what/window-management/02-rules/rules-window-management.md`
and `LBR-ST-13` in `.what/settings/02-rules/rules-settings.md`.

This file exists so the brief's substance can travel inside a dispatch prompt rather than by reference.
A worker in a separate worktree or a separate session gets everything it needs from the prompt text
itself; it must not be expected to read the conversation this came from.

## State when this was written

- `DEC-006` is `applied`, with `touches:` naming the six documents it changed.
- The corpus carries the rules: `LBR-WM-6` (arrangement target eligibility), `LBR-ST-13` (the
  painted-size invariant), one SRS constraint, one `UC-2` alternate flow, and a note each in
  `LC-arrangement-engine` and `LC-settings-shell`.
- **Nothing in `crates/` has changed yet.** That is the whole of the work below.
- Everything the brief names must be **committed** on the branch a worker's worktree is cut from. A
  worktree branches from a commit, and a worker that cannot find its brief writes itself a replacement
  and works from the guess.

## The problem, stated once

`crates/daemon/src/arrangement/win32.rs` resolves its arrangement target from `GetForegroundWindow()`
and filters it for validity, cloaking, monitor, and DPI — never for who owns it. So when the Wira Desk
Settings window holds the foreground and a snap or stack chord fires, `SetWindowPos` grows *that*
window's frame to half-screen or full-screen.

The Settings window is frameless, transparent, and laid out at a fixed size
(`crates/settings/ui/main_window.slint`: `no-frame`, `background: Colors.transparent`, `width`/`height`
bound to 760×610, or 580×380 while onboarding). The layout keeps painting at its fixed size in the
top-left corner. The rest of the enlarged frame is invisible and still owns its hit-test area, so it
swallows mouse clicks over whatever is behind it.

Three things were verified while deciding, so nobody re-derives them:

- Slint marks the window non-resizable and installs min/max inner size when layout constraints are
  fixed. winit turns that into `ptMinTrackSize`/`ptMaxTrackSize` in `WM_GETMINMAXINFO`. **Those fields
  do not constrain `SetWindowPos`** — they govern user drag-resize tracking and maximize. They are
  already set on this window and the bug happens anyway.
- winit's `WM_WINDOWPOSCHANGING` handler only tracks which monitor a fullscreen window moved to. It
  never inspects or clamps `cx`/`cy`.
- `arrangement/win32.rs` holds the only `SetWindowPos` in the workspace, which is what makes a single
  gate in `resolve_context_for` cover all four arrangement commands.

---

# Task A — the daemon stops targeting its own windows

Ships first. It removes the reported symptom and adds no `unsafe` beyond the module's existing idiom.

## Files in scope

- `crates/daemon/src/arrangement/win32.rs` — the only file that must change.
- `3p.md` — the code tracker, one entry.
- Read only: `crates/shared/src/constants.rs`, where `SETTINGS_EXE_NAME` already lives.

## What to build

A target-ownership gate inside `resolve_context_for`, placed **after** the cloaked guard and **before**
`MonitorFromWindow`, so the sequence reads validity → visibility → ownership → geometry.

The gate returns `None` — exactly as the existing guards do, with a `#[cfg(debug_assertions)]` trace in
the format the neighbouring guards use — when either holds:

1. The target's process id equals the daemon's own (`GetWindowThreadProcessId` vs `GetCurrentProcessId`).
2. The basename of the target's process image equals `SETTINGS_EXE_NAME`, compared case-insensitively.

Identity is read with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, …)` +
`QueryFullProcessImageNameW(PROCESS_NAME_WIN32)` and nothing else. No `SendMessage`, no
`GetWindowText`, no blocking cross-process call — `LBR-WM-3` binds this path.

**Fail open, not closed.** If the identity cannot be read, treat the window as *not* ours and let the
arrangement proceed. This mirrors the module's existing posture, where a failed `DwmGetWindowAttribute`
degrades to "not cloaked". Write the reason in the comment: failing closed would silently kill snapping
for any window whose process cannot be opened, and Task B is what covers the residual false negative.

## Constraints that bind

- **Do not import from `crate::cycling`.** That module is deliberately decoupled from this one —
  `is_cloaked` is duplicated locally rather than imported, and the file header says why. Duplicate the
  basename comparison the same way, with the same kind of comment.
- **Do not use the window class**, which the UI toolkit registers generically, and **do not use the
  window title**, even though `crates/settings/src/main.rs` does for its single-instance check — any
  window titled "Wira Desk" would match.
- **Do not add a registration channel.** A process id the Settings process reports to the daemon was
  refused in the decision; `OQ-17` in `.control/questions/assumptions.md` records why.
- **Do not touch `crates/daemon/src/hook.rs`.** The chord must stay swallowed there. Passing it through
  would fire Windows' previous-virtual-desktop action, because `reservation()` in
  `crates/shared/src/shortcut.rs` deliberately does not reserve `Ctrl+Win+Arrow`.
- **Do not add a second gate in `execute_stack`.** `crates/daemon/src/worker.rs` already returns early
  when `resolve_context()` yields `None`, before it enumerates anything. One gate is the whole change.
- **Every new `unsafe` block needs a real `SAFETY:` note** stating the precondition the block relies on,
  not a description of the call. `undocumented_unsafe_blocks` is `deny` at workspace level, so the build
  fails without it. Match the register of the notes already in that file.
- **Allocation:** one `vec![0u16; …]` path buffer per arrangement command is fine, and should be
  commented as deliberate. The buffer-reuse machinery in `crates/daemon/src/cycling/source.rs` exists
  because that path allocates once *per window* across a sweep of hundreds; this path runs once per
  command on the Worker thread. Say so, so a later reviewer does not "fix" it.
- **No spine decision identifiers or process jargon in Rust source.**
  `scripts/verify-public-export.ps1` fails the build on them inside `crates/`. Citing `DEC-006` or
  `LBR-WM-6` is fine; citing a spine `AD-` identifier is not.

## Tests

Extract the comparison as a pure function over a UTF-16 slice — basename extraction plus
case-insensitive match — and unit-test it in the file's existing `mod tests`. Cover at minimum:

| Input | Expected |
|---|---|
| `C:\Program Files\Wira Desk\wiradesk-settings.exe` | ours |
| the same path with forward slashes | ours |
| bare `wiradesk-settings.exe`, no separator | ours |
| `WIRADESK-SETTINGS.EXE` | ours |
| `wiradesk-settings-helper.exe` | **not** ours |
| `notepad.exe`, and an empty slice | **not** ours |

The Win32 half stays untestable without a live window; that gap is already disclosed in the same file
and needs no new apology. Keep `context_for_invalid_window_is_none` green.

**The guard must be seen failing.** Invert the comparison, watch the new tests go red, restore it. Use
`cargo test --workspace --no-fail-fast` for that run — a runner that stops at the first failure fakes
the same evidence a never-red test does.

## Done means

- The new tests pass, and were observed red under the inverted guard.
- `cargo fmt --all` clean.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace` green. The variable is required: the
  daemon links an elevation manifest that would otherwise apply to the test harness, which then cannot
  launch at all.
- `powershell -File scripts/verify-public-export.ps1 -Path .` reports no *new* finding. The tree is
  already red on pre-existing ones. Fix the source, never widen a pattern.
- An entry in `3p.md` — the code tracker, not `docs/3p.md` — recording what changed and the part that is
  not obvious: the gate is on the Worker rather than in the hook, and the disposition is a swallow
  rather than a passthrough.
- Nothing changed outside `crates/daemon/src/arrangement/win32.rs` and `3p.md`.

---

# Task B — the Settings window's painted size becomes its Win32 size

Sequenced **after** Task A, and gated on `OQ-19` in `.control/questions/assumptions.md` being settled:
whether any third-party window manager on the owner's machine actually resizes a non-resizable
frameless window through raw `SetWindowPos`. That question is the entire justification for paying for
this task on top of Task A. If it comes back negative, this task does not ship, and `LBR-ST-13` waits.

## Files in scope

- `crates/settings/src/` — a new small module, so `main.rs` does not grow another concern.
- `crates/settings/Cargo.toml` — `windows-sys` needs the feature carrying `SetWindowSubclass`; it is
  not in the current feature list.
- `3p.md` — one entry.

## What to build

Subclass the Settings window and clamp size changes at `WM_WINDOWPOSCHANGING`, before the toolkit's own
handler sees them. Position changes pass through untouched; only size is clamped. The HWND is reachable
from the Slint side through the winit window — `crates/settings/src/main.rs` already reaches into it
with `with_winit_window` for `drag_window`.

## The trap, named so it is not discovered in review

**A blanket clamp breaks the onboarding transition.** The same window is 580×380 while onboarding and
760×610 afterwards, and that resize is driven by Slint through the same `SetWindowPos` path an external
caller uses. A guard that refuses every size change refuses that one too, and the onboarding modal
never grows into the settings shell. `LBR-ST-13` states both halves for exactly this reason.

So the clamp must distinguish a size the window itself currently declares legal from one imposed on it.
Two mechanisms are acceptable; pick one, and write down in the code why you picked it:

- **Clamp against the window's own declared track size.** Obtain `ptMinTrackSize`/`ptMaxTrackSize` as
  the window currently declares them, and clamp `cx`/`cy` into that range. Self-maintaining: when the
  layout's constraints change, the legal range changes with them, so the onboarding transition passes
  by construction. Risk to check before committing to it — whether obtaining those values from inside
  the `WM_WINDOWPOSCHANGING` handler re-enters the toolkit's own state lock.
- **Clamp against an intended size the Rust side owns**, updated in the same place the onboarding flag
  is flipped and reached from the subclass through its reference data. No re-entrancy, at the cost of a
  second place that knows the window's sizes — a drift risk worth a comment.

Do not solve this by removing the fixed size and making the window resizable. That option was
considered and refused in `DEC-006` on cost: every pane would need reflow rules at arbitrary sizes, and
that is a UX wave, not a guard.

## Constraints that bind

- One `unsafe` boundary, and a `SAFETY:` note that states the real precondition — including the
  subclass's lifetime relative to a window the toolkit owns and destroys.
- Chain to the default subclass handler for every message not clamped.
- Do not touch any `.slint` file. This task changes no layout and no visual metric.
- Do not repeat Task A's gate here. These are two layers of one decision, not two copies of it.

## Tests, and what cannot be tested

Both properties must be proven, and the second is the one that gets skipped:

1. An external size change is clamped — the window's outer size is unchanged after an external
   `SetWindowPos` asking for half-screen.
2. **The onboarding transition still resizes.** 580×380 → 760×610 must still happen.

Neither is a unit test; both need a live window. That is UI-driving work and must not be run by the
orchestrator — one desktop has one keyboard focus and one accessibility tree, which filesystem
isolation does not duplicate. Write the raw captures and a summary to disk under `.work/` so the result
is verifiable from artefacts rather than from a report. Any pure helper the clamp uses — the range
arithmetic — gets ordinary unit tests, and gets seen failing.

`OQ-18` is open on the first property: whether a clamp at `WM_WINDOWPOSCHANGING` actually holds against
an `SWP_ASYNCWINDOWPOS` request, which is how the daemon posts. Settling it is part of this task, and
the way to settle it is to break the guard deliberately and watch the ghost frame return.

## Done means

Everything under Task A's *Done means*, plus the two live properties captured to disk, plus a `3p.md`
entry recording which of the two clamp mechanisms was chosen and why the other was not.

---

## Out of scope for both tasks

- The hook, the ring, the throttle, the VM/RDP bypass.
- Any `.slint` file, any visual metric, any copy.
- Making the Settings window resizable.
- The stale toolkit description in the corpus — the spine's Settings-binary entry and
  `LC-settings-shell` still describe an egui/eframe presentation layer while the crate is Slint 1.17
  with the Skia renderer. Real drift, recorded in the memlog, and it belongs to `wdi-reconcile`. Do not
  fix it here and do not let it widen this change set.
