//! System Tray `HICON` creation via runtime GDI (no external binary assets).
//! Icons are rasterized manually into a BGRA buffer then converted to a 32-bpp ARGB
//! `HICON` through `CreateDIBSection` + `CreateIconIndirect`. `base` provides the
//! normal icon; `with_warning`/`with_critical` add a red dot or cross overlay.

use core::ffi::c_void;
use std::mem::{size_of, zeroed};

use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BI_RGB,
    DIB_RGB_COLORS,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

pub const ICON_SIZE: i32 = 32;
const SIZE: usize = ICON_SIZE as usize;

/// Windows accent color (#0078D4) and derivatives, plus alert red (#E81123).
const ACCENT: [u8; 3] = [0xD4, 0x78, 0x00]; // BGR
const TITLEBAR: [u8; 3] = [0x9E, 0x5A, 0x00]; // BGR (darker accent)
const CONTENT: [u8; 3] = [0xF3, 0xF3, 0xF3]; // BGR (light gray)
const ALERT: [u8; 3] = [0x23, 0x11, 0xE8]; // BGR of #E81123 (R=E8,G=11,B=23)
const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// Top-down BGRA pixel buffer of size `SIZE x SIZE`.
struct Pixmap {
    data: Vec<u8>,
}

impl Pixmap {
    fn transparent() -> Self {
        Self {
            data: vec![0u8; SIZE * SIZE * 4],
        }
    }

    #[inline]
    fn set(&mut self, x: i32, y: i32, bgr: [u8; 3], a: u8) {
        if x < 0 || y < 0 || x >= ICON_SIZE || y >= ICON_SIZE {
            return;
        }
        let i = (y as usize * SIZE + x as usize) * 4;
        self.data[i] = bgr[0];
        self.data[i + 1] = bgr[1];
        self.data[i + 2] = bgr[2];
        self.data[i + 3] = a;
    }

    /// Fill rectangle [x0,x1) x [y0,y1).
    fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, bgr: [u8; 3]) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.set(x, y, bgr, 0xFF);
            }
        }
    }

    /// Fill circle at (cx,cy) with radius r.
    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, bgr: [u8; 3]) {
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r * r {
                    self.set(x, y, bgr, 0xFF);
                }
            }
        }
    }

    /// Draw a thick line (used for the cross) from (x0,y0) to (x1,y1).
    fn thick_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, half: i32, bgr: [u8; 3]) {
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let cx = (x0 as f32 + (x1 - x0) as f32 * t).round() as i32;
            let cy = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
            for dy in -half..=half {
                for dx in -half..=half {
                    self.set(cx + dx, cy + dy, bgr, 0xFF);
                }
            }
        }
    }
}

/// Rasterize the base Wira Desk glyph (window motif).
fn base_pixmap() -> Pixmap {
    let mut p = Pixmap::transparent();
    // Window frame (accent).
    p.fill_rect(4, 5, 28, 27, ACCENT);
    // Title bar (darker accent).
    p.fill_rect(4, 5, 28, 11, TITLEBAR);
    // Content area (light gray).
    p.fill_rect(6, 12, 26, 25, CONTENT);
    // Round corners: trim outer corner pixels.
    for &(x, y) in &[(4, 5), (27, 5), (4, 26), (27, 26)] {
        p.set(x, y, [0, 0, 0], 0);
    }
    p
}

/// Base icon (Normal state).
pub fn base() -> HICON {
    build_hicon(&base_pixmap())
}

/// Warning icon (Tier 2): base plus a small red dot at bottom-right.
pub fn with_warning() -> HICON {
    let mut p = base_pixmap();
    p.fill_circle(24, 24, 6, ALERT);
    build_hicon(&p)
}

/// Critical icon (Tier 3): dimmed base plus a large red cross.
pub fn with_critical() -> HICON {
    let mut p = base_pixmap();
    // Thick red cross covering the icon.
    p.thick_line(6, 6, 25, 25, 2, ALERT);
    p.thick_line(25, 6, 6, 25, 2, ALERT);
    // Thin white highlight in the center for contrast.
    let _ = WHITE;
    build_hicon(&p)
}

/// Convert a BGRA buffer to a 32-bpp ARGB `HICON`.
fn build_hicon(p: &Pixmap) -> HICON {
    // SAFETY: `BITMAPINFO` and `ICONINFO` are plain C structs of integers and handles, so
    // `zeroed` yields valid values and every field GDI reads is assigned below.
    //
    // The load-bearing precondition is the `copy_nonoverlapping` into `bits`: the
    // destination must be at least `p.data.len()` bytes. It is, and not by coincidence —
    // the header declares `ICON_SIZE × ICON_SIZE` at 32 bpp, so `CreateDIBSection`
    // allocates `SIZE * SIZE * 4` bytes, which is the exact length `Pixmap::transparent`
    // allocates from the same constant. Overlap is impossible: one buffer is our `Vec`, the
    // other is GDI's. The copy happens only after `bits` is confirmed non-null.
    //
    // `mask_bits` is `[u8; 128]`, which is precisely a 32×32 1-bpp mask with its scanlines
    // already 4-byte aligned, and `CreateBitmap` reads it only during the call.
    //
    // Handle discipline: `GetDC(0)` is checked before use because `ReleaseDC(0, 0)` would
    // violate the API contract, and it is released once, immediately after the DIB exists —
    // the section does not need the DC afterwards. `CreateIconIndirect` copies both bitmaps
    // into the icon, which is what makes deleting them straight afterwards correct rather
    // than a use-after-free; a zero `hbm_mask` is tolerated because a 32-bpp icon carries
    // its own alpha, so the guarded delete is the only thing that has to notice.
    unsafe {
        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize =
            size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = ICON_SIZE;
        bmi.bmiHeader.biHeight = -ICON_SIZE; // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let hdc = GetDC(0);
        if hdc == 0 {
            // `ReleaseDC(0, 0)` is UB per API contract; do not proceed to DIB.
            return 0;
        }
        let mut bits: *mut c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, 0, 0);
        ReleaseDC(0, hdc);

        if hbm_color == 0 || bits.is_null() {
            if hbm_color != 0 {
                DeleteObject(hbm_color);
            }
            return 0;
        }
        std::ptr::copy_nonoverlapping(p.data.as_ptr(), bits as *mut u8, p.data.len());

        // 1-bpp mask: modern shell ignores the mask (uses 32-bpp alpha), but
        // legacy paths (`DrawIcon`, Alt-Tab thumbnail) read the AND-mask. Zero-init
        // so bits are not garbage — 32×32×1bpp = 128 bytes, each scanline is
        // already word-aligned (4 bytes).
        let mask_bits = [0u8; 128];
        let hbm_mask = CreateBitmap(
            ICON_SIZE,
            ICON_SIZE,
            1,
            1,
            mask_bits.as_ptr() as *const c_void,
        );

        let mut ii: ICONINFO = zeroed();
        ii.fIcon = 1;
        ii.hbmColor = hbm_color;
        ii.hbmMask = hbm_mask;
        let hicon = CreateIconIndirect(&ii);

        DeleteObject(hbm_color);
        if hbm_mask != 0 {
            DeleteObject(hbm_mask);
        }
        hicon
    }
}
