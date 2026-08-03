# ShortcutInput

The "Listening mode" shortcut capturer from WinTick Settings (FR-18). Click (or focus + Enter) to arm; it records the next physical key combo and never accepts typed text. Press `Esc` to cancel.

```jsx
const [combo, setCombo] = React.useState(["Win", "`"]);
<ShortcutInput value={combo} onChange={setCombo} native />
```

- Controlled: pass `value` (array of labels) + `onChange`.
- Renders the captured combo with `<Keycap>`; shows a pulsing "Listening…" state while armed.
