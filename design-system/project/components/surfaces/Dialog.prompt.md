# Dialog

Window / dialog chrome. Defaults to **native Windows-11** — caption bar with min / max / close buttons (close goes red on hover) — because WinTick's Settings, About and onboarding all live in real OS windows. Set `native={false}` for a warm brand modal.

```jsx
<Dialog title="WinTick — Settings" icon={<img src="…glyph-mono.svg" width="16" />}
        onClose={close}
        footer={<><Button variant="native">OK</Button></>}>
  …settings rows…
</Dialog>
```

- Wrap in your own dimmed overlay for a true modal.
- Footer typically holds `<Button variant="native">`.
