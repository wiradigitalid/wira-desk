---
status: blocked
---

# BMad Dev Auto Result

Status: blocked
Blocking condition: unclear intent

## Auto Run Result

Invocation was `/bmad-dev-auto` only (2026-07-23), with no spec file path, story/ticket ID, or free-form implementation intent.

Per step-01, the workflow cannot infer what to implement from sprint status or artifact listings alone. `workflow.on_complete` resolved empty — no extra terminal step.

## Suggested next invocation

Point the skill at one of:

- An existing spec: path to `spec-*.md` with frontmatter `status` (`draft`, `ready-for-dev`, `in-progress`, etc.)
- An epic story ID: e.g. `2-1-asynchronous-keyboard-hook-foundation` (next story after Epic 1; still `backlog` in `sprint-status.yaml`, story file already present)
- A story file: e.g. `_bmad-output/implementation-artifacts/2-1-asynchronous-keyboard-hook-foundation.md`
- Free-form intent describing a single shippable goal

Example: `/bmad-dev-auto 2-1-asynchronous-keyboard-hook-foundation`
