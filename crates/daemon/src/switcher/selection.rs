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

/// Move selection up by one row (by `cols`).
pub fn select_up(
    candidate_count: usize,
    current_index: usize,
    cols: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 || cols == 0 {
        return (current_index, 0);
    }
    let new_index = if current_index >= cols {
        current_index - cols
    } else {
        current_index
    };
    let new_page = new_index.checked_div(per_page).unwrap_or(0);
    (new_index, new_page)
}

/// Move selection down by one row (by `cols`).
pub fn select_down(
    candidate_count: usize,
    current_index: usize,
    cols: usize,
    per_page: usize,
) -> (usize, usize) {
    if candidate_count == 0 || cols == 0 {
        return (current_index, 0);
    }
    let target = current_index + cols;
    let new_index = if target < candidate_count {
        target
    } else {
        current_index
    };
    let new_page = new_index.checked_div(per_page).unwrap_or(0);
    (new_index, new_page)
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
}
