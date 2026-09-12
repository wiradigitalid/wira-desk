//! Adaptive grid layout derived from work area dimensions.

pub const CARD_W: i32 = 240;
pub const CARD_H: i32 = 180;
pub const HEADER_H: i32 = 28;
pub const GUTTER: i32 = 16;
pub const MARGIN: i32 = 24;
pub const MAX_ROWS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardLayout {
    pub chrome_rect: Rect,
    pub header_rect: Rect,
    pub preview_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitcherLayout {
    pub overlay_rect: Rect,
    pub cols: usize,
    pub rows: usize,
    pub cards: Vec<CardLayout>,
    pub total_pages: usize,
    pub current_page: usize,
    pub page_indicator_rect: Option<Rect>,
}

/// Compute columns, rows, and per-page capacity from work area width and candidate count.
pub fn compute_grid(work_area_width: i32, candidate_count: usize) -> (usize, usize, usize) {
    if candidate_count == 0 {
        return (0, 0, 0);
    }
    let usable = (work_area_width - 2 * MARGIN).max(CARD_W);
    let max_cols = ((usable + GUTTER) / (CARD_W + GUTTER)).max(1) as usize;
    let cols = max_cols.min(candidate_count).max(1);
    let rows = candidate_count.div_ceil(cols).clamp(1, MAX_ROWS);
    let per_page = cols * rows;
    (cols, rows, per_page)
}

/// Compute full layout geometry for the given work area and candidates page.
pub fn compute_layout(work_area: Rect, candidate_count: usize, page: usize) -> SwitcherLayout {
    let (cols, rows, per_page) = compute_grid(work_area.width, candidate_count);
    if per_page == 0 {
        return SwitcherLayout {
            overlay_rect: Rect::new(work_area.x, work_area.y, 0, 0),
            cols: 0,
            rows: 0,
            cards: Vec::new(),
            total_pages: 0,
            current_page: 0,
            page_indicator_rect: None,
        };
    }

    let total_pages = candidate_count.div_ceil(per_page);
    let current_page = page.min(total_pages.saturating_sub(1));

    let page_start = current_page * per_page;
    let page_count = (candidate_count - page_start).min(per_page);

    let content_w = cols as i32 * CARD_W + (cols as i32 - 1).max(0) * GUTTER;
    let content_h = rows as i32 * CARD_H + (rows as i32 - 1).max(0) * GUTTER;

    let page_indicator_h = if total_pages > 1 { 24 } else { 0 };
    let overlay_w = content_w + 2 * MARGIN;
    let overlay_h = content_h + 2 * MARGIN + page_indicator_h;

    let overlay_x = work_area.x + (work_area.width - overlay_w) / 2;
    let overlay_y = work_area.y + (work_area.height - overlay_h) / 2;

    let mut cards = Vec::with_capacity(page_count);
    for i in 0..page_count {
        let col = (i % cols) as i32;
        let row = (i / cols) as i32;

        let card_x = overlay_x + MARGIN + col * (CARD_W + GUTTER);
        let card_y = overlay_y + MARGIN + row * (CARD_H + GUTTER);

        let chrome_rect = Rect::new(card_x, card_y, CARD_W, CARD_H);
        let header_rect = Rect::new(card_x + 6, card_y + 4, CARD_W - 12, HEADER_H);
        // Previews live strictly below the header area (no overlap with DWM thumbnail projection)
        let preview_rect = Rect::new(
            card_x + 4,
            card_y + HEADER_H + 4,
            CARD_W - 8,
            CARD_H - HEADER_H - 8,
        );

        cards.push(CardLayout {
            chrome_rect,
            header_rect,
            preview_rect,
        });
    }

    let page_indicator_rect = if total_pages > 1 {
        Some(Rect::new(
            overlay_x + MARGIN,
            overlay_y + MARGIN + content_h,
            content_w,
            page_indicator_h,
        ))
    } else {
        None
    };

    SwitcherLayout {
        overlay_rect: Rect::new(overlay_x, overlay_y, overlay_w, overlay_h),
        cols,
        rows,
        cards,
        total_pages,
        current_page,
        page_indicator_rect,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn columns_derive_from_the_work_area_and_never_overflow_it() {
        for width in [1366, 1920, 2560, 3840] {
            let work_area = Rect::new(0, 0, width, 1080);
            for count in [1, 2, 5, 8, 14, 20, 35, 60] {
                let layout = compute_layout(work_area, count, 0);
                assert!(
                    layout.overlay_rect.width <= work_area.width,
                    "overlay width {} exceeds work area width {}",
                    layout.overlay_rect.width,
                    work_area.width
                );
                assert!(
                    layout.overlay_rect.height <= work_area.height,
                    "overlay height {} exceeds work area height {}",
                    layout.overlay_rect.height,
                    work_area.height
                );
                assert!(layout.cols >= 1);
                assert!(layout.rows >= 1 && layout.rows <= MAX_ROWS);
                assert!(layout.total_pages >= 1);

                // Assert no page is ever empty across all pages
                for page in 0..layout.total_pages {
                    let page_layout = compute_layout(work_area, count, page);
                    assert!(
                        !page_layout.cards.is_empty(),
                        "page {} for count {} must not be empty",
                        page,
                        count
                    );
                }
            }
        }
    }

    #[test]
    fn candidates_beyond_one_page_paginate_rather_than_shrink() {
        let work_area = Rect::new(0, 0, 1920, 1080);
        let layout_small = compute_layout(work_area, 6, 0);
        let layout_large = compute_layout(work_area, 30, 0);

        // Card dimensions remain constant CARD_W x CARD_H instead of shrinking
        assert_eq!(layout_small.cards[0].chrome_rect.width, CARD_W);
        assert_eq!(layout_large.cards[0].chrome_rect.width, CARD_W);
        assert!(layout_large.total_pages > 1);
    }
}
