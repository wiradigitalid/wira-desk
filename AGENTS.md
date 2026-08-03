# Repository Instructions â€” Wira Desk

Read this before making a decision, answering a question, running a command, or writing a
file.

## Verify claims against artifacts

The single most valuable habit in this repository, kept from an earlier and much heavier
process that was otherwise retired: **do not accept a report that something was done â€” check
it.** Run the gate. Read the diff. Open the file. A report is evidence of intent, not of
outcome.

This is not theoretical here. It caught a dead-code constant that would have made an entire
rename a silent no-op, a sanitiser that had never removed anything but had corrupted a
published source file, and a gate that could not pass in the place it was written.

## Unsafe code

This is a Win32 FFI codebase, so `unsafe` is unavoidable â€” undocumented `unsafe` is not.
`undocumented_unsafe_blocks` and `missing_safety_doc` are `deny` in the workspace lints, so a
new block without a `SAFETY:` comment fails the build.

Write the precondition the block actually relies on, not a description of the call. See
`CONTRIBUTING.md` for what a useful one looks like, and `docs/decisions.md` for why this
boundary is the one held to a standard.

## Progress tracking

Two trackers record what happened and what is next, in Progress / Plans / Problems form:

| Tracker | Scope |
| --- | --- |
| `3p.md` | Code, assets, and non-Markdown configuration |
| `docs/3p.md` | Documentation |

At the start of a turn, read the tracker for the area you are about to touch, so work is not
repeated. After finishing, add an entry to it. Keep entries specific: what changed, and why
it was not obvious.

## Before you finish

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
$env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
```

The environment variable is required. The daemon links an elevation manifest that would
otherwise apply to the test harness too, which then cannot launch at all.

## Publication hygiene

`scripts/verify-public-export.ps1` enforces what may appear in this repository: no local
machine paths, no unapproved product claims, no internal requirement identifiers in product
source. If it reports a finding, fix the source. Widening a pattern to make a finding
disappear is the failure mode that file exists to prevent.