---
status: final
created: 2026-07-06
updated: 2026-07-06
sources:
  - ../../prds/prd-WinTick-2026-07-06/prd.md
  - ../mom-2026-07-06-ux-design.md
---

# WinTick Experience

## Foundation
WinTick adalah utilitas desktop Windows yang berjalan di latar belakang secara efisien. Secara fundamental, ia dipecah menjadi dua *binary* untuk mempertahankan performa:
- `wintick.exe`: Daemon latar belakang super ringan (<2MB RAM) tanpa antarmuka GUI sama sekali.
- `wintick-settings.exe`: Jendela UI interaktif untuk *onboarding* (First-Run) dan konfigurasi *Settings*, dipanggil secara *on-demand*.

## Information Architecture
Interaksi utama mutlak dikendalikan melalui pintasan *keyboard* global.
Interaksi sekunder terletak pada klik-kanan di ikon System Tray yang memunculkan *Context Menu*:
- Settings...
- View Logs
- Auto-Start *(toggle)*
- Check for Updates...
- About
- Exit

Struktur Jendela Settings `{components.settings_dialog}` (*wintick-settings.exe*):
- **Core Switcher**: Konfigurasi pintasan utama (`Win + \``) dan toggle *Alt-fallback*.
- **Window Snapping**: Pengaturan pintasan untuk Left/Right/Maximize/Monitor.
- **Stack Layout**: Toggle untuk mengaktifkan susunan bertumpuk (*overlapping stack*) 50% khusus monitor kecil.

## Voice and Tone
- **Neutral & Direct**: Sebagai utilitas sistem murni, teks pada UI Settings harus lugas dan informatif, tanpa bahasa kasual yang berlebihan.
- **Solutive Microcopy**: Pesan peringatan (seperti kegagalan hook) harus langsung menawarkan solusi (misal: "Harap jalankan sebagai Administrator").

## Accessibility Floor
- **Keyboard Navigation**: Seluruh elemen interaktif di dalam `{components.settings_dialog}` wajib dapat dinavigasi secara penuh menggunakan tombol *Tab*.
- **Screen Reader Support**: Elemen *toggle* dan kotak *input shortcut* harus mendeskripsikan status aktif/non-aktifnya dengan jelas kepada *screen reader* via *UI Automation* bawaan Windows.

## Component Patterns
- **Shortcut Capturer**: Saat pengguna mengklik *input box* shortcut, UI tidak menerima teks ketikan, melainkan masuk ke mode "Listening" untuk menangkap kombinasi *keyboard* fisik yang ditekan selanjutnya.
- **Decoupled Architecture**: Mengklik "Settings" pada tray akan mengeksekusi proses terpisah (`wintick-settings.exe`). Daemon utama tidak pernah di-blok oleh proses *rendering UI*, mengamankan respon *keyboard hook*.

## State Patterns
Indikator visual utama adalah ikon Tray `{components.tray_icon}` yang mengomunikasikan kesehatan sistem:
- **Normal**: Ikon bawaan. WinTick aktif merespons.
- **Warning / Logged**: Ikon + `{colors.tray_alert}` titik merah kecil. Terdapat *error non-fatal* yang dicatat secara diam-diam (*silent log*).
- **Error / Dead**: Ikon + `{colors.tray_alert}` tanda silang merah besar. Inisialisasi gagal atau *keyboard hook* terlepas secara sepihak oleh OS. **Wajib didampingi oleh *Toast Notification* Windows** (mengingat ikon tray disembunyikan ke dalam *overflow menu* secara *default*, pengguna butuh peringatan langsung saat aplikasi lumpuh).

## Interaction Primitives
- **Window Cycling**: Menekan `Win + \`` memindahkan fokus ke jendela berikutnya di aplikasi yang sama secara instan. Meniru MacOS secara absolut — nol animasi, nol transisi visual.

## Key Flows

### Flow 1: First-Run Onboarding Simulation
1. Pengguna menjalankan WinTick untuk pertama kalinya seumur hidup.
2. Alih-alih bersembunyi diam-diam di tray, modul `wintick-settings.exe` langsung meluncurkan simulasi interaktif.
3. Layar simulasi menuntun pengguna untuk mempraktikkan pintasan `Win + \``.
4. Sebuah *dummy window* di dalam UI secara visual berganti fokus untuk melatih memori otot pengguna (seperti panduan gestur MacOS).
5. Setelah berhasil, pengguna menutup simulasi dan WinTick sepenuhnya bersembunyi ke latar belakang, siap digunakan.

### Flow 2: Konfigurasi Jendela Settings
1. Pengguna mengklik kanan ikon Tray dan memilih "Settings...".
2. `wintick-settings.exe` terbuka. RAM sementara dialokasikan untuk memuat GUI.
3. Pengguna mengubah *shortcut snapping* layar penuh dan mencentang "Enable Overlapping Stack".
4. Pengaturan disimpan (ditulis ke `config.toml`). Daemon utama mendeteksi perubahan berkas dan me-reload konfigurasi tanpa *restart*.
5. Pengguna menutup jendela Settings. Proses UI terbunuh, dan RAM sistem kembali dibebaskan (<2MB).

### Flow 3: Rotasi Jendela Murni (Skenario Rian)
1. Protagonis (Rian) menekan `Cmd + \`` di *keyboard* Mac eksternalnya (terbaca sebagai `Win + \``).
2. WinTick menangkap pintasan secara *low-level* tanpa mengganggu sistem.
3. Fokus OS berpindah ke jendela Chrome kedua milik Rian seketika.
4. Tidak ada animasi, tidak ada antarmuka pemilih. Murni instan.

### Flow 4: Konfrontasi Aplikasi Hang (Skenario Maya)
1. Protagonis (Maya) menekan `Win + \`` untuk berpindah ke aplikasi berat.
2. Jendela target berikutnya ternyata dalam status *Not Responding*.
3. Menerapkan filosofi *UX Honesty*, WinTick tetap memindahkan fokus ke jendela yang *hang* tersebut alih-alih melompatinya.
4. Maya melihat *title bar* jendelanya *hang*, lalu mematikannya sendiri.

### Flow 5: Hambatan UIPI Bypass (Skenario Budi)
1. Protagonis (Budi) sedang aktif di jendela Task Manager (berjalan dengan *Administrator Privileges*).
2. Budi menekan `Win + \``.
3. Karena WinTick berjalan di latar belakang dengan *Highest Privileges* secara otomatis, ia tetap berhasil menangkap pintasan dan memindahkan fokus tanpa diblokir oleh keamanan UIPI.
