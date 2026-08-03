# Minutes of Meeting (MoM) — Diskusi Proyek WinTick

## Informasi Rapat
- **Tanggal**: 2026-07-09
- **Waktu**: 06:11:12 - 06:28:03
- **Proyek**: WinTick
- **Topik**: Elevated Administrator Access & Onboarding Flow
- **Peserta**:
  - `kodesh87` (User)
  - **Winston** (System Architect)
  - **John** (Product Manager)
  - **Sally** (UX Designer)
  - **Amelia** (Senior Software Engineer)
  - **Vex** (Security Engineer)
  - **Grumbal** (The Adversary / Red Team)
  - **Boundary** (Edge-Case Hunter)
  - **Dana** (The Pragmatist)

---

## 1. Ringkasan Eksekutif (Executive Summary)
Diskusi ini membahas rancangan arsitektur hak akses pada aplikasi WinTick. Karena WinTick adalah utilitas siklus jendela (*window cycling*) yang harus mampu beroperasi pada semua jendela aktif termasuk jendela elevated (seperti Task Manager dan Command Prompt Administrator), aplikasi wajib mendapatkan hak akses *Administrator* untuk mem-bypass *User Interface Privilege Isolation* (UIPI) Windows. 

Untuk menghindari ketidaknyamanan pop-up UAC (*User Account Control*) setiap kali komputer dinyalakan kembali (*restart*), disepakati penggunaan **Windows Task Scheduler** dengan opsi *Highest Privileges* untuk meluncurkan daemon secara sunyi (*silent*). 

Dalam diskusi, tim melakukan analisis mendalam melalui simulasi *Red Team vs Blue Team* dan *Pre-mortem Analysis* untuk mengidentifikasi celah keamanan (seperti DLL Hijacking dan eskalasi hak akses) serta potensi kegagalan inisialisasi (*Logon Race Condition* dan bentrokan instansi ganda). Semua poin mitigasi dan alur UX instalasi/onboarding didefinisikan secara konkret untuk diimplementasikan pada pembaruan spesifikasi teknis (Story 1.2).

---

## 2. Alur Percakapan (Flow of Speech)

### Ronde 0: Inisiasi Sesi
- **User Prompt**: Mengaktifkan sesi diskusi MoM dengan topik *elevated administrator access*.
- **Agent Response**: Mengonfirmasi inisiasi sesi MoM dan bersiap merekam kontribusi secara kronologis.

### Ronde 1: Analisis Kebutuhan Elevasi Hak Akses & Alternatif
- **User Prompt**: Menanyakan apakah hak akses elevated Administrator diperlukan, kegunaannya, apakah ada cara lain, dan bagaimana cara mem-bypass prompt UAC saat restart komputer agar berjalan sunyi (*silent*).
- **Diskusi Tim**:
  - **Winston (Architect)** menjelaskan bahwa hak Administrator wajib digunakan demi mem-bypass **UIPI (User Interface Privilege Isolation)**. Tanpa hak elevated, API hook global (`SetWindowsHookExW`) milik WinTick tidak akan bisa menangkap input atau memindahkan fokus ketika pengguna berada di jendela elevated (seperti Task Manager, CMD Admin, atau IDE elevated).
  - **John (PM)** dan **Sally (UX)** menekankan bahwa memunculkan prompt UAC setiap kali komputer menyala adalah pelanggaran berat terhadap kenyamanan pengguna (*user experience*).
  - **Amelia (Dev)** menawarkan solusi **Windows Task Scheduler**. Dengan membuat *Scheduled Task* saat instalasi awal yang memicu biner `wintick.exe` berjalan saat logon (`at logon`) dengan parameter `"Run with highest privileges"`, Windows akan meluncurkan daemon secara elevated di latar belakang secara *silent* tanpa memunculkan prompt UAC.
  - **Vex (Security)** khawatir bahwa menjalankan daemon sebagai Administrator penuh akan meningkatkan area serangan (*attack surface*).
  - **Winston (Architect)** mengajukan alternatif **UIAccess** (`uiAccess=true` di manifest), namun ini membutuhkan biner yang ditandatangani secara digital (*digital signature*) dan diletakkan di direktori aman seperti `%ProgramFiles%`. Hal ini dinilai terlalu menyulitkan untuk pengembangan awal dan distribusi biner yang belum ditandatangani.
  - **Amelia (Dev)** merekomendasikan manifest `requireAdministrator` untuk daemon, serta fungsi pemeriksaan token `is_elevated()` via `GetTokenInformation` agar jika biner dijalankan tanpa hak admin (misalnya manifes diabaikan), ia akan memicu MessageBox Tier 1 (Startup Fatal) lalu keluar bersih.

