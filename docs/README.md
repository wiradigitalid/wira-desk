# Documentation

- [`threat-model.md`](threat-model.md) — trust boundaries, why each privilege is required,
  the attack surface, and the risks that remain after mitigation. Start here if you are
  evaluating whether to trust an elevated daemon with a global keyboard hook.
- [`decisions.md`](decisions.md) — why the code looks the way it does: the Win32 behaviours
  that are not what their names suggest, and the obvious-looking alternatives that were tried
  first and broke something. Read it before simplifying the keyboard hook, the cycling order,
  or the activation path.

User-facing policy documents are in the repository root: `README.md`, `SECURITY.md`,
`PRIVACY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`.