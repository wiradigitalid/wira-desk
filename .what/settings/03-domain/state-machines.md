# State Machines — settings

## Onboarding state (`onboarding-completion`)

```mermaid
stateDiagram-v2
    [*] --> Pending: No config.toml
    Pending --> InTutorial: --onboarding launch
    InTutorial --> Completed: Tutorial finished
    InTutorial --> Skipped: Skip Tutorial clicked
    Completed --> [*]: Settings closes
    Skipped --> [*]: Settings closes
    Pending --> [*]: Existing config (skip onboarding)
```

| State | Next launch behaviour |
| --- | --- |
| Pending | Daemon launches settings with `--onboarding` |
| InTutorial | User must complete or skip |
| Completed / Skipped | Tray-only daemon; Settings opens to main dialog |

## Shortcut capture state (`LC-shortcut-capturer`)

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Listening: Field focused / Listen clicked
    Listening --> Idle: Valid chord captured
    Listening --> Idle: Escape cancels
```

Listening mode ignores typed characters; only physical key combinations are accepted (FR-18).
