# Addendum — WinTick PRD

Dokumen ini menyimpan kedalaman teknis, rationale keputusan arsitektur, dan konteks tambahan yang diperoleh selama sesi *brainstorming* dan *Advanced Elicitation*, namun secara struktural lebih cocok sebagai rujukan arsitektur/solusi daripada badan utama PRD.

---

## A1. Rationale Penolakan `RegisterHotKey`
**Sumber:** MoM 6 Jul — Ronde 10 (Socratic Questioning)

API `RegisterHotKey` awalnya dipertimbangkan sebagai penyederhanaan arsitektur (menghilangkan kebutuhan Utas-Ganda). Namun API ini beroperasi dengan model *first-come-first-served* — jika aplikasi pihak ketiga telah mendaftarkan *hotkey* yang sama, WinTick akan **kalah** dan tidak bisa mendaftarkan pintasannya. Ini tidak dapat diterima untuk utilitas yang menjanjikan "dominasi absolut atas input OS".

**Keputusan:** `RegisterHotKey` digugurkan secara permanen. `WH_KEYBOARD_LL` adalah satu-satunya pendekatan yang dapat meng-*override* pintasan aplikasi lain secara diam-diam.

---

## A2. Rationale Mempertahankan `std` (Menolak `#![no_std]`)
**Sumber:** MoM 6 Jul — Ronde 17 (Reverse Engineering)

Menghapus *Standard Library* Rust memang akan menghasilkan biner ~50-100KB lebih kecil, namun menghancurkan jaminan konkurensi (*thread safety*) bawaan — memaksa penggunaan C-FFI mentah untuk sinkronisasi utas. Akar masalah pembengkakan biner Windows sebenarnya bukan `std`, melainkan abstraksi COM pada *crate* `windows` (bukan `windows-sys`).

**Keputusan:** Mempertahankan `std` + `windows-sys` + profil rilis agresif menghasilkan estimasi biner 250KB–400KB, lebih dari cukup memenuhi target <500KB.

---

## A3. Rationale Justifikasi vs AutoHotkey / Skrip Ringan
**Sumber:** MoM 6 Jul — Ronde 18 (Shark Tank Pitch)

Utilitas *background* pihak ketiga (AutoHotkey, C#, Python) menghadapi kerentanan fundamental:
- **Cascading Hang:** Rawan lumpuh saat berinteraksi dengan aplikasi *Not Responding* karena menggunakan API sinkron.
- **Micro-Stutters:** *Garbage Collector* pada runtime managed (C#, Python) memicu jeda periodik yang merusak persepsi "instan".
- **Hook Dropout:** Tanpa isolasi utas berprioritaritras tinggi, OS Windows akan mencabut *global hook* jika respons melebihi `LowLevelHooksTimeout` (300ms).

**Keputusan:** Seluruh kerumitan arsitektur WinTick (Utas-Ganda, Ring Buffer, sterilisasi API) dikonfirmasi proporsional dan bukan *over-engineering*.

---

## A4. Evolusi Fitur Snapping — Perjalanan Keputusan
**Sumber:** MoM 4 Jul R6, R7, R8 → MoM 6 Jul R3 → Rekonsiliasi PRD

- **R6 (4 Jul):** Fitur *BetterSnapTool snapping* dan *overlapping stack* pertama kali diusulkan.
- **R3 (6 Jul — Subtraction):** Fitur pemindahan antar-monitor dikeluarkan dari *scope* (delegasi ke `Win+Shift+Arrow` bawaan Windows). Fitur *overlapping stack* sempat di-*cut* karena dianggap menambah overhead kalkulasi DPI.
- **Rekonsiliasi PRD:** Pengguna mengkonfirmasi bahwa fitur *snapping* dan *overlapping stack* harus tetap dalam *scope* sebagai P2. Keputusan *subtraction* dari Ronde 3 di-*override* secara parsial.

---

## A5. Skema Konfigurasi TOML (Referensi)
**Sumber:** SPEC Companion — `conventions.md`

```toml
[switcher]
shortcut = "win+backtick"
fallback_shortcut = "alt+backtick"

[snapping]
snap_half_left = "ctrl+win+left"
snap_half_right = "ctrl+win+right"
snap_maximize = "ctrl+win+enter"
move_next_monitor = "ctrl+win+shift+enter"

[layout]
enable_overlapping_stack = true
stack_width_percent = 50
```

---

## A6. Opsi Arsitektur yang Ditolak

| Opsi | Alasan Penolakan |
| :--- | :--- |
| `RegisterHotKey` | Kalah prioritas pada skenario *first-come-first-served* (lihat A1) |
| `#![no_std]` Rust | Merusak *thread safety* bawaan (lihat A2) |
| *Crate* `windows` (COM wrapper) | Membengkakkan biner karena abstraksi COM, diganti `windows-sys` C-FFI murni |
| Framework GUI untuk System Tray | Membengkakkan RAM, diganti murni Win32 API |
| *Internal Z-Order caching* | Menyebabkan desinkronisasi dengan interaksi mouse (lihat NFR Stateless Z-Order) |
| *Skip* jendela *hang* | Melanggar filosofi UX Honesty (lihat FR-4) |
