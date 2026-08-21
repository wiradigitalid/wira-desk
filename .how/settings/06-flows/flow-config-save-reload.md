# Flow — Config save and daemon reload

**Component:** settings  
**Realizes:** UC-4, BR-1, AD-5, LBR-ST-2

## Sequence

```mermaid
sequenceDiagram
    participant User
    participant Shell as LC-settings-shell
    participant Writer as LC-config-writer
    participant Disk as config.toml
    participant Daemon as daemon hidden window

    User->>Shell: Save
    Shell->>Writer: save(config)
    Writer->>Writer: validate_shortcut (all fields)
    alt validation fails
        Writer-->>Shell: error (SCN-01)
        Shell-->>User: inline message
    else valid
        Writer->>Disk: write temp + atomic replace
        Writer->>Daemon: PostMessage(WM_APP_RELOAD_CONFIG)
        alt FindWindowW null
            Writer-->>Shell: warn daemon not found
        else signal delivered
            Daemon->>Disk: re-read Config
            Daemon-->>User: new shortcuts active (no restart)
        end
        Shell-->>User: save confirmation
    end
```

## Postconditions

- On success, on-disk `config.toml` matches in-memory draft.
- Daemon reloads only after file is complete; partial writes are impossible by ordering.