### Ronde 2: Evaluasi Soliditas & Celah Eskalasi Hak Akses (Privilege Escalation)
- **User Prompt**: Mempertanyakan apakah solusi pendaftaran Task Scheduler ini solid dan dijamin berhasil.
- **Diskusi Tim**:
  - **Amelia (Dev)** menegaskan solusi ini dijamin berhasil karena dijalankan di Session 1 (Desktop Aktif Pengguna), bukan Session 0 (isolasi Windows Service), sehingga keyboard hook global tetap berfungsi penuh. Cara ini juga digunakan oleh utilitas populer seperti CCleaner dan MSI Afterburner.
  - **Vex (Security)** mendeteksi celah keamanan kritis: Jika biner `wintick.exe` diletakkan di direktori yang dapat ditulis oleh pengguna standar (seperti Downloads atau folder user biasa), malware non-admin bisa menimpa biner tersebut. Saat Windows melakukan reboot, Task Scheduler akan menjalankan malware tersebut sebagai Administrator secara diam-diam (*silent privilege escalation*).
  - **Winston (Architect)** memberikan solusi pertahanan mutlak: Seluruh biner produksi wajib diletakkan di direktori `%ProgramFiles%\WinTick` yang dilindungi oleh Windows Access Control Lists (ACLs). User biasa tidak memiliki izin Write/Modify di folder ini.
  - **Dana (Pragmatist)** menyetujui bahwa untuk jalur pengembangan (dev) boleh berjalan lokal, tetapi paket produksi harus dikunci di `%ProgramFiles%`.
  - **Amelia (Dev)** menambahkan bahwa pendaftaran tugas scheduler akan dilakukan melalui settings GUI (`wintick-setting.exe`) yang dipanggil secara elevated dari daemon induk.

### Ronde 3: Alur UX Instalasi & Pembagian Peran Kode
- **User Prompt**: Menanyakan kapan tugas scheduler dibuat dan merumuskan langkah-langkah UX instalasi (1. Install biner, 2. Jalankan daemon elevated + UAC prompt 1x, 3. Munculkan onboarding via GUI settings untuk mendaftarkan task).
- **Diskusi Tim**:
  - **Sally (UX)** menekankan bahwa pendaftaran tugas scheduler harus transparan dan meminta persetujuan eksplisit dari pengguna di dalam UI onboarding.
  - **Amelia (Dev)** menyetujui pembagian peran: Pendaftaran Scheduled Task adalah wilayah kerja `wintick-setting.exe` (GUI) agar daemon `wintick.exe` tetap ramping (< 2MB RAM). Biner settings GUI otomatis berjalan elevated (tanpa prompt UAC kedua) karena diwarisi dari daemon induk elevated yang memanggilnya.
  - **Winston (Architect)** menyusun logika deteksi *first-run*: Daemon memeriksa keberadaan berkas `config.toml` di folder `%APPDATA%\WinTick`. Jika kosong, ia meluncurkan `wintick-setting.exe --onboarding`.
  - **John (PM)** merumuskan 5 langkah alur UX definitif:
    1. *Instalasi*: File diletakkan di `%ProgramFiles%\WinTick` (membutuhkan hak admin sekali).
    2. *Eksekusi Pertama*: User menjalankan `wintick.exe`, memicu prompt UAC pertama dan satu-satunya.
    3. *Onboarding*: Daemon mendeteksi ketiadaan config, memanggil `wintick-setting.exe --onboarding` secara elevated tanpa UAC prompt baru.
    4. *Registrasi*: Pengguna menyetujui opsi boot startup di GUI, lalu GUI mendaftarkan tugas Scheduler dan menulis `config.toml`.
    5. *Silent Boot*: Untuk seterusnya, booting Windows meluncurkan daemon secara elevated dan sunyi via Task Scheduler.

### Ronde 4 s/d 8: Advanced Elicitation (Red Team vs Blue Team & Pre-mortem)
- **User Prompt**: Mengaktifkan evaluasi keamanan dan ketahanan menggunakan *slash command* `/bmad-advanced-elicitation`.
- **Diskusi Tim (Ronde 5 - Red Team vs Blue Team)**:
  - **Red Team (Vex)** mengancam dengan serangan **DLL Hijacking** (meletakkan file DLL jahat di working directory scheduler).
  - **Blue Team (Amelia)** menangkis dengan memanggil fungsi API Windows `SetDllDirectoryW(L"")` di awal proses `main()` pada daemon untuk menghapus direktori kerja dari jalur pencarian DLL, serta mengosongkan parameter `Start in` pada tugas scheduler.
  - **Red Team (Grumbal)** mengancam dengan modifikasi aksi Scheduled Task.
  - **Blue Team (Amelia & Winston)** membuktikan bahwa Windows menolak manipulasi tugas scheduler berkategori *highest privileges* oleh proses non-admin (Access is Denied).
