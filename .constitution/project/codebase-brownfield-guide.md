---
status: Accepted
ratified_by: 67f2645     # the last commit that changed `crates/` — the code this file describes
---

# brownfield — codebase guide

**Loaded when:** writing or reviewing code.

Filled by the distillation of wave W1 from the code at `67f2645`. This product is brownfield in two
distinct ways, and confusing them is the mistake this file exists to prevent.

## The two inheritances

**A renamed product.** Wira Desk was WinTick. The rename is complete in code — every constant, path,
window class, and binary name says Wira Desk — but the *user's machine* may still hold the old
product's state, so the migration path is live code, not history.

**A corpus written after the code.** `.what/` and `.how/` were harvested from BMAD planning artifacts
dated 2026-07-06, well after the implementation existed and, in places, after it had changed. The
corpus is therefore **not** automatically the authority on what the product does.

## The rule that follows, and it is the important one

**When a document and the shipped code disagree, find out which one moved before changing either.**

Wave W1's review found eight places where the corpus described a mechanism the code never had. The
one that proves the point: three documents specified that the tray's "View Logs" opens the log
*folder* via `ShellExecuteW(..., L"explore", ...)`. `git log -S '"explore"'` returns exactly one
commit — the commit that landed the corpus. `menu::view_logs` has spawned `notepad.exe` on the log
file since the initial release. The document was never true.

Had that been "fixed" in the direction the corpus pointed, working, tested, shipped behaviour would
have been rewritten to satisfy a promise nobody had made. So:

- `git log -S '<the literal>'` on the mechanism in question, before proposing a change. It answers
  "which side moved" in one command, and it answered it every time in W1.
- Shipped code covered by a passing test is evidence. A document is a claim. Neither automatically
  wins, but they are not the same kind of thing.
- Correcting the document is the normal outcome here, and it is not a shortcut. Correcting the code
  is a change to a promise, and that goes through `wdi-decision`.

## The WinTick migration, concretely

`crates/shared/src/migrate.rs` performs a one-time copy from `%APPDATA%\WinTick\config.toml` to
`%APPDATA%\WiraDesk\config.toml` when the new path does not yet exist. It is idempotent, and it
**preserves the legacy directory deliberately** so a user can roll back.

That preservation has a consequence worth knowing before touching it: deleting the whole
`%APPDATA%\WiraDesk` folder is not a factory reset, because the migration then re-runs from the
still-present legacy directory. A reset deletes `config.toml` only. `CHANGELOG.md` states this for
users; it is repeated here because it makes the migration look broken to anyone testing a reset.

Covered by `migrate::tests::copies_config_and_preserves_legacy`,
`migrate::tests::idempotent_second_run`, and `migrate::tests::skips_when_new_dir_already_exists`.

## Publication is part of the build

This repository is public and was exported from a private one. `scripts/verify-public-export.ps1`
asserts what may appear here and **fails closed**; CI runs it on every push.

It is an assertion, never a rewrite. An earlier design ran regex substitutions over exported files:
they removed nothing in practice and one of them corrupted a comment in published source. A mechanism
that has never detected anything and has damaged output once is worse than none, so files are
authored publication-clean and the gate verifies that claim.

Two consequences for a new change:

- **Fix the content, never the pattern.** Widening a rule to clear a finding is the failure mode the
  file was written to prevent, and it says so.
- The corpus directories are exempt from the identifier and claims rules because they are where those
  identifiers are *defined*; `crates/`, `docs/`, and the root documents are not. A finding in product
  source is real. `3p.md` and `docs/3p.md` are enforced too — W1 tripped that by writing requirement
  identifiers into a tracker entry, and the fix was to reword the entry.

## What is not in the tests

Two shipped requirements have no automated coverage, both because their observable effect is in
another process: the tray icon returning after Explorer restarts, and View Logs opening the file.
They are `RISK-1` and `RISK-2` in the risk register, verified by hand at release. Do not read their
absence from the suite as absence from the product.
