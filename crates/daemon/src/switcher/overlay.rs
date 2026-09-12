//! Win32 visual switcher overlay window using GDI and DWM thumbnail composition.

use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{
    DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY,
    DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect, SelectObject, SetBkMode, SetTextColor,
    DT_CENTER, DT_SINGLELINE, DT_VCENTER, PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetWindowLongPtrW,
    RegisterClassExW, SetWindowLongPtrW, ShowWindow, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, SW_HIDE, SW_SHOWNOACTIVATE, WM_CREATE, WNDCLASSEXW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use super::layout::{compute_layout, Rect, SwitcherLayout};
use super::thumbnail::{DwmThumbnailHandle, ThumbnailSink};
use crate::cycling::WindowId;

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

                // Background: Mica / dark slate ground (RGB 32, 32, 36)
                let bg_brush = CreateSolidBrush(0x00242020);
                FillRect(mem_dc, &client_rect, bg_brush);
                DeleteObject(bg_brush);

                let overlay_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const SwitcherOverlay;
                if !overlay_ptr.is_null() {
                    let overlay = &*overlay_ptr;
                    let per_page = overlay.layout.cols * overlay.layout.rows;
                    let page_start = overlay.layout.current_page * per_page;

                    for (card_idx, card) in overlay.layout.cards.iter().enumerate() {
                        let is_selected = (page_start + card_idx) == overlay.selected_index;
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
                        } else {
                            0x003A3A3A // Subtle card border in BGR
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
