import React from "react";

/**
 * Dialog / Window — Windows-11 window chrome by default (WinTick's Settings, About,
 * onboarding all live in native windows). Set `native={false}` for a warm brand modal.
 */
export function Dialog({
  title = "",
  icon = null,
  native = true,
  width = 440,
  onClose = () => {},
  onMin = null,
  onMax = null,
  footer = null,
  children,
  style = {},
}) {
  const dark = false;
  const CaptionBtn = ({ kind, onClick }) => {
    const glyph = {
      min: <svg width="10" height="10" viewBox="0 0 10 10"><line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" strokeWidth="1" /></svg>,
      max: <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" fill="none" stroke="currentColor" strokeWidth="1" /></svg>,
      close: <svg width="10" height="10" viewBox="0 0 10 10"><line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" strokeWidth="1" /><line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" strokeWidth="1" /></svg>,
    }[kind];
    const [h, setH] = React.useState(false);
    return (
      <button
        onClick={onClick}
        onMouseEnter={() => setH(true)}
        onMouseLeave={() => setH(false)}
        aria-label={kind}
        style={{
          width: 44, height: 32, border: 0, cursor: "pointer",
          display: "inline-flex", alignItems: "center", justifyContent: "center",
          background: h ? (kind === "close" ? "#E81123" : "var(--n-bg-subtle, #e5e5e5)") : "transparent",
          color: h && kind === "close" ? "#fff" : "var(--n-text-primary, #1a1a1a)",
          transition: "background 90ms",
        }}
      >
        {glyph}
      </button>
    );
  };

  if (native) {
    return (
      <div
        className="native"
        style={{
          width, borderRadius: "var(--n-radius-window, 8px)",
          background: "var(--n-bg-app)", color: "var(--n-text-primary)",
          boxShadow: "var(--n-shadow-window)",
          border: "1px solid var(--n-stroke)",
          overflow: "hidden", fontFamily: "var(--font-native)",
          ...style,
        }}
      >
        <div style={{
          display: "flex", alignItems: "center",
          height: 32, paddingLeft: 12,
          background: "var(--n-bg-layer)",
          borderBottom: "1px solid var(--n-stroke)",
        }}>
          {icon && <span style={{ display: "inline-flex", marginRight: 8 }}>{icon}</span>}
          <span style={{ fontSize: 12.5, fontWeight: 400, flex: 1 }}>{title}</span>
          <div style={{ display: "flex" }}>
            <CaptionBtn kind="min" onClick={onMin || (() => {})} />
            <CaptionBtn kind="max" onClick={onMax || (() => {})} />
            <CaptionBtn kind="close" onClick={onClose} />
          </div>
        </div>
        <div style={{ padding: 20 }}>{children}</div>
        {footer && (
          <div style={{
            display: "flex", justifyContent: "flex-end", gap: 8,
            padding: "12px 20px", borderTop: "1px solid var(--n-stroke)",
            background: "var(--n-bg-layer)",
          }}>{footer}</div>
        )}
      </div>
    );
  }

  // Brand modal
  return (
    <div style={{
      width, borderRadius: "var(--radius-lg)",
      background: "var(--surface-card)", boxShadow: "var(--shadow-xl)",
      overflow: "hidden", fontFamily: "var(--font-sans)", ...style,
    }}>
      <div style={{ display: "flex", alignItems: "flex-start", gap: 12, padding: "22px 24px 0" }}>
        {icon && <span style={{ display: "inline-flex" }}>{icon}</span>}
        <div style={{ flex: 1 }}>
          <h3 style={{ margin: 0, fontSize: 20, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--text-strong)" }}>{title}</h3>
        </div>
        <button onClick={onClose} aria-label="close" style={{
          border: 0, background: "transparent", cursor: "pointer",
          color: "var(--text-muted)", fontSize: 18, lineHeight: 1, padding: 2,
        }}>✕</button>
      </div>
      <div style={{ padding: "12px 24px 4px", color: "var(--text-body)", fontSize: 15, lineHeight: 1.6 }}>{children}</div>
      {footer && (
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 10, padding: "16px 24px 24px" }}>{footer}</div>
      )}
    </div>
  );
}
