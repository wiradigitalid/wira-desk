import React from "react";

/**
 * Button — Wira Digital brand button.
 * Warm saga fill, calm hover (darken one step), no scale on press.
 * Use `native` for the Windows-11 accent button inside WinTick app surfaces.
 */
export function Button({
  variant = "primary",
  size = "md",
  disabled = false,
  fullWidth = false,
  iconLeft = null,
  iconRight = null,
  as = "button",
  children,
  style = {},
  ...rest
}) {
  const sizes = {
    sm: { h: "var(--control-h-sm)", px: "12px", fs: "13px", gap: "7px" },
    md: { h: "var(--control-h-md)", px: "18px", fs: "15px", gap: "8px" },
    lg: { h: "var(--control-h-lg)", px: "24px", fs: "16px", gap: "10px" },
  };
  const s = sizes[size] || sizes.md;

  const base = {
    display: fullWidth ? "flex" : "inline-flex",
    width: fullWidth ? "100%" : "auto",
    alignItems: "center",
    justifyContent: "center",
    gap: s.gap,
    height: s.h,
    padding: `0 ${s.px}`,
    fontFamily: "var(--font-sans)",
    fontSize: s.fs,
    fontWeight: 600,
    letterSpacing: "-0.01em",
    lineHeight: 1,
    borderRadius: "var(--radius-sm)",
    border: "1px solid transparent",
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.5 : 1,
    transition: "background var(--dur-fast) var(--ease-standard), border-color var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard)",
    whiteSpace: "nowrap",
    userSelect: "none",
    textDecoration: "none",
    boxSizing: "border-box",
  };

  const variants = {
    primary: {
      background: "var(--brand)",
      color: "var(--brand-contrast)",
    },
    secondary: {
      background: "var(--surface-card)",
      color: "var(--text-strong)",
      borderColor: "var(--border-strong)",
    },
    ghost: {
      background: "transparent",
      color: "var(--text-body)",
    },
    danger: {
      background: "var(--signal-danger)",
      color: "#fff",
    },
    native: {
      background: "var(--n-accent, #0067C0)",
      color: "var(--n-accent-text, #fff)",
      fontFamily: "var(--font-native)",
      fontWeight: 400,
      borderRadius: "var(--n-radius-control, 4px)",
    },
  };

  const hoverBg = {
    primary: "var(--brand-hover)",
    secondary: "var(--surface-sunken)",
    ghost: "var(--surface-sunken)",
    danger: "#c20f1f",
    native: "var(--n-accent-hover, #0058A8)",
  };

  const [hover, setHover] = React.useState(false);
  const v = variants[variant] || variants.primary;
  const merged = { ...base, ...v };
  if (hover && !disabled) {
    if (variant === "primary" || variant === "danger" || variant === "native") merged.background = hoverBg[variant];
    else merged.background = hoverBg[variant];
    if (variant === "secondary") merged.borderColor = "var(--tanah-400)";
  }

  const Tag = as;
  return (
    <Tag
      style={{ ...merged, ...style }}
      disabled={as === "button" ? disabled : undefined}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      {...rest}
    >
      {iconLeft}
      {children}
      {iconRight}
    </Tag>
  );
}
