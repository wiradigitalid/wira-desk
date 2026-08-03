---
baseline_commit: 720d155
---
# Story 2.2: Stateless Z-Order & App-Specific Matching

Status: ready-for-dev

## Story

As a pengguna multi-tasking,
I want WinTick hanya merotasi jendela milik aplikasi yang sedang aktif,
so that saya bisa berpindah antar dokumen Word tanpa terlempar ke Chrome.

## Acceptance Criteria

1. **Given** beberapa jendela dari aplikasi yang sama & berbeda **When** pintasan penukaran ditekan **Then** Worker mengevaluasi `EnumWindows` real-time (tanpa cache)
2. **And** hanya memfilter jendela dengan `Exe Name` identik dengan jendela aktif.

## Tasks / Subtasks

- [ ] Task 1: Modul worker cycling (`crates/daemon/src/worker.rs`)
  - [ ] `cycle_windows()` dipicu `Command::Cycle`
  - [ ] Ambil `GetForegroundWindow()`; resolusi exe name aktif via `GetWindowThreadProcessId` + `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` + `QueryFullProcessImageNameW`; ekstrak basename (lowercase)
- [ ] Task 2: Enumerasi stateless real-time (AC: 1, AD-3)
  - [ ] `EnumWindows` dengan callback yang mengumpulkan HWND kandidat KE DALAM buffer stack lokal (bukan cache global) tiap keypress
  - [ ] Urutan Z-order = urutan EnumWindows (top→bottom)
- [ ] Task 3: Same-app matching (AC: 2, AD-4)
  - [ ] Bandingkan exe basename tiap kandidat dengan exe aktif (case-insensitive). PID BUKAN identitas utama (Electron multi-proses)
  - [ ] Exclusion: lewati `WS_EX_TOOLWINDOW`, non-visible (detail penuh di Story 2.3)
- [ ] Task 4: Pindah fokus ke "berikutnya"
  - [ ] Dari daftar terurut Z-order, jendela aktif ada di posisi teratas; target = kandidat berikutnya (wrap-around). Angkat via `SetForegroundWindow` (+ trik `AllowSetForegroundWindow`/attach thread input bila perlu agar reliabel)
  - [ ] Graceful fail (NFR9): jika target invalid saat diaktifkan, lanjut ke kandidat berikutnya tanpa crash
- [ ] Task 5: Kernel-API sterilization (NFR8)
  - [ ] Callback HANYA pakai API non-blocking (`IsWindowVisible`, `GetWindowLongW`, `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`, `GetClassNameW`). DILARANG `SendMessage`/`GetWindowText`
- [ ] Task 6: Verifikasi: 3 jendela Notepad + 1 Chrome; cycle hanya berputar antar Notepad.

## Dev Notes
- **AD-3:** stateless — enumerasi tiap keypress, tanpa cache Z-order.
- **AD-4:** identitas = exe basename; class name hanya untuk exclusion.
- **NFR8/NFR9:** non-blocking APIs; skip target invalid.
- Feature windows-sys: `Win32_UI_WindowsAndMessaging` (EnumWindows, GetForegroundWindow, SetForegroundWindow, GetWindowLongW), `Win32_System_Threading` (OpenProcess, QueryFullProcessImageNameW), `Win32_Foundation`.
- Buffer kandidat: array statis kecil di stack (mis. `[HWND; 64]`) + count, hindari Vec bila mudah; namun worker thread boleh alokasi (bukan hook thread). Prioritaskan kebenaran; jaga tetap ringan.

### File
- [NEW] `crates/daemon/src/worker.rs`
- [MODIFY] `tray.rs`/`main.rs` (dispatch Command::Cycle → worker)

### References
- [Source: ARCHITECTURE-SPINE.md#AD-3,#AD-4] ; [Source: epics.md#Story-2.2] (FR-1)

## Dev Agent Record
### Agent Model Used
Claude Opus 4.8 (Claude Code)
### Completion Notes List
### File List
