# Kertas Kerja Migrasi: Porting Settings UI & Onboarding ke Slint

## 1. Identifikasi & Sasaran
- **Proyek**: Wira Desk (`crates/settings`)
- **Tujuan**: Memigrasikan seluruh antarmuka Settings UI dan First-Run Interactive Onboarding Wizard dari `eframe`/`egui` ke **Slint** (`slint-ui`).
- **Prinsip Utama**:
  1. **UI/UX Sama Persis**: Dimensi, tata letak, custom caption bar frameless, sistem 5-pane, modal onboarding 3 langkah, token warna Fluent 2 Dark/Light, dan tipografi Segoe UI Variable Text.
  2. **Fitur Sama Persis**: Perekaman shortcut interaktif, deteksi konflik internal, penyelesaian 1-klik `[ Swap ⇄ ]`, denylist shortcut reserved OS, toggle auto-start, pengaturan stack width ratio, passthrough VM & exception lists, serta onboarding wizard.
  3. **Fungsi Sama Persis**: Serialisasi TOML atomik (`config.toml`), sinyal IPC daemon (`WM_APP_RELOAD_CONFIG`), single-instance mutex (`SETTINGS_SINGLE_INSTANCE_MUTEX`), dan deteksi tema registry (`AppsUseLightTheme`).
  4. **Perilaku Sama Persis**: State machine `Welcome` → `TrySwitching` → `Done`, simulasi interaktif dua dummy window, shortcut Escape cancellation, dan safeguard disable tombol Save saat konflik.
  5. **Unit Tests Terjaga 100%**: Mempertahankan seluruh 75 unit test pada `crates/settings` tanpa regresi.

---

## 2. Matriks Paritas Komponen

| Komponen / Fitur | Status eframe/egui | Status Target Slint | Status Paritas |
|---|---|---|---|
| Frameless Window Shell | 680×590 (Settings), 580×380 (Onboarding) | `Window { no-frame: true; }` | ⏳ In Progress |
| Custom Titlebar & Drag | 36px drag area + Minimize/Close | `Titlebar` Slint component | ⏳ In Progress |
| 5-Pane Vertical Sidebar | 175px sidebar + indicator pill | `Sidebar` Slint component | ⏳ In Progress |
| General Pane | Auto-start + Spatial + Honesty | `GeneralPane` Slint component | ⏳ In Progress |
| Shortcuts Pane | 6 rows + Listening + Amber + Swap | `ShortcutsPane` Slint component | ⏳ In Progress |
| Layout & Snapping Pane | Overlapping stack + Width slider | `LayoutPane` Slint component | ⏳ In Progress |
| VM & Exceptions Pane | Process bypass & class lists | `VmExceptionsPane` Slint component | ⏳ In Progress |
| About Pane | Branding, version, typeface, utility | `AboutPane` Slint component | ⏳ In Progress |
| Onboarding Wizard | 3-step progress bar, sandbox, footer | `OnboardingModal` Slint component | ⏳ In Progress |
| Save Bar & Footer Status | Dynamic status dot + Revert/Save | Sticky footer bar Slint component | ⏳ In Progress |
| Unit Test Suite | 75 unit tests passing | 75 unit tests passing | ⏳ In Progress |

---

## 3. Log Eksekusi & Catatan Perubahan
- [x] Rencana arsitektur dibuat dan disetujui.
- [x] Kertas kerja `.work/slint-migration-worklog.md` diinisialisasi.
- [ ] Penyesuaian `crates/settings/Cargo.toml` & `crates/settings/build.rs` untuk Slint.
- [ ] Pembuatan modul UI Slint (`crates/settings/ui/`).
- [ ] Implementasi backend Rust & Slint Bridge (`src/main.rs`, `src/app.rs`, `src/theme.rs`).
- [ ] Verifikasi suite pengujian dan audit kualitas (`cargo test`, `cargo clippy`, `cargo fmt`).
