//! Win32 visual switcher overlay window using GDI and DWM thumbnail composition.

use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY,
    DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect, ScreenToClient, SelectObject,
    SetBkMode, SetTextColor, CLEARTYPE_QUALITY, DEFAULT_CHARSET, DT_CENTER, DT_END_ELLIPSIS,
    DT_SINGLELINE, DT_VCENTER, FW_SEMIBOLD, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    VARIABLE_PITCH,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DrawIconEx, GetClientRect, GetCursorPos,
    GetWindowLongPtrW, RegisterClassExW, SetWindowLongPtrW, ShowWindow, CREATESTRUCTW, CS_HREDRAW,
    CS_VREDRAW, DI_NORMAL, GWLP_USERDATA, SW_HIDE, SW_SHOWNOACTIVATE, WM_CREATE, WM_LBUTTONDOWN,
    WM_MOUSEMOVE, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use super::layout::{compute_layout, Rect, SwitcherLayout};
use super::thumbnail::{DwmThumbnailHandle, ThumbnailSink};
use crate::cycling::WindowId;

const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_ROUND: u32 = 2;

/// Retrieves window title safely using `GetWindowTextW`.
/// `GetWindowTextW` against another process retrieves the cached caption maintained by the
/// window manager and is documented not to hang on an unresponsive window, making it admissible
/// on the switcher's render path even though the full enumeration sweep in `cycling/source.rs`
/// forbids it.
pub fn get_window_title(hwnd: HWND) -> String {
    let mut buf: [u16; 128] = [0; 128];
    // SAFETY: `GetWindowTextW` is called with a verified HWND and stack buffer of size 128.
    let len = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextW(
            hwnd,
            buf.as_mut_ptr(),
            buf.len() as i32,
        )
    };
    if len > 0 {
        String::from_utf16_lossy(&buf[..len as usize])
    } else {
        String::new()
    }
}

/// Retrieves the class or window icon using `WM_GETICON` or `GCLP_HICON`.
pub fn get_window_icon(hwnd: HWND) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassLongPtrW, SendMessageTimeoutW, GCLP_HICON, GCLP_HICONSM, ICON_BIG, ICON_SMALL,
        ICON_SMALL2, SMTO_ABORTIFHUNG,
    };
    let mut icon: isize = 0;

    // 1. Try SendMessageTimeoutW with ICON_SMALL, ICON_SMALL2, ICON_BIG (50ms timeout)
    for &icon_type in &[ICON_SMALL as usize, ICON_SMALL2 as usize, ICON_BIG as usize] {
        // SAFETY: SendMessageTimeoutW queries WM_GETICON with abort-if-hung flag and 50ms cap.
        let ok = unsafe {
            SendMessageTimeoutW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_GETICON,
                icon_type,
                0,
                SMTO_ABORTIFHUNG,
                50,
                &mut icon as *mut isize as *mut usize,
            )
        };
        if ok != 0 && icon != 0 {
            return icon;
        }
    }

    // 2. Try GetClassLongPtrW fallback
    for &gcl in &[GCLP_HICONSM, GCLP_HICON] {
        // SAFETY: GetClassLongPtrW safely queries class icon metadata.
        let h = unsafe { GetClassLongPtrW(hwnd, gcl) };
        if h != 0 {
            return h as isize;
        }
    }

    0
}

/// Read the current Windows app theme (Dark = true, Light = false).
pub fn is_dark_theme() -> bool {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let subkey: Vec<u16> = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let value: Vec<u16> = "AppsUseLightTheme"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;

    // SAFETY: RegGetValueW queries DWORD registry value safely.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut core::ffi::c_void,
            &mut size,
        )
    };

    if status == 0 {
        data == 0
    } else {
        true // Default to dark theme for overlay
    }
}

