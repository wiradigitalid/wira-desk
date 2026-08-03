import React from "react";

/**
 * TrayIcon — WinTick's system-tray presence, communicating health via the 3-tier
 * protocol: normal · warning (small red dot) · error (large red cross).
 * The glyph is inlined so the component is self-contained.
 */
export function TrayIcon({
  state = "normal",
  size = 32,
  onDark = true,
  style = {},
}) {
  const fg = onDark ? "#ffffff" : "var(--tanah-800)";
  const alert = "var(--signal-danger, #E81123)";

  return (
    <span style={{ position: "relative", display: "inline-flex", width: size, height: size, ...style }}
      role="img" aria-label={`WinTick tray icon — ${state}`}>
      <svg width={size} height={size} viewBox="0 0 32 32" fill="none">
        <rect x="5" y="5" width="16" height="16" rx="3" fill="none" stroke={fg} strokeWidth="2.4" />
        <rect x="11" y="11" width="16" height="16" rx="3" fill={fg} />
      </svg>

      {state === "warning" && (
        <span style={{
          position: "absolute", right: -1, top: -1,
          width: size * 0.34, height: size * 0.34,
          borderRadius: "50%", background: alert,
          border: `1.5px solid ${onDark ? "#1a1a1a" : "#f3f3f3"}`,
          boxSizing: "border-box",
        }} />
      )}

      {state === "error" && (
        <svg width={size} height={size} viewBox="0 0 32 32" fill="none"
          style={{ position: "absolute", inset: 0 }}>
          <circle cx="16" cy="16" r="15" fill={alert} fillOpacity="0.14" />
          <line x1="7" y1="7" x2="25" y2="25" stroke={alert} strokeWidth="3.2" strokeLinecap="round" />
          <line x1="25" y1="7" x2="7" y2="25" stroke={alert} strokeWidth="3.2" strokeLinecap="round" />
        </svg>
      )}
    </span>
  );
}
