# TrayMenu

The Windows-11 flyout shown when right-clicking WinTick's tray icon. Fluent styling (rounded, layered, mica shadow). Defaults to the full WinTick context menu (FR-16); pass `items` to customize.

```jsx
<TrayMenu onSelect={(it) => open(it.label)} />

<TrayMenu items={[
  { kind: "item", label: "Settings…" },
  { kind: "toggle", label: "Auto-Start", checked: true },
  { kind: "separator" },
  { kind: "item", label: "Exit" },
]} />
```

Toggles show a Fluent check when `checked`.
