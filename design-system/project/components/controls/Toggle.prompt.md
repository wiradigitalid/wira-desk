# Toggle

On/off switch. WinTick's Settings window is rows of these (enable snapping, auto-start, overlapping stack…). Pass `label`/`description` to render the whole settings row; omit both for a bare switch.

```jsx
<Toggle checked={on} onChange={setOn} label="Enable window snapping" description="Ctrl + Win + arrows" />
<Toggle native checked={autostart} onChange={setAutostart} label="Start with Windows" />
```

- **native**: Windows-11 Fluent styling for in-product surfaces.
- Accessible: `role="switch"` + `aria-checked`.
