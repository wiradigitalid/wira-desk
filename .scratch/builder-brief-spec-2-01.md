# Builder brief — SPEC-2-01

You are the **builder** for one ticket under an active WDI autopilot mandate (`DEC-016`). The
**coordinator** (a separate Claude Code session) dispatched you; it will review your work independently
and never trusts your own report — only the diff, the test output, and the full suite.

## Ticket

Read `.scratch/spec-2-shortcuts-pane-fixes/issues/01-commit-on-departure-and-stepper-parity.md` in full —
that file is the actual ticket (component `settings`, satisfies `UC-4`, `UC-9`). Do not paraphrase it from
this brief; read it directly.

## What to do — Phase 3, Steps 1 and 2 of the `wdi-build` skill

1. **Step 1 (encode):** Invoke the `tdd` skill (`/tdd` or the Skill tool) to write failing tests that
   encode every acceptance-criterion checkbox in the ticket, at the seams already implied by the ticket's
   existing test names (`shortcut_row_slint_snapshot::tests::*`, `persistence::tests::*`,
   `layout_pane_slint_snapshot::tests::*`). Confirm the tests are actually red, for the right reason,
   before moving on.
2. **Step 2 (build):** Invoke the `implement` skill (`/implement` or the Skill tool) to make those tests
   green. It will call `/code-review` on itself internally — that is expected and does NOT replace the
   separate review the coordinator will dispatch afterward.
3. Run the **full workspace test suite green once**, not just this ticket's own tests — the commands are
   in `.constitution/project/codebase-stack-guide.md`:
   ```
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   $env:WIRADESK_SKIP_MANIFEST = '1'; cargo test --workspace
   ```
4. **Commit to the current branch** (`autopilot/DEC-016`). Do **not** push, do **not** open a PR, do **not**
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