pub const SWITCHER_WINDOW_CLASS: &[u16] = &[
    'W' as u16, 'i' as u16, 'r' as u16, 'a' as u16, 'D' as u16, 'e' as u16, 's' as u16, 'k' as u16,
    'V' as u16, 'i' as u16, 's' as u16, 'u' as u16, 'a' as u16, 'l' as u16, 'S' as u16, 'w' as u16,
    'i' as u16, 't' as u16, 'c' as u16, 'h' as u16, 'e' as u16, 'r' as u16, 0,
];

/// Active visual switcher overlay state.
pub struct SwitcherOverlay {
    hwnd: HWND,
    layout: SwitcherLayout,
    selected_index: usize,
    candidates: Vec<WindowId>,
    thumbnails: Vec<DwmThumbnailHandle>,
    thumbnail_sink: Arc<Mutex<dyn ThumbnailSink>>,
}

impl SwitcherOverlay {
    pub fn new(thumbnail_sink: Arc<Mutex<dyn ThumbnailSink>>) -> Self {
        Self {
            hwnd: 0,
            layout: SwitcherLayout {
                overlay_rect: Rect::default(),
                cols: 0,
                rows: 0,
                cards: Vec::new(),
                total_pages: 0,
                current_page: 0,
                page_indicator_rect: None,
            },
            selected_index: 0,
            candidates: Vec::new(),
            thumbnails: Vec::new(),
            thumbnail_sink,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.hwnd != 0
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn per_page(&self) -> usize {
        self.layout.cols * self.layout.rows
    }

    pub fn cols(&self) -> usize {
        self.layout.cols
    }

    /// Show the switcher overlay for the given candidates and selected index.
    pub fn show(&mut self, work_area: Rect, candidates: Vec<WindowId>, selected_index: usize) {
        self.candidates = candidates;
        self.selected_index = selected_index;

        let per_page = if self.candidates.is_empty() {
            1
        } else {
            let (_, _, p) = super::layout::compute_grid(work_area.width, self.candidates.len());
            p.max(1)
        };
        let page = self.selected_index / per_page;

        self.layout = compute_layout(work_area, self.candidates.len(), page);
        self.ensure_window();

        if self.hwnd != 0 {
            // SAFETY: `ShowWindow` is called with the valid non-zero HWND created in `ensure_window`.
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    self.hwnd,
                    -1_isize, // HWND_TOPMOST
                    self.layout.overlay_rect.x,
                    self.layout.overlay_rect.y,
                    self.layout.overlay_rect.width,
                    self.layout.overlay_rect.height,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
                ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
            self.update_thumbnails();
        }
    }

    /// Hide and dismiss the switcher overlay, dropping all thumbnail handles.
    pub fn dismiss(&mut self) {
        self.thumbnails.clear();
        if self.hwnd != 0 {
            // SAFETY: `ShowWindow(SW_HIDE)` and `DestroyWindow` hide and tear down the popup window.
            unsafe {
                ShowWindow(self.hwnd, SW_HIDE);
                DestroyWindow(self.hwnd);
            }
            self.hwnd = 0;
        }
    }

    pub fn set_selected_index(&mut self, new_index: usize) {
        if self.candidates.is_empty() {
            return;
        }
        let per_page = self.layout.cols * self.layout.rows;
        let old_page = self.selected_index.checked_div(per_page).unwrap_or(0);
        let new_page = new_index.checked_div(per_page).unwrap_or(0);

        self.selected_index = new_index;
        if new_page != old_page {
            self.layout = compute_layout(self.layout.overlay_rect, self.candidates.len(), new_page);
            self.update_thumbnails();
        }

        if self.hwnd != 0 {
            // SAFETY: InvalidateRect triggers a repaint of the overlay for selection highlight.
            unsafe {
                windows_sys::Win32::Graphics::Gdi::InvalidateRect(self.hwnd, std::ptr::null(), 1);
            }
        }
    }

    fn ensure_window(&mut self) {
        if self.hwnd != 0 {
            return;
        }
        register_switcher_class();

        // SAFETY: `CreateWindowExW` creates a top-level unactivated popup tool window.
        // `lpCreateParams` passes `self as *mut SwitcherOverlay` which is stored into `GWLP_USERDATA`
        // on `WM_CREATE`. The pointer remains valid for the window's lifetime because `SwitcherOverlay`
        // is owned by the worker thread's thread-local `SWITCHER` and is only dismissed/destroyed in-place.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                SWITCHER_WINDOW_CLASS.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                self.layout.overlay_rect.x,
                self.layout.overlay_rect.y,
                self.layout.overlay_rect.width,
                self.layout.overlay_rect.height,
                0,
                0,
                0,
                self as *mut SwitcherOverlay as *const std::ffi::c_void,
            )
        };

        if hwnd != 0 {
            // Apply rounded corners preference via DwmSetWindowAttribute
            let corner_pref = DWMWCP_ROUND;
            // SAFETY: DwmSetWindowAttribute safely sets the window corner preference attribute on Windows 11.
            unsafe {
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_WINDOW_CORNER_PREFERENCE,
                    &corner_pref as *const u32 as *const std::ffi::c_void,
                    std::mem::size_of::<u32>() as u32,
                );
            }
        }

