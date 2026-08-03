# Keycap

A physical keyboard key set in mono — the core visual motif of WinTick (a shortcut-driven product). Render one key or a whole combo.

```jsx
<Keycap combo={["Win", "`"]} />
<Keycap combo={["Ctrl", "Win", "Enter"]} size="lg" tone="brand" />
<Keycap>Esc</Keycap>
```

- **combo**: array of key labels, joined with `+`.
- **tone**: `default` (paper) · `brand` (saga) · `ink` · `native`.
- The backtick key is written as `` ` ``.