- **Diskusi Tim (Ronde 6 - Pre-mortem Analysis)**:
  - **Boundary (Edge-Case)** mengidentifikasi bahwa jika Scheduler menjalankan biner sebagai `SYSTEM`, folder `%APPDATA%` akan merujuk ke profil sistem, menyebabkan config tidak terbaca oleh GUI user.
  - **Blue Team (Amelia)** memitigasi dengan mewajibkan parameter `/ru "%USERNAME%"` pada pendaftaran tugas agar biner elevated berjalan di bawah identitas pengguna aktif.
  - **Grumbal (Red)** menduga adanya *Logon Race Condition* di mana daemon berjalan terlalu cepat saat boot sebelum sub-sistem DWM siap menerima global keyboard hook, menyebabkan `SetWindowsHookExW` gagal dan memicu exit fatal.
  - **Blue Team (Amelia)** memitigasi dengan menambahkan **Retry Loop** (mencoba ulang pasang hook 5 kali dengan jeda 1 detik) sebelum melempar Fatal Error.
  - **Winston (Architect)** mengkhawatirkan bentrokan multi-instansi daemon pada Fast User Switching.
  - **Blue Team (Amelia)** menetapkan mitigasi: (1) pendaftaran task spesifik untuk `%USERNAME%` aktif, (2) implementasi **Single Instance Lock** menggunakan Windows Named Mutex `Global\WinTickSingleInstanceMutex`.

---

## 3. Kesimpulan & Keputusan Utama

1. **Persyaratan Wajib Elevasi (UIPI Bypass)**: Daemon `wintick.exe` wajib memiliki hak Administrator menggunakan manifes `requireAdministrator` untuk menjamin global keyboard hook dan siklus fokus berjalan sukses pada jendela elevated.
2. **Startup Sunyi (Silent Boot)**: Masalah UAC prompt berulang saat boot diatasi dengan mendaftarkan tugas ke Windows Task Scheduler dengan pengaturan:
   - Pemicu: **At logon of the specific user**
   - Hak Akses: **Run with highest privileges**
   - Akun Eksekusi: **`%USERNAME%` aktif** (menjamin keselarasan jalur folder `%APPDATA%` konfigurasi).
3. **Pembagian Tanggung Jawab (Decoupling)**:
   - `wintick.exe` (Daemon): Tetap stateless dan ringan. Hanya mendeteksi *first-run* (ketiadaan `config.toml` di `%APPDATA%`), memanggil settings GUI, dan meluncurkan keyboard hook.
   - `wintick-setting.exe` (GUI Settings/Onboarding): Menampilkan UI Onboarding, meminta persetujuan pengguna untuk auto-start, membuat berkas `config.toml` (menyetel flag `first_run = false`), dan mendaftarkan Scheduled Task via perintah shell `schtasks.exe`.
4. **Pengerasan Keamanan (Security Hardening)**:
   - Lokasi instalasi produksi dikunci di folder `%ProgramFiles%\WinTick` dengan ACL tingkat administrator.
   - Penambahan baris fungsi `SetDllDirectoryW(L"")` di awal entri main daemon untuk menangkal eksploitasi DLL Hijacking.
   - Parameter `Start in` pada konfigurasi Scheduled Task wajib dikosongkan atau diarahkan secara absolut ke direktori `%ProgramFiles%`.
5. **Mitigasi Stabilitas & Startup**:
   - Menerapkan *Retry Loop* sebanyak 5 kali (jeda 1 detik) saat inisiasi global hook di startup untuk menghindari kegagalan akibat *Logon Race Condition* dengan sub-sistem DWM.
   - Menggunakan Named Mutex Windows (`Global\WinTickSingleInstanceMutex`) untuk memastikan hanya ada tepat satu instansi daemon yang aktif di memori sistem.

---

## 4. Daftar Tindakan Lanjut (Action Items)

| Tindakan / Tugas | Penanggung Jawab | Status |
| :--- | :--- | :--- |
| Perbarui dokumen spesifikasi Story 1.2 ([1-2-administrator-elevation-and-uipi-bypass.md](../implementation-artifacts/1-2-administrator-elevation-and-uipi-bypass.md)) dengan memasukkan arsitektur instalasi, pembagian peran onboarding, poin-poin mitigasi keamanan (DLL Hijacking), dan keandalan (Retry Loop, Named Mutex). | AI Agent (Antigravity) | [ ] Belum Selesai |
| Modifikasi manifes daemon `wintick.manifest` untuk meminta `requireAdministrator`. | AI Agent (Amelia) | [ ] Belum Selesai |
| Implementasikan fungsi C-FFI `is_elevated()` dan pengecekan token di `crates/daemon/src/main.rs`. | AI Agent (Amelia) | [ ] Belum Selesai |
| Tambahkan inisialisasi Named Mutex, `SetDllDirectoryW`, dan retry loop pada hook di `crates/daemon/src/main.rs`. | AI Agent (Amelia) | [ ] Belum Selesai |
| Desain alur onboarding pada `wintick-setting.exe` yang mencakup persetujuan pendaftaran Scheduled Task. | AI Agent (Sally) | [ ] Belum Selesai |
| Implementasikan perintah pembuatan Scheduled Task `/create /tn "WinTickDaemon" /tr ... /rl highest` di kode pengaturan startup. | AI Agent (Amelia) | [ ] Belum Selesai |
