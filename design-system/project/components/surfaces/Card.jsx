import React from "react";

/**
 * Card — warm brand surface. White on paper with EITHER a soft shadow OR a subtle
 * border (not both). No colored left-border accents, no glass on brand surfaces.
 */
export function Card({
  variant = "elevated",
  padding = "lg",
  interactive = false,
  as = "div",
  children,
  style = {},
  ...rest
}) {
  const pads = { none: 0, sm: "14px", md: "20px", lg: "24px", xl: "32px" };
  const base = {
    background: "var(--surface-card)",
    borderRadius: "var(--radius-md)",
    padding: pads[padding] ?? pads.lg,
    boxSizing: "border-box",
    transition: "box-shadow var(--dur-base) var(--ease-standard), transform var(--dur-base) var(--ease-standard)",
  };
  const variants = {
    elevated: { boxShadow: "var(--shadow-md)", border: "1px solid transparent" },
    outline: { border: "1px solid var(--border-subtle)" },
    sunken: { background: "var(--surface-sunken)", border: "1px solid var(--border-subtle)" },
    ink: { background: "var(--surface-inverse)", color: "var(--text-inverse)", border: "1px solid transparent" },
  };

  const [hover, setHover] = React.useState(false);
  const merged = { ...base, ...(variants[variant] || variants.elevated), ...style };
  if (interactive) {
    merged.cursor = "pointer";
    if (hover) {
      merged.boxShadow = "var(--shadow-lg)";
      merged.transform = "translateY(-2px)";
    }
  }

  const Tag = as;
  return (
    <Tag
      style={merged}
      onMouseEnter={interactive ? () => setHover(true) : undefined}
      onMouseLeave={interactive ? () => setHover(false) : undefined}
      {...rest}
    >
      {children}
    </Tag>
  );
}
