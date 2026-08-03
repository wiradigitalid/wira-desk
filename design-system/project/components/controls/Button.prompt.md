# Button

Brand call-to-action. Warm saga fill with a calm hover (darkens one step, no scale). Use `variant="native"` for the Windows-11 accent button inside WinTick app recreations.

```jsx
<Button onClick={install}>Unduh gratis</Button>
<Button variant="secondary">Pelajari</Button>
<Button variant="native" size="sm">OK</Button>
```

- **variant**: `primary` (default, saga) · `secondary` (outline) · `ghost` (text) · `danger` (#E81123) · `native` (Win11 accent).
- **size**: `sm` 32 · `md` 40 · `lg` 48.
- Pass `iconLeft`/`iconRight` (e.g. a Lucide `<i data-lucide>` or SVG). Sentence case labels, never Title Case.
