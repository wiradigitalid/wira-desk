//! Selection navigation across cards and pages in cycle_order.

/// Move selection forward by one candidate.
/// Wrapping past the end returns to 0 on page 0.
pub fn select_next(
    candidate_count: usize,
    current_index: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 {
        return (0, 0);
    }
    let next_index = (current_index + 1) % candidate_count;
    let next_page = next_index.checked_div(per_page).unwrap_or(0);
    (next_index, next_page)
}

/// Move selection backward by one candidate.
/// Wrapping before 0 goes to last candidate and its corresponding page.
pub fn select_prev(
    candidate_count: usize,
    current_index: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 {
        return (0, 0);
    }
    let prev_index = if current_index == 0 {
        candidate_count - 1
    } else {
        current_index - 1
    };
    let prev_page = prev_index.checked_div(per_page).unwrap_or(0);
    (prev_index, prev_page)
}

/// Move selection up by one row within the current page and clamp at the top row.
pub fn select_up(
    candidate_count: usize,
    current_index: usize,
    cols: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 || cols == 0 {
        return (current_index, 0);
    }
    let page = current_index.checked_div(per_page).unwrap_or(0);
    let page_start = page * per_page;
    let in_page = current_index - page_start;

    let new_index = if in_page >= cols {
        current_index - cols
    } else {
        current_index
    };
    (new_index, page)
}

/// Move selection down by one row within the current page and clamp at the bottom row.
pub fn select_down(
    candidate_count: usize,
    current_index: usize,
    cols: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 || cols == 0 {
        return (current_index, 0);
    }
    let page = current_index.checked_div(per_page).unwrap_or(0);
    let page_start = page * per_page;
    let page_end = (page_start + per_page).min(candidate_count);

    let new_index = if current_index + cols < page_end {
        current_index + cols
    } else {
        current_index
    };
    (new_index, page)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn advancing_past_a_page_edge_turns_the_page() {
        let candidate_count = 14;
        let per_page = 6;

        // Start at last card of page 0 (index 5)
        let (idx, page) = select_next(candidate_count, 5, per_page);
        assert_eq!(idx, 6);
        assert_eq!(page, 1, "moving past page 0 edge turns to page 1");

        // Start at card 0 of page 1 (index 6) and move prev
        let (prev_idx, prev_page) = select_prev(candidate_count, 6, per_page);
        assert_eq!(prev_idx, 5);
        assert_eq!(
            prev_page, 0,
            "moving back past page 1 edge returns to page 0"
        );
    }

    #[test]
    fn up_and_down_clamp_within_page() {
        let candidate_count = 14;
        let per_page = 6;
        let cols = 3;

        // On page 0: row 0 is 0, 1, 2; row 1 is 3, 4, 5
        // Up on row 1 (index 4) -> index 1, page 0
        let (up_idx, up_page) = select_up(candidate_count, 4, cols, per_page);
        assert_eq!(up_idx, 1);
        assert_eq!(up_page, 0);

        // Up on row 0 (index 1) -> clamps at index 1, page 0
        let (up_clamp_idx, up_clamp_page) = select_up(candidate_count, 1, cols, per_page);
        assert_eq!(up_clamp_idx, 1);
        assert_eq!(up_clamp_page, 0);

        // Down on row 0 (index 1) -> index 4, page 0
        let (down_idx, down_page) = select_down(candidate_count, 1, cols, per_page);
        assert_eq!(down_idx, 4);
        assert_eq!(down_page, 0);

        // Down on row 1 (index 4) -> clamps at index 4, page 0
        let (down_clamp_idx, down_clamp_page) = select_down(candidate_count, 4, cols, per_page);
        assert_eq!(down_clamp_idx, 4);
        assert_eq!(down_clamp_page, 0);

        // On page 1: row 0 is 6, 7, 8; row 1 is 9, 10, 11
        // Up on index 7 (row 0 of page 1) -> clamps at index 7, page 1 (does not escape to page 0)
        let (p1_up_idx, p1_up_page) = select_up(candidate_count, 7, cols, per_page);
        assert_eq!(p1_up_idx, 7);
        assert_eq!(p1_up_page, 1);
    }
}
