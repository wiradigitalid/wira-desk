# Builder brief — SPEC-3-02

You are the **builder** for one ticket under an active WDI autopilot mandate (`DEC-016`). The
**coordinator** (a separate Claude Code session) dispatched you; it will review your work independently
and never trusts your own report — only the diff, the test output, and the full suite. Note: `/implement`
calling `/code-review` on itself internally is expected but does NOT count as that independent review —
the coordinator dispatches a genuinely separate one after you finish.

## Ticket

Read `.scratch/spec-3-per-action-shortcut-enable-disable/issues/02-hook-registration-exclusion.md` in
full — that file is the actual ticket (component `window-management`, satisfies `UC-12`, `FR-29`). Also
read `.scratch/spec-3-per-action-shortcut-enable-disable/SPEC.md` for the broader contract. Do not
paraphrase either from this brief; read them directly.

**One checkbox is likely already satisfied**: `SPEC-3-01` (already closed, on this same branch) removed
`crates/daemon/src/arrangement/stack.rs`'s check against the retired `layout.enable_overlapping_stack`
field as part of its own commit — confirm this is genuinely still true (no reference to that field remains
anywhere in `crates/daemon/`) rather than re-doing the removal.

## What to do — Phase 3, Steps 1 and 2 of the `wdi-build` skill

1. **Step 1 (encode):** Invoke the `tdd` skill (`/tdd` or the Skill tool) to write failing tests that
   encode every remaining acceptance-criterion checkbox in the ticket, at the seams the ticket's own test
   list implies (`hook::tests::*`, `arrangement::stack::tests::*`). Confirm the tests are actually red, for
   the right reason, before moving on.
2. **Step 2 (build):** Invoke the `implement` skill (`/implement` or the Skill tool) to make those tests
   green.
3. **Bundled cleanup (not a checkbox, but cheap while you're in this area):** `crates/settings/src/theme.rs`
   still defines `pub const TOGGLE_OVERLAPPING_STACK` (a `ControlSemantics` accessible-name entry), orphaned
   since `SPEC-3-01` removed the LayoutPane control it described. Delete the constant and its two remaining
   references in `theme.rs`'s own inventory tests (`every_control_has_a_non_empty_accessible_name`,
   `accessible_names_are_unique`) — confirmed by two independent review agents as real dead code, not
   required by any ticket checkbox.
4. Run the **full workspace test suite green once**, not just this ticket's own tests — the commands are
   in `.constitution/project/codebase-stack-guide.md`:
   ```
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   $env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
   ```
5. **Commit to the current branch** (`autopilot/DEC-016`). Do **not** push, do **not** open a PR, do **not**
   merge anything — the coordinator is the only hand that pushes.

## Three rules that bind every builder in this corpus

- **Debugging is conditional, never a phase.** If a test or build fails and you don't know why, invoke
  `wdi-systematic-debugging` before proposing any fix. A third failed fix attempt means stop and report —
  do not try a fourth variant.
- **The corpus is not yours to change.** Do not edit anything under `.what/`, `.how/`, or an `applied`
  `DEC-`. If the ticket's acceptance criteria turn out to contradict what the code should actually do,
  stop and report that conflict verbatim instead of quietly resolving it.
- **Verification is run, not assumed.** Actually run the commands above; do not report a suite as green
  without having run it in this session, in this worktree.

## Boundaries

- You are working in an isolated git worktree already checked out to `autopilot/DEC-016`. Nobody else
  builds here concurrently — you have it exclusively until you finish.
- Do not touch any other ticket, any other spec, or the registries (`specs.yaml`, `decisions.yaml`,
  `index.yaml`, `components.yaml`) — that is the coordinator's job.
- When you are done (or stuck at the third failed fix, or blocked on a real scope question), stop and
  write one final message summarizing: which step you reached, what's committed (commit SHA), and
  whether the full suite is green. Do not keep going past that point.
