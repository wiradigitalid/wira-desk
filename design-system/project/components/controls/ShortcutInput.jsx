import React from "react";
import { Keycap } from "./Keycap.jsx";

/**
 * ShortcutInput — the "Listening mode" capturer from WinTick Settings.
 * Click to arm; the next physical key combo is captured (it never accepts typed text).
 * Controlled via `value` (array of key labels) + `onChange`.
 */
export function ShortcutInput({
  value = ["Win", "`"],
  onChange = () => {},
  native = false,
  disabled = false,
  style = {},
}) {
  const [listening, setListening] = React.useState(false);
  const ref = React.useRef(null);

  const label = (e) => {
    const map = { Control: "Ctrl", Meta: "Win", " ": "Space", "`": "`" };
    if (e.key === "Escape") return null;
    const mods = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");
    if (e.metaKey) mods.push("Win");
    let main = e.key;
    if (["Control", "Alt", "Shift", "Meta"].includes(main)) main = null;
    else main = map[main] || (main.length === 1 ? main.toUpperCase() : main);
    return main ? [...mods, main] : mods;
  };

  const onKeyDown = (e) => {
    if (!listening) return;
    e.preventDefault();
    if (e.key === "Escape") { setListening(false); return; }
    const combo = label(e);
    if (combo && combo.length) {
      onChange(combo);
      setListening(false);
    }
  };

  const borderCol = listening
    ? (native ? "var(--n-accent, #0067C0)" : "var(--brand)")
    : (native ? "var(--n-stroke-control, #dadada)" : "var(--border-strong)");

  return (
    <div
      ref={ref}
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-label="Shortcut capturer"
      onClick={() => !disabled && setListening(true)}
      onBlur={() => setListening(false)}
      onKeyDown={onKeyDown}
      style={{
        display: "inline-flex", alignItems: "center", gap: 8,
        minWidth: 172, height: native ? 32 : 40,
        padding: "0 12px",
        background: native ? "var(--n-bg-solid, #fff)" : "var(--surface-card)",
        border: `${listening ? 2 : 1}px solid ${borderCol}`,
        borderRadius: native ? "var(--n-radius-control, 4px)" : "var(--radius-sm)",
        boxShadow: listening && !native ? "var(--ring-brand)" : "none",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        fontFamily: native ? "var(--font-native)" : "var(--font-sans)",
        boxSizing: "border-box",
        transition: "border-color var(--dur-fast), box-shadow var(--dur-fast)",
      }}
    >
      {listening ? (
        <span style={{
          fontSize: 13, color: native ? "var(--n-accent, #0067C0)" : "var(--brand)",
          fontWeight: 500, display: "inline-flex", alignItems: "center", gap: 6,
        }}>
          <span style={{
            width: 7, height: 7, borderRadius: "50%",
            background: native ? "var(--n-accent, #0067C0)" : "var(--brand)",
          }} />
          Listening… press keys
        </span>
      ) : (
        <span style={{ display: "inline-flex", alignItems: "center", gap: 5 }}>
          <Keycap combo={value} size="sm" tone={native ? "native" : "default"} />
        </span>
      )}
    </div>
  );
}
