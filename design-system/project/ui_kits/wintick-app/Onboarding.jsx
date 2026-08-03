/* WinTick — First-Run Onboarding Simulation (FR-17)
 * Native Windows-11 window that teaches the Win + ` muscle memory with a live dummy
 * window that changes focus as the user practises. Includes "Skip Tutorial".
 * Exposes window.Onboarding. */
(function () {
  const { Button, Keycap, TrayIcon } = window.WiraDigitalDesignSystem_61a646;

  const DUMMIES = [
    { id: 0, title: "Chrome — Riset visual", host: "figma.com" },
    { id: 1, title: "Chrome — Dokumentasi", host: "developer.mozilla.org" },
    { id: 2, title: "Chrome — Dashboard", host: "vercel.com" },
  ];

  function DummyWindow({ w, active, offset }) {
    return (
      <div style={{
        position: "absolute", left: 40 + offset * 34, top: 30 + offset * 26,
        width: 300, height: 190, borderRadius: 8, overflow: "hidden",
        background: "var(--n-bg-solid)",
        border: "1px solid var(--n-stroke)",
        boxShadow: active ? "0 18px 40px rgba(0,0,0,.28)" : "0 6px 14px rgba(0,0,0,.14)",
        transform: active ? "translateY(-6px) scale(1.0)" : "scale(0.985)",
        filter: active ? "none" : "saturate(.85) brightness(.98)",
        transition: "all 200ms cubic-bezier(0.2,0,0,1)",
        zIndex: active ? 5 : 1,
      }}>
        <div style={{
          height: 30, display: "flex", alignItems: "center", gap: 8, padding: "0 10px",
          background: active ? "var(--n-bg-layer)" : "var(--n-bg-app)",
          borderBottom: "1px solid var(--n-stroke)",
        }}>
          <span style={{ width: 9, height: 9, borderRadius: "50%", background: active ? "var(--n-accent)" : "var(--n-text-disabled)" }} />
          <span style={{ fontSize: 11.5, color: "var(--n-text-secondary)", fontWeight: active ? 600 : 400 }}>{w.title}</span>
        </div>
        <div style={{ padding: 12 }}>
          <div style={{ fontSize: 10, color: "var(--n-text-disabled)", fontFamily: "var(--font-mono)" }}>{w.host}</div>
          <div style={{ marginTop: 10, height: 8, width: "80%", borderRadius: 4, background: "var(--n-bg-subtle)" }} />
          <div style={{ marginTop: 8, height: 8, width: "55%", borderRadius: 4, background: "var(--n-bg-subtle)" }} />
          <div style={{ marginTop: 8, height: 8, width: "68%", borderRadius: 4, background: "var(--n-bg-subtle)" }} />
        </div>
      </div>
    );
  }

  function Onboarding({ onFinish = () => {}, onSkip = () => {} }) {
    const [active, setActive] = React.useState(0);
    const [count, setCount] = React.useState(0);
    const done = count >= 3;

    const cycle = React.useCallback(() => {
      setActive((a) => (a + 1) % DUMMIES.length);
      setCount((c) => c + 1);
    }, []);

    React.useEffect(() => {
      const onKey = (e) => {
        if (e.key === "`" && (e.metaKey || e.altKey || e.ctrlKey)) { e.preventDefault(); cycle(); }
      };
      window.addEventListener("keydown", onKey);
      return () => window.removeEventListener("keydown", onKey);
    }, [cycle]);

    return (
      <div className="native" style={{
        width: 620, borderRadius: 10, overflow: "hidden",
        background: "var(--n-bg-app)", boxShadow: "var(--n-shadow-window)",
        border: "1px solid var(--n-stroke)", fontFamily: "var(--font-native)",
      }}>
        {/* caption */}
        <div style={{ height: 32, display: "flex", alignItems: "center", gap: 8, padding: "0 12px", background: "var(--n-bg-layer)", borderBottom: "1px solid var(--n-stroke)" }}>
          <TrayIcon state="normal" size={16} onDark={false} />
          <span style={{ fontSize: 12.5, flex: 1 }}>Selamat datang di WinTick</span>
        </div>

        <div style={{ padding: "22px 26px 8px" }}>
          <div style={{ fontFamily: "var(--font-mono)", fontSize: 10, letterSpacing: ".08em", textTransform: "uppercase", color: "var(--n-accent)" }}>Langkah 1 dari 1 · Coba sekarang</div>
          <h2 style={{ margin: "8px 0 6px", fontSize: 24, fontWeight: 700, color: "var(--n-text-primary)", letterSpacing: "-0.01em" }}>
            Berpindah antar jendela aplikasi yang sama
          </h2>
          <p style={{ margin: 0, fontSize: 14, lineHeight: 1.55, color: "var(--n-text-secondary)", maxWidth: 460 }}>
            Tekan pintasan di bawah untuk memutar fokus antara tiga jendela Chrome ini. Persis seperti <span style={{fontFamily:"var(--font-mono)"}}>Cmd&nbsp;+&nbsp;`</span> di Mac — instan, tanpa animasi pengalih.
          </p>
        </div>

        {/* stage */}
        <div style={{ position: "relative", height: 270, margin: "10px 26px 0", borderRadius: 8, background: "repeating-linear-gradient(135deg, #e9e4dc 0 2px, #f0ece5 2px 22px)", border: "1px solid var(--n-stroke)", overflow: "hidden" }}>
          {DUMMIES.map((w, i) => (
            <DummyWindow key={w.id} w={w} active={active === i} offset={i} />
          ))}
        </div>

        {/* footer */}
        <div style={{ display: "flex", alignItems: "center", gap: 16, padding: "18px 26px 22px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <Keycap combo={["Win", "`"]} size="lg" tone="native" />
            <Button variant="native" onClick={cycle}>{done ? "Sekali lagi" : "Simulasikan tekan"}</Button>
          </div>
          <div style={{ flex: 1, fontSize: 13, color: done ? "var(--n-accent)" : "var(--n-text-secondary)", fontWeight: done ? 600 : 400 }}>
            {done ? "Bagus! Kamu sudah paham. WinTick siap bekerja di latar belakang." : `Fokus berpindah ${count}/3 — lanjut sampai terasa alami.`}
          </div>
          {done
            ? <Button variant="native" onClick={onFinish}>Selesai</Button>
            : <button onClick={onSkip} style={{ border: 0, background: "transparent", color: "var(--n-text-secondary)", fontSize: 13, cursor: "pointer", fontFamily: "var(--font-native)" }}>Skip Tutorial</button>}
        </div>
      </div>
    );
  }

  Object.assign(window, { Onboarding });
})();
