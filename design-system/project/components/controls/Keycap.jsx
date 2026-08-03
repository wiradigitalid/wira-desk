import React from "react";

/**
 * Keycap — a physical keyboard key, set in mono. Core to WinTick's shortcut language.
 * Render a single key via children, or a full combo via `combo={["Win", "`"]}`.
 */
export function Keycap({
  children,
  combo = null,
  size = "md",
  tone = "default",
  style = {},
}) {
  const sizes = {
    sm: { h: 22, minW: 22, fs: 11, px: 6, r: 5 },
    md: { h: 28, minW: 28, fs: 13, px: 8, r: 6 },
    lg: { h: 38, minW: 38, fs: 16, px: 11, r: 8 },
  };
  const s = sizes[size] || sizes.md;

  const tones = {
    default: { bg: "var(--white)", fg: "var(--text-strong)", edge: "var(--tanah-300)" },
    brand:   { bg: "var(--saga-500)", fg: "#fff", edge: "var(--saga-700)" },
    ink:     { bg: "var(--tanah-800)", fg: "#fff", edge: "var(--black)" },
    native:  { bg: "var(--n-bg-solid, #fff)", fg: "var(--n-text-primary, #1a1a1a)", edge: "var(--n-stroke-control, #cfcfcf)" },
  };
  const t = tones[tone] || tones.default;

  const cap = (label, key) => (
    <kbd
      key={key}
      style={{
        display: "inline-flex", alignItems: "center", justifyContent: "center",
        height: s.h, minWidth: s.minW, padding: `0 ${s.px}px`,
        fontFamily: "var(--font-mono)", fontSize: s.fs, fontWeight: 600,
        color: t.fg, background: t.bg,
        border: `1px solid ${t.edge}`,
        borderBottomWidth: 2.5,
        borderRadius: s.r,
        lineHeight: 1,
        boxSizing: "border-box",
        whiteSpace: "nowrap",
      }}
    >
      {label}
    </kbd>
  );

  if (combo) {
    return (
      <span style={{ display: "inline-flex", alignItems: "center", gap: s.px - 2, ...style }}>
        {combo.map((k, i) => (
          <React.Fragment key={i}>
            {cap(k, i)}
            {i < combo.length - 1 && (
              <span style={{ fontFamily: "var(--font-mono)", fontSize: s.fs, color: "var(--text-faint)", fontWeight: 500 }}>+</span>
            )}
          </React.Fragment>
        ))}
      </span>
    );
  }
  return <span style={style}>{cap(children)}</span>;
}
