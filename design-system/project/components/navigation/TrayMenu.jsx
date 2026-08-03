import React from "react";

/**
 * TrayMenu — the Windows-11 flyout that appears on right-clicking WinTick's tray icon.
 * Fluent styling: rounded, acrylic-ish layer, mica shadow. Supports items, toggles,
 * and separators. Defaults to WinTick's full context menu (FR-16).
 */
const DEFAULT_ITEMS = [
  { kind: "item", label: "Settings\u2026" },
  { kind: "item", label: "View Logs" },
  { kind: "toggle", label: "Auto-Start", checked: true },
  { kind: "separator" },
  { kind: "item", label: "Check for Updates\u2026" },
  { kind: "item", label: "About" },
  { kind: "separator" },
  { kind: "item", label: "Exit" },
];

export function TrayMenu({ items = DEFAULT_ITEMS, width = 220, onSelect = () => {}, style = {} }) {
  const [hover, setHover] = React.useState(-1);

  const Check = () => (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
      <path d="M2.5 7.5 L6 11 L11.5 3.5" stroke="var(--n-accent, #0067C0)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );

  return (
    <div className="native" style={{
      width, padding: 4,
      background: "var(--n-bg-layer)",
      border: "1px solid var(--n-stroke)",
      borderRadius: 8,
      boxShadow: "var(--n-shadow-flyout)",
      fontFamily: "var(--font-native)",
      ...style,
    }}>
      {items.map((it, i) => {
        if (it.kind === "separator") {
          return <div key={i} style={{ height: 1, background: "var(--n-stroke)", margin: "4px 8px" }} />;
        }
        const h = hover === i;
        return (
          <button
            key={i}
            onMouseEnter={() => setHover(i)}
            onMouseLeave={() => setHover(-1)}
            onClick={() => onSelect(it, i)}
            style={{
              display: "flex", alignItems: "center", gap: 10, width: "100%",
              height: 34, padding: "0 10px", border: 0, textAlign: "left",
              background: h ? "var(--n-bg-subtle)" : "transparent",
              color: "var(--n-text-primary)",
              borderRadius: 5, cursor: "pointer",
              fontSize: 13.5, fontWeight: 400, fontFamily: "var(--font-native)",
            }}
          >
            <span style={{ width: 16, display: "inline-flex", justifyContent: "center", flex: "none" }}>
              {it.kind === "toggle" && it.checked ? <Check /> : null}
            </span>
            <span style={{ flex: 1 }}>{it.label}</span>
          </button>
        );
      })}
    </div>
  );
}
