# 02: Adaptive grid derived from the work area, pagination, and spatial navigation

**What to build:**
The pure layout and selection engine that turns one row into a grid, and a grid into pages.
Columns come from the monitor's work area rather than from a fixed table; rows are capped at
three; anything beyond one page's worth of cards pages rather than shrinks. Up and Down move a
full row, and movement past a page edge turns the page. No Win32 in this module.

**Blocked by:** 01

**Status:** done

- [x] `cols = clamp(floor((usable + GUTTER) / (CARD_W + GUTTER)), 1, n)` with
      `usable = work_area.width - 2 * MARGIN`; `rows = min(ceil(n / cols), MAX_ROWS)`;
      `per_page = cols * rows`. `CARD_W`, `GUTTER`, `MARGIN` and `MAX_ROWS` are constants, not
      settings
- [x] Each card yields **two** rectangles — a chrome box and a preview box — because a DWM
      thumbnail composes above the destination window and nothing may be drawn on top of it
- [x] The preview box is uniform across cards; per-window aspect ratio is not computed, because
      DWM letterboxes the source inside the destination rectangle itself
- [x] Page count and active page index, with the leaving page's thumbnails unregistered and the
      arriving page's registered on every turn — at most `per_page` live thumbnails at any
      moment
- [x] Advancing past the last card on a page turns to the next page with the selection on its
      first card; retreating past the first card turns back to the previous page's last card
- [x] Up and Down move one row within a page and clamp at the top and bottom rows
- [x] Page indicator (`● ○ ○`) painted in the overlay's chrome area
- [x] Layout is per-monitor DPI aware and is computed against the work area of the monitor
      holding the active window, not the primary monitor
- [x] Unit tests cover 1, 2, 5, 8, 14, 20, 35 and 60 candidates against work-area widths of
      1366, 1920, 2560 and 3840, asserting no layout ever exceeds the work area and no page is
      empty
