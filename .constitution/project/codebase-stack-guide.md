---
status: Accepted
ratified_by: 67f2645     # the last commit that changed `crates/` — the code this file describes
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
| `settings` | `wiradesk-settings.exe` | The egui settings window and the first-run tutorial |
| `shared` | — | `Config` TOML types, the `u8` command enum, every constant, `%APPDATA%` paths |

Both binaries depend on `shared`, and nothing depends on a binary. A type used by both sides belongs
in `shared` — the reason is not tidiness: `ONBOARDING_FLAG` and the `WM_APP_*` values are a contract
between two processes, and a typo in one of them must be a compile error rather than a feature that
silently never fires.

## Dependencies, as pinned

| Crate | Version | Note |
| --- | --- | --- |
| `windows-sys` | 0.52 | Raw C-FFI only. The full `windows` crate's COM metadata is deliberately avoided |
| `eframe` + `egui` | 0.35 | `settings` only, and `eframe` carries `features = ["accesskit"]` |
| `ttf-parser` | 0.25 | Validates a system font's bytes before egui is asked to load them |
| `toml` | 1.1 | Moved from 0.8 by #5 |
| `serde` | 1.0 | `derive` |

`accesskit` is **not** an eframe default feature. Without it the UI Automation tree is never
published and every accessibility criterion fails silently — passing a manual look while failing a
screen reader. It is requested explicitly for that reason, and removing it is not a dependency
cleanup.

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
