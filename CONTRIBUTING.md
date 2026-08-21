# Contributing

Contributions are welcome via pull request.

## Before submitting

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The test suite needs one environment variable. The daemon links an elevation manifest,
which would otherwise apply to the test harness too and make it fail before any test
ran:

```powershell
$env:WIRADESK_SKIP_MANIFEST = '1'
cargo test --workspace
```

## What CI enforces

Four gates, all of which must pass:

- **fmt, clippy `-D warnings`, tests, release build** on Windows.
- **Secret scan** â€” `gitleaks` over both history and the working tree, using
  `.gitleaks.toml`. Do not widen that allowlist to make a finding go away; triage it.
- **Dependencies** â€” `cargo-deny check` (advisories, licences, bans, sources) using
  `deny.toml`, whose graph is pinned to `x86_64-pc-windows-msvc`. A new dependency
  carrying a licence outside the allow-list will fail, deliberately.

## Unsafe code

This is a Win32 FFI codebase, so `unsafe` is unavoidable â€” undocumented `unsafe` is
not. `undocumented_unsafe_blocks` and `missing_safety_doc` are `deny` in the workspace
lints, so every `unsafe` block needs a `SAFETY:` comment and the build fails without
one.

Write the precondition the block actually relies on, not a description of the call.
The useful ones usually name a buffer capacity against the size argument passed to
Win32, which component owns a handle and where it is released exactly once, which
thread the call must run on, or why an invalid handle is tolerable because the API
reports failure rather than faulting. `docs/threat-model.md` explains why this
boundary is the one worth holding to a standard.

## Repository conventions

`CLAUDE.md`, `AGENTS.md`, and `.cursorrules` hold the instructions AI coding tools read, and
they are byte-identical on purpose â€” three tools, three filenames, one set of rules. Change
one and change all three in the same commit; drift between them is silent, and the tool that
reads the stale copy is the one that misbehaves.

`.constitution/project/constitution.md` records how work is conducted here. `3p.md` and `docs/3p.md`
track progress for code and documentation respectively â€” read the relevant one before starting
and add an entry when finished.

## Planning with BMAD and WDI Method

`_bmad-output/` holds the planning archive from [BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD).
Read `_bmad-output/README.md` first â€” it explains what is historical and what was redacted.

The tooling **is committed** so contributors get the same agent setup without re-running the
installer:

| Path | Purpose |
| --- | --- |
| `_bmad/` | BMAD configuration, team overrides in `_bmad/custom/`, and `config.user.toml` |
| `.claude/skills/` | Skills for Claude Code |
| `.agents/skills/` | Skills for Cursor and Antigravity |
| `.work/` | Ephemeral agent workspace (committed when present) |
| `.constitution/` | WDI Method guides plus product rules |
| `.control/` | Registries and structure maps |

After cloning, install or refresh only if versions drift:

```powershell
npx bmad-method install
npx wdi-method update --yes
```

Contributing to the code does not require running those commands.