# Flow — Hook death Tier 3

**Component:** window-management  
**Realizes:** UC-3 (error path), AD-7, AD-8, SCN-02

## Sequence

```mermaid
sequenceDiagram
    participant Health as health.rs heartbeat
    participant Hook as LC-hook-thread
    participant Worker as LC-worker-thread
    participant Tray as LC-tray-controller
    participant User

    loop Every 10 seconds
        Health->>Hook: WM_APP_HOOK_CHECK
        Hook->>Hook: SetWindowsHookExW (re-register on the hook thread)
        alt Re-register succeeds
            Hook->>Worker: WM_APP_HOOK_REFRESH_OK
            Worker->>Tray: set_tier(normal or warning if warning_latched)
            Worker->>Worker: toast_sent = false
        else Re-register fails, fail_count < 3
            Hook->>Hook: increment fail_count, retain prior HHOOK
        else Re-register fails, fail_count >= 3
            Hook->>Worker: WM_APP_HOOK_DEAD
            Worker->>Tray: set_tier(critical)
            alt toast_sent == false
                Tray->>User: one balloon toast
                Tray->>Tray: toast_sent = true
            end
        end
    end
    Note over Health,User: The heartbeat never stops. A later tick that succeeds restores Normal<br/>with no restart, which is why the toast latch is reset rather than left set.
```

## Postconditions

- User sees critical tray overlay (X) and at most one toast.
- Daemon process remains alive; user may open Settings or View Logs from tray.
- No repeated toast spam on subsequent heartbeat failures.
- Recovery needs no restart: the next successful heartbeat re-registers the hook, resets `hook_dead_toast_sent`, and returns the icon to `Normal` (or `Warning` when `warning_latched`). See SCN-02 steps 8-11 and `state-machines.md` `Dead -> Active`.