        self.hwnd = hwnd;
    }

    fn update_thumbnails(&mut self) {
        self.thumbnails.clear();
        if self.hwnd == 0 {
            return;
        }

        let per_page = self.layout.cols * self.layout.rows;
        let page_start = self.layout.current_page * per_page;

        for (card_idx, card) in self.layout.cards.iter().enumerate() {
            let candidate_idx = page_start + card_idx;
            if candidate_idx >= self.candidates.len() {
                break;
            }
            let src_window = self.candidates[candidate_idx];

            let mut sink = match self.thumbnail_sink.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let handle = match sink.register(self.hwnd, src_window.0) {
                Ok(h) => h,
                Err(_) => {
                    // Failed registration degrades gracefully; leaves an empty card
                    continue;
                }
            };
            drop(sink);

            // Configure thumbnail destination rectangle
            let props = DWM_THUMBNAIL_PROPERTIES {
                dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE | DWM_TNP_OPACITY,
                rcDestination: RECT {
                    left: card.preview_rect.x - self.layout.overlay_rect.x,
                    top: card.preview_rect.y - self.layout.overlay_rect.y,
                    right: (card.preview_rect.x - self.layout.overlay_rect.x)
                        + card.preview_rect.width,
                    bottom: (card.preview_rect.y - self.layout.overlay_rect.y)
                        + card.preview_rect.height,
                },
                rcSource: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                opacity: 255,
                fVisible: 1,
                fSourceClientAreaOnly: 0,
            };

            // SAFETY: `DwmUpdateThumbnailProperties` updates the destination rectangle on the active thumbnail handle.
            unsafe {
                let _ = DwmUpdateThumbnailProperties(handle, &props);
            }

            self.thumbnails.push(DwmThumbnailHandle::new(
                handle,
                Arc::clone(&self.thumbnail_sink),
            ));
        }
    }
}

fn register_switcher_class() {
    static REGISTERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if REGISTERED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let class_info = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(switcher_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0,
        hIcon: 0,
        hCursor: 0,
        hbrBackground: 0,
        lpszMenuName: std::ptr::null(),
        lpszClassName: SWITCHER_WINDOW_CLASS.as_ptr(),
        hIconSm: 0,
    };

    // SAFETY: Registers the `WiraDeskVisualSwitcher` window class.
    unsafe {
        let _ = RegisterClassExW(&class_info);
    }
}

