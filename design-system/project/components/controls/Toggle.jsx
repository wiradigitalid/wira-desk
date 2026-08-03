import React from "react";

/**
 * Toggle — on/off switch. WinTick's Settings window is built from rows of these.
 * `native` renders the Windows-11 Fluent switch; default renders the warm brand switch.
 */
export function Toggle({
  checked = false,
  onChange = () => {},
  disabled = false,
  native = false,
  label = null,
  description = null,
  id,
  style = {},
  ...rest
}) {
  const W = 40, H = 22, KNOB = 16, PAD = 3;
  const on = checked;

  const trackColor = native
    ? (on ? "var(--n-accent, #0067C0)" : "var(--n-bg-solid, #fff)")
    : (on ? "var(--brand)" : "var(--tanah-300)");
  const trackBorder = native
    ? (on ? "var(--n-accent, #0067C0)" : "var(--n-stroke-control, #8a8a8a)")
    : "transparent";
  const knobColor = native
    ? (on ? "var(--n-accent-text, #fff)" : "var(--n-text-secondary, #5d5d5d)")
    : "#fff";

  const sw = (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={typeof label === "string" ? label : undefined}
      disabled={disabled}
      onClick={() => !disabled && onChange(!on)}
      style={{
        position: "relative",
        width: W, height: H,
        flex: "none",
        borderRadius: 999,
        border: `${native ? 1.5 : 0}px solid ${trackBorder}`,
        background: trackColor,
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        padding: 0,
        transition: "background var(--dur-base) var(--ease-standard)",
        boxSizing: "border-box",
      }}
      {...rest}
    >
      <span
        style={{
          position: "absolute",
          top: "50%",
          left: on ? W - KNOB - PAD - (native ? 1.5 : 0) : PAD,
          width: KNOB, height: KNOB,
          marginTop: -KNOB / 2,
          borderRadius: "50%",
          background: knobColor,
          boxShadow: native ? "none" : "0 1px 2px rgba(26,22,17,0.3)",
          transition: "left var(--dur-base) var(--ease-standard), background var(--dur-base)",
        }}
      />
    </button>
  );

  if (!label && !description) return React.cloneElement(sw, { style: { ...sw.props.style, ...style } });

  return (
    <label
      htmlFor={id}
      style={{
        display: "flex", alignItems: "center", gap: 12,
        justifyContent: "space-between",
        fontFamily: native ? "var(--font-native)" : "var(--font-sans)",
        cursor: disabled ? "not-allowed" : "pointer",
        ...style,
      }}
    >
      <span style={{ minWidth: 0 }}>
        <span style={{
          display: "block",
          fontSize: native ? 14 : 15, fontWeight: native ? 400 : 500,
          color: native ? "var(--n-text-primary)" : "var(--text-strong)",
        }}>{label}</span>
        {description && (
          <span style={{
            display: "block", marginTop: 2,
            fontSize: native ? 12 : 13,
            color: native ? "var(--n-text-secondary)" : "var(--text-muted)",
          }}>{description}</span>
        )}
      </span>
      {sw}
    </label>
  );
}
