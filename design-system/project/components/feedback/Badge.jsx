import React from "react";

/**
 * Badge / Tag — small status or eyebrow label. Used for "GRATIS", "RINGAN",
 * version tags, and functional statuses.
 */
export function Badge({
  children,
  tone = "neutral",
  appearance = "soft",
  size = "md",
  caps = false,
  dot = false,
  style = {},
}) {
  const palette = {
    brand:   { solid: "var(--saga-500)", soft: "var(--saga-50)", softText: "var(--saga-700)", text: "#fff", line: "var(--saga-300)" },
    neutral: { solid: "var(--tanah-700)", soft: "var(--tanah-100)", softText: "var(--tanah-700)", text: "#fff", line: "var(--tanah-300)" },
    gold:    { solid: "var(--emas-500)", soft: "var(--accent-soft)", softText: "var(--emas-700)", text: "#fff", line: "var(--emas-400)" },
    danger:  { solid: "var(--signal-danger)", soft: "var(--signal-danger-soft)", softText: "#a30d1a", text: "#fff", line: "#f2a6ad" },
    warning: { solid: "var(--signal-warning)", soft: "var(--signal-warning-soft)", softText: "#8a6209", text: "#3a2c04", line: "#eecf7a" },
    success: { solid: "var(--signal-success)", soft: "var(--signal-success-soft)", softText: "#1c6b3d", text: "#fff", line: "#a6dcbb" },
  };
  const p = palette[tone] || palette.neutral;
  const sizes = {
    sm: { h: 18, fs: 10, px: 7 },
    md: { h: 22, fs: 11.5, px: 9 },
    lg: { h: 28, fs: 13, px: 12 },
  };
  const s = sizes[size] || sizes.md;

  const looks = {
    solid:   { background: p.solid, color: p.text, border: "1px solid transparent" },
    soft:    { background: p.soft, color: p.softText, border: "1px solid transparent" },
    outline: { background: "transparent", color: p.softText, border: `1px solid ${p.line}` },
  };

  return (
    <span style={{
      display: "inline-flex", alignItems: "center", gap: 5,
      height: s.h, padding: `0 ${s.px}px`,
      fontFamily: caps ? "var(--font-mono)" : "var(--font-sans)",
      fontSize: s.fs, fontWeight: caps ? 500 : 600,
      letterSpacing: caps ? "var(--tracking-caps)" : "0",
      textTransform: caps ? "uppercase" : "none",
      borderRadius: "var(--radius-pill)",
      lineHeight: 1, whiteSpace: "nowrap", boxSizing: "border-box",
      ...(looks[appearance] || looks.soft), ...style,
    }}>
      {dot && <span style={{
        width: 6, height: 6, borderRadius: "50%",
        background: appearance === "solid" ? "currentColor" : p.solid,
      }} />}
      {children}
    </span>
  );
}
