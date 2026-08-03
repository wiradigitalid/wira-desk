# Minutes of Meeting (MoM) — Diskusi Proyek WinTick

**Metadata**
- **Tanggal**: 2026-07-06
- **Waktu Mulai**: 14:38 WIB
- **Waktu Selesai**: 15:33 WIB
- **Topik**: UX Design — Desain Interaksi & Visual System Tray Utility
- **Peserta**: kodesh87, AI Agent (Antigravity)

## 1. Ringkasan Eksekutif (Executive Summary)
Sesi ini menetapkan fondasi pengalaman pengguna (UX) untuk WinTick. Keputusan utama adalah mempertahankan interaksi yang murni tak kasat mata (*invisible* dan instan tanpa animasi) seperti halnya MacOS, dengan antarmuka yang sangat dibatasi pada System Tray. Untuk mengakomodasi kebutuhan GUI Konfigurasi (Settings) yang ramah pengguna dan *Onboarding* interaktif tanpa melanggar batasan ketat RAM (<2MB), disepakati sebuah keputusan arsitektur krusial: memisahkan antarmuka Settings ke dalam program *executable* terpisah (`wintick-settings.exe`).

## 2. Alur Percakapan (Flow of Speech)

**Ronde 0**
- *User Prompt*: buatkan sesi MoM, untuk pembahasan UX
- *Agent Response*: Menginisialisasi sesi perekaman MoM dan melemparkan 5 pertanyaan panduan terkait: (1) System Tray Icon, (2) Tray Context Menu, (3) Konfigurasi Settings, (4) First-Run Experience, dan (5) Feedback Visual.

**Ronde 1**
- *User Prompt*: Menegaskan tidak boleh ada animasi pada perpindahan (*Feedback Visual* harus persis MacOS). Ikon tray statis, tetapi meminta ada titik merah jika ada log, dan silang merah jika error. *Context Menu* berisi: Settings, About, Check for Updates, Exit. Menginginkan *Settings GUI* yang standar dan fungsional. Untuk *First-Run*, tidak perlu muncul saat Windows restart, tetapi saat dijalankan pertama kali perlu ada simulasi interaktif untuk latihan (seperti gestur MacOS).
- *Agent Response*: Mengonfirmasi kelayakan visual state ikon (titik/silang merah) murni menggunakan manipulasi aset `HICON` Win32 tanpa penalti RAM. Mengidentifikasi konflik arsitektur: GUI Settings yang modern akan merusak batas RAM daemon (<2MB). Oleh karena itu, diusulkan *Separation of Concerns* dengan memecah antarmuka Settings dan *Onboarding Simulation* ke dalam modul program terpisah (`wintick-settings.exe`) yang hanya hidup saat dipanggil.

**Ronde 2**
- *User Prompt*: Menyetujui arsitektur pemisahan program untuk GUI Settings. Mengoreksi state visual ikon: ada log = titik merah, tidak aktif/error = silang merah. Menetapkan bahwa GUI Settings harus modular: setiap fitur dapat dikonfigurasi secara mandiri (mengubah shortcut utama, mengubah/mematikan fitur tambahan).
- *Agent Response*: Mencatat koreksi visual state ikon. Mengonfirmasi struktur jendela Settings akan modular (grup per-fitur dengan kotak *input hotkey* dan *toggle* aktif/non-aktif).

## 3. Kesimpulan & Keputusan Utama (Conclusions & Key Decisions)
1. **Decoupled Settings UI (Architectural Decision)**: WinTick akan memiliki dua *binary* terpisah. `wintick.exe` (daemon latar belakang, <2MB RAM, murni Win32), dan `wintick-settings.exe` (pustaka GUI untuk Settings dan Onboarding).
2. **Tray Icon State Machine**: 
   - Normal = Ikon statis polos.
   - Peringatan/Log = Ikon dengan _overlay_ titik merah kecil.
   - Error/Mati = Ikon dengan _overlay_ tanda silang merah besar.
3. **Context Menu Layout**: Terdiri dari `Settings...`, `Check for Updates...`, `About`, dan `Exit`.
4. **First-Run Interactive Simulation**: Saat aplikasi dijalankan untuk pertama kalinya seumur hidup (bukan *startup boot* biasa), aplikasi meluncurkan jendela interaktif untuk melatih memori otot pengguna. Dilengkapi dengan tombol **Skip Tutorial** yang eksplisit bagi *power user*.
5. **Invisible Rotation**: Perpindahan jendela mutlak instan dan tak kasat mata. Nol animasi, nol *overlay*.
6. **Modular Settings GUI**: Jendela Settings akan menyediakan kontrol *granular* (sakelar On/Off dan kotak pengaturan *hotkey*) untuk setiap fitur (Window Cycling, Snapping, Overlapping Stack).

## 4. Validasi Lanjutan (Advanced Elicitation)
Melalui *Party Mode* (Winston, Sally, John, Amelia) dengan dua metode:

**Metode 1: User Persona Focus Group**
- Disepakati penambahan tombol **Skip Tutorial** pada fitur *First-Run* untuk menghindari *friction* pada *power user*.
- Ide *toggle* "Skip Hung Windows" ditolak; mempertahankan absolutisme *UX Honesty*.
- Kekhawatiran UAC Prompt dipatahkan; anak proses yang dipanggil oleh induk *elevated* akan otomatis berjalan secara *elevated* pula secara diam-diam.

**Metode 2: Pre-Mortem Analysis**
- Ditemukan kelemahan fatal (Silent Failure): Ikon tray dengan status *Error* (silang merah) akan gagal terlihat pengguna karena OS Windows 11 menyembunyikan ikon tray secara *default*.
- **Solusi**: Saat WinTick mati atau *keyboard hook* terputus, ia wajib menembakkan *Toast Notification* Windows secara eksplisit ke layar pengguna.

## 5. Daftar Tindakan Lanjut (Action Items)
| Tindakan / Tugas | Penanggung Jawab | Status |
| :--- | :--- | :--- |
| Mengekstraksi keputusan MoM ini ke dalam artefak `DESIGN.md` | AI Agent | `[x] Selesai` |
| Mengekstraksi keputusan MoM ini ke dalam artefak `EXPERIENCE.md` | AI Agent | `[x] Selesai` |
| Mencatat keputusan arsitektur pemisahan *binary* ke memlog PRD/Architecture | AI Agent | `[x] Selesai` |
