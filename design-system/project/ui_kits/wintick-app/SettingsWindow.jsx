/* WinTick — Settings window (wintick-settings.exe). Native Windows-11 chrome.
 * Feature-grouped vertical layout: Core Switcher, Window Snapping, Stack Layout, System.
 * Exposes window.SettingsWindow. */
(function () {
  const { Toggle, ShortcutInput, Keycap, Button, Badge, TrayIcon } = window.WiraDigitalDesignSystem_61a646;

  function Group({ title, desc, children }) {
    return (
      <section style={{ marginBottom: 22 }}>
        <div style={{ marginBottom: 10 }}>
          <h3 style={{ margin: 0, fontSize: 15, fontWeight: 600, color: "var(--n-text-primary)" }}>{title}</h3>
          {desc && <p style={{ margin: "2px 0 0", fontSize: 12.5, color: "var(--n-text-secondary)" }}>{desc}</p>}
        </div>
        <div style={{ background: "var(--n-bg-layer)", border: "1px solid var(--n-stroke)", borderRadius: 8, overflow: "hidden" }}>
          {children}
        </div>
      </section>
    );
  }

  function Row({ label, desc, control, last }) {
    return (
      <div style={{
        display: "flex", alignItems: "center", gap: 16, padding: "12px 16px",
        borderBottom: last ? "none" : "1px solid var(--n-stroke)",
      }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: 14, color: "var(--n-text-primary)" }}>{label}</div>
          {desc && <div style={{ fontSize: 12, color: "var(--n-text-secondary)", marginTop: 2 }}>{desc}</div>}
        </div>
        <div style={{ flex: "none" }}>{control}</div>
      </div>
    );
  }

  function SettingsWindow({ onClose = () => {}, dark = false }) {
    const [snap, setSnap] = React.useState(true);
    const [stack, setStack] = React.useState(false);
    const [fallback, setFallback] = React.useState(true);
    const [precise, setPrecise] = React.useState(true);
    const [autostart, setAutostart] = React.useState(true);
    const [scMain, setScMain] = React.useState(["Win", "`"]);
    const [scFull, setScFull] = React.useState(["Ctrl", "Win", "Enter"]);
    const [scLeft, setScLeft] = React.useState(["Ctrl", "Win", "\u2190"]);

    return (
      <div className={"native" + (dark ? " dark" : "")} style={{
        width: 560, height: 540, display: "flex", flexDirection: "column",
        borderRadius: 10, overflow: "hidden",
        background: "var(--n-bg-app)", color: "var(--n-text-primary)",
        boxShadow: "var(--n-shadow-window)", border: "1px solid var(--n-stroke)",
        fontFamily: "var(--font-native)",
      }}>
        {/* caption */}
        <div style={{ height: 34, display: "flex", alignItems: "center", gap: 8, padding: "0 0 0 12px", background: "var(--n-bg-layer)", borderBottom: "1px solid var(--n-stroke)", flex: "none" }}>
          <TrayIcon state="normal" size={16} onDark={dark} />
          <span style={{ fontSize: 12.5, flex: 1 }}>WinTick — Settings</span>
          <button onClick={onClose} aria-label="close" style={{ width: 46, height: 34, border: 0, background: "transparent", cursor: "pointer", color: "var(--n-text-primary)" }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#E81123"; e.currentTarget.style.color = "#fff"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--n-text-primary)"; }}>
            <svg width="10" height="10" viewBox="0 0 10 10" style={{ display: "block", margin: "0 auto" }}><line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" strokeWidth="1" /><line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" strokeWidth="1" /></svg>
          </button>
        </div>

        {/* header strip */}
        <div style={{ display: "flex", alignItems: "center", gap: 12, padding: "16px 20px", flex: "none" }}>
          <div style={{ width: 40, height: 40, borderRadius: 8, background: "var(--n-bg-layer)", border: "1px solid var(--n-stroke)", display: "flex", alignItems: "center", justifyContent: "center" }}>
            <TrayIcon state="normal" size={22} onDark={dark} />
          </div>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 15, fontWeight: 600 }}>WinTick 0.1.0</div>
            <div style={{ fontSize: 12, color: "var(--n-text-secondary)" }}>Berjalan</div>
          </div>
          <Badge tone="success" dot size="sm">Aktif</Badge>
        </div>

        {/* scroll body */}
        <div style={{ flex: 1, overflowY: "auto", padding: "4px 20px 20px" }}>
          <Group title="Core Switcher" desc="Putar fokus antar jendela dari aplikasi yang sama.">
            <Row label="Pintasan utama" desc="Ditangkap sebagai kombinasi fisik, bukan teks." control={<ShortcutInput native value={scMain} onChange={setScMain} />} />
            <Row label="Aktifkan fallback Alt + `" desc="Dipakai bila Win diblokir OS." control={<Toggle native checked={fallback} onChange={setFallback} />} />
            <Row label="Pencocokan presisi" desc="Modifier tambahan membatalkan rotasi." control={<Toggle native checked={precise} onChange={setPrecise} />} last />
          </Group>

          <Group title="Window Snapping" desc="Penempatan jendela presisi, sadar-DPI.">
            <Row label="Aktifkan snapping" control={<Toggle native checked={snap} onChange={setSnap} />} />
            <Row label="Layar penuh" control={<ShortcutInput native value={scFull} onChange={setScFull} />} />
            <Row label="Snap kiri 50%" control={<ShortcutInput native value={scLeft} onChange={setScLeft} />} last />
          </Group>

          <Group title="Stack Layout" desc="Untuk monitor kecil.">
            <Row label="Overlapping stack (50%)" desc="Tumpuk maks. 3 jendela dengan offset." control={<Toggle native checked={stack} onChange={setStack} />} last />
          </Group>

          <Group title="System">
            <Row label="Jalankan saat Windows menyala" desc="Registry Run key (HKCU)." control={<Toggle native checked={autostart} onChange={setAutostart} />} last />
          </Group>
        </div>

        {/* footer */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, padding: "12px 20px", borderTop: "1px solid var(--n-stroke)", background: "var(--n-bg-layer)", flex: "none" }}>
          <Button variant="native" onClick={onClose}>Selesai</Button>
        </div>
      </div>
    );
  }

  Object.assign(window, { SettingsWindow });
})();