unsafe extern "system" fn switcher_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            let data_ptr = (*cs).lpCreateParams as isize;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, data_ptr);
            0
        }
        WM_MOUSEMOVE => {
            let overlay_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SwitcherOverlay;
            if !overlay_ptr.is_null() {
                let overlay = &mut *overlay_ptr;
                let mut pt: POINT = std::mem::zeroed();
                if GetCursorPos(&mut pt) != 0 {
                    ScreenToClient(hwnd, &mut pt);
                    let per_page = overlay.layout.cols * overlay.layout.rows;
                    let page_start = overlay.layout.current_page * per_page;

                    for (card_idx, card) in overlay.layout.cards.iter().enumerate() {
                        let local_x = card.chrome_rect.x - overlay.layout.overlay_rect.x;
                        let local_y = card.chrome_rect.y - overlay.layout.overlay_rect.y;

                        if pt.x >= local_x
                            && pt.x < local_x + card.chrome_rect.width
                            && pt.y >= local_y
                            && pt.y < local_y + card.chrome_rect.height
                        {
                            let candidate_idx = page_start + card_idx;
                            if candidate_idx != overlay.selected_index
                                && candidate_idx < overlay.candidates.len()
                            {
                                overlay.set_selected_index(candidate_idx);
                            }
                            break;
                        }
                    }
                }
            }
            0
        }
        WM_LBUTTONDOWN => {
            let overlay_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SwitcherOverlay;
            if !overlay_ptr.is_null() {
                let overlay = &mut *overlay_ptr;
                let mut pt: POINT = std::mem::zeroed();
                if GetCursorPos(&mut pt) != 0 {
                    ScreenToClient(hwnd, &mut pt);
                    let per_page = overlay.layout.cols * overlay.layout.rows;
                    let page_start = overlay.layout.current_page * per_page;

                    for (card_idx, card) in overlay.layout.cards.iter().enumerate() {
                        let local_x = card.chrome_rect.x - overlay.layout.overlay_rect.x;
                        let local_y = card.chrome_rect.y - overlay.layout.overlay_rect.y;

                        if pt.x >= local_x
                            && pt.x < local_x + card.chrome_rect.width
                            && pt.y >= local_y
                            && pt.y < local_y + card.chrome_rect.height
                        {
                            let candidate_idx = page_start + card_idx;
                            if candidate_idx < overlay.candidates.len() {
                                overlay.set_selected_index(candidate_idx);
                                crate::ring::push(shared::Command::SwitcherCommit.as_u8());
                                if let Some(worker_hwnd) = crate::worker::worker_hwnd() {
                                    let _ =
                                        windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                            worker_hwnd,
                                            shared::constants::WM_APP_COMMAND_READY,
                                            0,
                                            0,
                                        );
                                }
                            }
                            break;
                        }
                    }
                }
            }
            0
        }
        windows_sys::Win32::UI::WindowsAndMessaging::WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc != 0 {
                let mut client_rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut client_rect);

                // Double buffered GDI paint
                let mem_dc = CreateCompatibleDC(hdc);
                let mem_bmp = CreateCompatibleBitmap(
                    hdc,
                    client_rect.right - client_rect.left,
                    client_rect.bottom - client_rect.top,
                );
                let old_bmp = SelectObject(mem_dc, mem_bmp);

                let dark = is_dark_theme();
                // Background: Mica / slate ground (Dark: 0x00242020 #202024, Light: 0x00F3F3F3 #F3F3F3)
                let bg_color = if dark { 0x00242020 } else { 0x00F3F3F3 };
                let bg_brush = CreateSolidBrush(bg_color);
                FillRect(mem_dc, &client_rect, bg_brush);
                DeleteObject(bg_brush);

                let overlay_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const SwitcherOverlay;
                if !overlay_ptr.is_null() {
                    let overlay = &*overlay_ptr;
                    let per_page = overlay.layout.cols * overlay.layout.rows;
                    let page_start = overlay.layout.current_page * per_page;

                    // Query monitor DPI for text/icon scaling (96 is 100% standard baseline)
                    // SAFETY: `GetDpiForWindow` safely queries the per-monitor DPI for the overlay HWND.
                    let dpi = match unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(hwnd) }
                    {
                        0 => 96,
                        d => d,
                    };
                    let icon_size = ((16 * dpi as i32) + 48) / 96;
                    let font_height = ((12 * dpi as i32) + 48) / 96;

                    // Create DPI-scaled font for titles
                    let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
                    // SAFETY: `CreateFontW` creates a logical GDI font with DPI-scaled height.
                    let hfont = unsafe {
                        CreateFontW(
                            -font_height,
                            0,
                            0,
                            0,
                            FW_SEMIBOLD as i32,
                            0,
                            0,
                            0,
                            DEFAULT_CHARSET as u32,
                            OUT_DEFAULT_PRECIS as u32,
                            0,
                            CLEARTYPE_QUALITY as u32,
                            VARIABLE_PITCH as u32,
                            font_name.as_ptr(),
                        )
                    };
                    let old_font = if hfont != 0 {
                        // SAFETY: SelectObject selects the created font into the memory device context.
                        unsafe { SelectObject(mem_dc, hfont) }
                    } else {
                        0
                    };

                    for (card_idx, card) in overlay.layout.cards.iter().enumerate() {
                        let candidate_idx = page_start + card_idx;
                        let is_selected = candidate_idx == overlay.selected_index;
                        let local_x = card.chrome_rect.x - overlay.layout.overlay_rect.x;
                        let local_y = card.chrome_rect.y - overlay.layout.overlay_rect.y;
                        let card_rect = RECT {
                            left: local_x,
                            top: local_y,
                            right: local_x + card.chrome_rect.width,
                            bottom: local_y + card.chrome_rect.height,
                        };

                        let border_color = if is_selected {
                            0x00D77800 // Vibrant accent blue in BGR
                        } else if dark {
                            0x003A3A3A // Subtle card border in Dark mode
                        } else {
                            0x00D0D0D0 // Subtle card border in Light mode
                        };
                        let border_brush = CreateSolidBrush(border_color);
                        FrameRect(mem_dc, &card_rect, border_brush);
                        if is_selected {
                            let inner_rect = RECT {
                                left: card_rect.left + 1,
                                top: card_rect.top + 1,
                                right: card_rect.right - 1,
                                bottom: card_rect.bottom - 1,
                            };
                            FrameRect(mem_dc, &inner_rect, border_brush);
                        }
                        DeleteObject(border_brush);

                        // Draw card header: Application icon and window title
                        if candidate_idx < overlay.candidates.len() {
                            let target_window = overlay.candidates[candidate_idx];
                            let h_local_x = card.header_rect.x - overlay.layout.overlay_rect.x;
                            let h_local_y = card.header_rect.y - overlay.layout.overlay_rect.y;

                            // Icon scaled with monitor DPI
                            let hicon = get_window_icon(target_window.0 as HWND);
                            if hicon != 0 {
                                DrawIconEx(
                                    mem_dc,
                                    h_local_x,
                                    h_local_y + 4,
                                    hicon,
                                    icon_size,
                                    icon_size,
                                    0,
                                    0,
                                    DI_NORMAL,
                                );
                            }

                            // Window title text at call site:
                            // Note on GetWindowTextW: retrieves cached caption from window manager,
                            // documented not to hang on unresponsive cross-process windows.
                            let title = get_window_title(target_window.0 as HWND);
                            let title_wide: Vec<u16> = title.encode_utf16().collect();
                            if !title_wide.is_empty() {
                                let mut text_rect = RECT {
                                    left: h_local_x + icon_size + 6,
                                    top: h_local_y + 2,
                                    right: h_local_x + card.header_rect.width,
                                    bottom: h_local_y + card.header_rect.height,
                                };
                                SetBkMode(mem_dc, TRANSPARENT as i32);
                                let text_color = if dark { 0x00FFFFFF } else { 0x001A1A1A };
                                SetTextColor(mem_dc, text_color);
                                DrawTextW(
                                    mem_dc,
                                    title_wide.as_ptr(),
                                    title_wide.len() as i32,
                                    &mut text_rect,
                                    DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
                                );
                            }
                        }
                    }

                    // Paint page indicator dots if total_pages > 1:
                    if overlay.layout.total_pages > 1 {
                        if let Some(indicator_rect) = overlay.layout.page_indicator_rect {
                            let local_x = indicator_rect.x - overlay.layout.overlay_rect.x;
                            let local_y = indicator_rect.y - overlay.layout.overlay_rect.y;
                            let mut text_rect = RECT {
                                left: local_x,
                                top: local_y,
                                right: local_x + indicator_rect.width,
                                bottom: local_y + indicator_rect.height,
                            };

                            let mut dot_chars: Vec<u16> = Vec::new();
                            for p in 0..overlay.layout.total_pages {
                                if p > 0 {
                                    dot_chars.push(' ' as u16);
                                }
                                if p == overlay.layout.current_page {
                                    dot_chars.push(0x25CF);
                                } else {
                                    dot_chars.push(0x25CB);
                                }
                            }

                            SetBkMode(mem_dc, TRANSPARENT as i32);
                            SetTextColor(mem_dc, 0x00C0C0C0);
                            DrawTextW(
                                mem_dc,
                                dot_chars.as_ptr(),
                                dot_chars.len() as i32,
                                &mut text_rect,
                                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                            );
                        }
                    }

                    if hfont != 0 {
                        // SAFETY: Restore previously selected font and delete created font object.
                        unsafe {
                            SelectObject(mem_dc, old_font);
                            DeleteObject(hfont);
                        }
                    }
                }

                // BitBlt to screen
                windows_sys::Win32::Graphics::Gdi::BitBlt(
                    hdc,
                    0,
                    0,
                    client_rect.right - client_rect.left,
                    client_rect.bottom - client_rect.top,
                    mem_dc,
                    0,
                    0,
                    windows_sys::Win32::Graphics::Gdi::SRCCOPY,
                );

                SelectObject(mem_dc, old_bmp);
                DeleteObject(mem_bmp);
                DeleteDC(mem_dc);
            }
            EndPaint(hwnd, &ps);
            0
        }
        windows_sys::Win32::UI::WindowsAndMessaging::WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn chrome_never_overlaps_the_preview_rectangle() {
        let work_area = Rect::new(0, 0, 1920, 1080);
        let layout = compute_layout(work_area, 6, 0);

        for card in &layout.cards {
            // Preview rectangle is strictly contained within the chrome rectangle
            assert!(card.preview_rect.x >= card.chrome_rect.x);
            assert!(card.preview_rect.y >= card.chrome_rect.y);
            assert!(
                card.preview_rect.x + card.preview_rect.width
                    <= card.chrome_rect.x + card.chrome_rect.width
            );
            assert!(
                card.preview_rect.y + card.preview_rect.height
                    <= card.chrome_rect.y + card.chrome_rect.height
            );
        }
    }

    #[test]
    fn a_failed_registration_still_renders_icon_and_title() {
        struct FailingThumbnailSink;
        impl ThumbnailSink for FailingThumbnailSink {
            fn register(&mut self, _destination: HWND, _source: HWND) -> Result<isize, i32> {
                Err(-1) // Registration failure (e.g. WDA_EXCLUDEFROMCAPTURE or access denied)
            }
            fn unregister(&mut self, _handle: isize) {}
        }

        let sink = Arc::new(Mutex::new(FailingThumbnailSink));
        let mut overlay = SwitcherOverlay::new(sink);
        // Showing overlay with failing sink does not panic or crash
        overlay.show(
            Rect::new(0, 0, 1920, 1080),
            vec![WindowId(1), WindowId(2)],
            0,
        );
        // Even when thumbnail registration fails, the overlay state remains valid and dismisses cleanly
        assert_eq!(overlay.thumbnails.len(), 0);
        overlay.dismiss();
    }
}
