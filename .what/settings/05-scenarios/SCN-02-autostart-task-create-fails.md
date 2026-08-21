# SCN-02 — Auto-start task creation fails

**Parent UC:** UC-6  
**Actor:** Power User  
**Trigger:** User enables auto-start but Task Scheduler rejects task creation.

## Preconditions

- Settings General tab is visible.
- Auto-start is currently off.
- User account lacks permission to create ONLOGON tasks, or `schtasks` returns non-zero.

## Steps

1. User toggles Auto-start on.
2. `LC-config-writer` invokes `schtasks /Create` with `/RU %USERNAME%` and `/RL HIGHEST`.
3. Command fails (access denied or policy block).
4. Writer logs warning with exit code.
5. UI reverts toggle to off and shows inline error.
6. `config.toml` `auto_start.enabled` remains `false`.

## Postcondition

- No orphaned scheduled task exists.
- User can retry after gaining permission or running Settings elevated if policy allows.

## Failure envelope

| Condition | User sees | Log |
| --- | --- | --- |
| `schtasks` failure | Toggle reverts + error banner | `warn!` with exit code |
