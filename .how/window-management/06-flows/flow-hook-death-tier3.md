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
        Hook-->>Health: handle invalid
    end
    Health->>Hook: attempt re-register
    alt Re-register succeeds
        Hook-->>Health: new handle
        Health->>Tray: set_tier(normal)
    else Re-register fails (threshold)
        Health->>Worker: escalate Tier 3
        Worker->>Tray: set_tier(critical)
        alt toast_sent == false
            Tray->>User: one balloon toast
            Tray->>Tray: toast_sent = true
        end
        Note over User: Cycling shortcuts no longer work until daemon restart
    end
```

## Postconditions

- User sees critical tray overlay (X) and at most one toast.
- Daemon process remains alive; user may open Settings or View Logs from tray.
- No repeated toast spam on subsequent heartbeat failures.
