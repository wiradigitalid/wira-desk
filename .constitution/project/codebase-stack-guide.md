---
status: Accepted
ratified_by: c803a1d     # the last commit that changed `crates/` — the code this file describes
---

# stack — codebase guide

**Loaded when:** writing or reviewing code.

Filled by the distillation of wave W1 from the code at `67f2645`. Every figure here was read from a
manifest or a source file, not carried over from a plan.

## The commands

Run from the repository root. All three are what CI runs, and the third is the one people get wrong.

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
```

`WIRADESK_SKIP_MANIFEST` is **required**, not optional. The daemon links an elevation manifest
(`requireAdministrator`); without the variable that manifest applies to the test harness too, and the
harness then cannot launch at all — the failure looks like a broken toolchain rather than a missing
environment variable, which is why it is stated first here.

Release build: `./build.ps1`. The release profile is `lto = true`, `opt-level = "z"`,
`strip = true`, `panic = "abort"`, `codegen-units = 1`.

There is a second profile, `release-metrics`: release codegen with `debug-assertions` on, because
every metric seam in the daemon is `#[cfg(debug_assertions)]`-gated and could otherwise only be
measured on a build no user runs. `overflow-checks` is pinned off in it deliberately. **It is never
shipped** — `build.ps1` and the release gate both use `--release`.

## Shape

A single Cargo workspace, three crates, `resolver = "2"`, edition 2021, target
`x86_64-pc-windows-msvc`:

| Crate | Binary | Holds |
| --- | --- | --- |
| `daemon` | `wiradesk.exe` | The hook thread, worker, tray, arrangement, health, autostart |
| `settings` | `wiradesk-settings.exe` | The Slint settings window and the first-run tutorial |
| `shared` | — | `Config` TOML types, the `u8` command enum, every constant, `%APPDATA%` paths |

Both binaries depend on `shared`, and nothing depends on a binary. A type used by both sides belongs
in `shared` — the reason is not tidiness: `ONBOARDING_FLAG` and the `WM_APP_*` values are a contract
between two processes, and a typo in one of them must be a compile error rather than a feature that
silently never fires.

## Dependencies, as pinned

| Crate | Version | Note |
| --- | --- | --- |
| `windows-sys` | 0.52 | Raw C-FFI only. The full `windows` crate's COM metadata is deliberately avoided |
| `slint` | 1.17 | `settings` only, `default-features = false` plus `backend-winit`, `renderer-skia`, `accessibility`, `compat-1-2`. Paired with `i-slint-backend-winit`, `slint-build` (a build dependency), and `i-slint-backend-testing` (a dev dependency) — bump all four together, never one |
| `winit` | 0.30 | `settings` only, and it MUST match the version `i-slint-backend-winit` 1.17 resolves to; the window handle the titlebar and elevation path need is reached through it |
| `toml` | 1.1 | Moved from 0.8 by #5 |
| `serde` | 1.0 | `derive`. `settings` also carries `serde_json` for the update check |

Slint is declared with `default-features = false`, so **every** feature above is load-bearing and
none is a leftover. `accessibility` is the one to understand before touching this list: without it the
UI Automation tree is never published, every accessibility criterion fails silently — passing a manual
look while failing a screen reader — and, because the snapshot tests below drive controls through
`ElementHandle::find_by_accessible_label`, the whole `settings` test suite stops being able to see the
window at all. Removing it is not a dependency cleanup.

## The UI is `.slint` markup, not Rust widget calls

`crates/settings/ui/` holds the markup — `main_window.slint`, `theme.slint`, `onboarding.slint`, one
file per pane under `panes/`, and reusable controls under `components/`. `crates/settings/build.rs` compiles
them through `slint-build`, which generates the Rust types `crates/settings/src/` then binds to; a control that does not
exist in the markup cannot be reached from Rust, and vice versa.

Two consequences worth stating, because both have already cost a return trip:

- **A pane is declared in two places.** The markup file and the pane enum in `crates/settings/src/app.rs` must agree,
  and the nav order is the declared order — not the alphabetical one.
- **Tests drive the real window, not a mock.** The `crates/settings/src/*_slint_snapshot.rs` modules boot
  `i-slint-backend-testing`'s `TestingBackend` and locate controls by accessible label, so a control
  without an `accessible-label` is untestable by construction. That harness sets values through the
  accessible-value setter, which is a *different code path* from a physical keystroke reaching a
  `TextInput` — the gap that let `DEF-5` ship. A test asserting typed input must say so explicitly.

The one COM exception is `IVirtualDesktopManager`, hand-written as a minimal vtable in
`crates/daemon/src/context/virtual_desktop.rs` rather than pulled in through a wrapper crate.

## Workspace lints — a compiler gate, not a review habit

```toml
[workspace.lints.clippy]
undocumented_unsafe_blocks = "deny"
missing_safety_doc = "deny"
```

`deny`, so a new `unsafe` block without a `SAFETY:` comment is a **compile error**. The level was
raised from `warn` the moment the backlog reached zero, because a warn-level gate sitting at zero is
one careless commit away from being a gate in name only.

Write the precondition the block actually relies on, not a restatement of the call. For this codebase
that is usually one of: buffer capacity matching the size argument handed to Win32; who owns a handle
or `Box` and where it is released exactly once; which thread a call must run on; or why an invalid
handle is tolerable because the API reports failure instead of faulting.

## Where the numbers live

Every tunable is a named constant in `crates/shared/src/constants.rs`, never a literal at a call
site: `RING_BUFFER_CAPACITY = 16`, `ANTI_MACRO_THROTTLE_MS = 50`, `HOOK_HEARTBEAT_SECS = 10`,
`HOOK_CHECK_FAIL_THRESHOLD = 3`, `HOOK_RETRY_MAX = 5`, `TASK_NAME = "WiraDesk"`,
`ONBOARDING_FLAG = "--onboarding"`, `DAEMON_WINDOW_CLASS`, `DAEMON_WINDOW_TITLE`, and the
`WM_APP_*` message set.

Changing one of these changes something a user feels, and several are quoted verbatim in the
architecture spine. A new value belongs in a decision before it belongs in this file.
