# Constitution

Operating rules for this repository. Architecture and implementation decisions are recorded
in `docs/decisions.md`; this document covers only how work is conducted.

## I â€” Verification over assertion

A claim MUST be verified against its artifact. Running the gate, reading the diff, and
opening the file are the evidence; a summary is not. This rule replaced a far heavier
multi-stage review process, and it is the part of that process that actually caught defects.

## II â€” Progress trackers

Two domains, each with its own tracker: `3p.md` for code, assets, and non-Markdown
configuration, and `docs/3p.md` for documentation. Read the relevant one before working and
update it afterwards. Work outside both domains MUST declare its own tracker and record that
declaration.

## III â€” Agent configuration stays in sync

`CLAUDE.md`, `AGENTS.md`, and `.cursorrules` are read by different tools and MUST stay
byte-identical. Changing one means changing all of them in the same commit.

## IV â€” Secrets

Credentials, keys, and tokens MUST NOT be committed â€” not in code, logs, trackers, or
documentation. Secret-bearing files belong outside the repository. A secret-scan gate runs in
CI over both history and the working tree; if it ever fires, rotate at the source before
anything else.

## V â€” Documented unsafe

Every `unsafe` block carries a `SAFETY:` comment stating its actual precondition, enforced by
the compiler rather than by review. See `CONTRIBUTING.md`.

## Amendments

Amendments are recorded in this file with their reasoning, and noted in `docs/3p.md`.