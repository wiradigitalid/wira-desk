//! Slint snapshot and layout tests for the Shortcuts pane.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::ShortcutField;
    use crate::shortcut_row_slint_snapshot::tests::{run_on_ui_thread, setup_shortcuts_window};
    use crate::theme;
    use i_slint_backend_testing::{ElementHandle, ElementRoot};
    use slint::ComponentHandle;

    /// One scroll gesture larger than the pane's scrollable extent, used to return to the top.
    ///
    /// It is not a "scroll to top" primitive and must not be read as one: it is an amount
    /// currently believed to exceed the extent. This pane is still gaining rows, and once the
    /// grows past this the walk below starts from a non-top position and a heading above it
    /// becomes unfindable — which fails loudly, in `find_scrolling_down`'s `None`, but reads as
    /// "the heading was never instantiated" rather than "the scroll idiom stopped reaching the
    /// top". If that day comes, scroll a viewport at a time until the first heading appears
    /// instead of raising this number.
    const SCROLL_PAST_TOP: f32 = 1200.0;

    /// The downward ladder, one home. Two divergent copies of this existed in this file.
    const SCROLL_DOWN_LADDER: [f32; 5] = [-200.0, -400.0, -600.0, -800.0, -1000.0];

    fn scroll_by(window: &crate::MainWindow, delta_y: f32) {
        window
            .window()
            .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                position: slint::LogicalPosition::new(300.0, 300.0),
                delta_x: 0.0,
                delta_y,
            });
    }

    /// Find an element by accessible label, scrolling down until it is instantiated.
    ///
    /// Two properties of Slint's testing backend make this necessary, and both were learned the
    /// expensive way: an element inside a scroll area does not exist in the accessible tree until
    /// it is scrolled into view, AND scrolling to the bottom recycles the ones at the top back out
    /// of it. So there is no scroll position from which all five groups are simultaneously
    /// present, and any test that measures them must measure each one while it is in view.
    fn find_scrolling_down(window: &crate::MainWindow, label: &str) -> Option<ElementHandle> {
        scroll_by(window, SCROLL_PAST_TOP);
        if let Some(el) = ElementHandle::find_by_accessible_label(window, label).next() {
            return Some(el);
        }
        for delta_y in SCROLL_DOWN_LADDER {
            scroll_by(window, delta_y);
            if let Some(el) = ElementHandle::find_by_accessible_label(window, label).next() {
                return Some(el);
            }
        }
        None
    }

    #[test]
    fn all_five_group_headings_are_in_the_rendered_tree() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Counted, not just iterated: an empty `GROUPS` would make the loop a no-op and this
            // test would pass having looked at nothing. That is the vacuity `b28d5ab` closed in
            // the width test with exactly this assertion, and this test never got it.
            let mut found = 0;
            for heading in ShortcutField::GROUPS {
                assert!(
                    find_scrolling_down(&window, heading).is_some(),
                    "Group heading '{heading}' must be findable in the rendered accessible tree"
                );
                found += 1;
            }
            assert_eq!(
                found,
                ShortcutField::GROUPS.len(),
                "every declared group heading must be checked, not skipped"
            );
            assert!(found >= 5, "the pane declares five groups; found {found}");

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn five_groups_fit_the_default_window_width_with_no_horizontal_scroll() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Sourced directly from main_window.slint's declared normal window dimensions
            let window_width = window.get_normal_width();
            let window_height = window.get_normal_height();
            window
                .window()
                .set_size(slint::LogicalSize::new(window_width, window_height));

            // Every heading, measured WHILE it is in view. Counted, because a loop that skips
            // what it cannot find is a loop that passes when it finds nothing: emptying the
            // heading labels made all three loops here no-ops and this test still went green.
            let mut headings_measured = 0;
            for heading in ShortcutField::GROUPS {
                let h = find_scrolling_down(&window, heading).unwrap_or_else(|| {
                    panic!("heading '{heading}' was never instantiated, so its width is unmeasured")
                });
                let right = h.absolute_position().x + h.size().width;
                assert!(
                    right <= window_width,
                    "Heading '{heading}' right edge ({right}) exceeds window width ({window_width})"
                );
                headings_measured += 1;
            }
            assert_eq!(
                headings_measured,
                ShortcutField::GROUPS.len(),
                "every group heading must be measured, not skipped"
            );

            // The right-hand cluster. Recycling means only the rows currently in view exist, so
            // the honest assertion is that at least one of each was actually measured — never
            // that a loop ran zero times without complaint.
            // Distinct elements, not visits: the walk revisits scroll positions, so counting
            // hits would double-count a row seen at two offsets and read as coverage it is not.
            let mut keycap_edges: Vec<f32> = Vec::new();
            let mut toggles_seen: std::collections::BTreeSet<String> = Default::default();
            let mut widest: f32 = 0.0;
            scroll_by(&window, SCROLL_PAST_TOP);
            for delta_y in std::iter::once(0.0).chain(SCROLL_DOWN_LADDER) {
                scroll_by(&window, delta_y);
                for field in ShortcutField::ALL {
                    let keycap_label = theme::shortcut_keycap_label(field.label());
                    for keycap in ElementHandle::find_by_accessible_label(&window, &keycap_label) {
                        let right = keycap.absolute_position().x + keycap.size().width;
                        assert!(
                            right <= window_width,
                            "Keycap for '{}' right edge ({right}) exceeds window width ({window_width})",
                            field.label()
                        );
                        widest = widest.max(right);
                        if !keycap_edges.iter().any(|e| (e - right).abs() < 0.5) {
                            keycap_edges.push(right);
                        }
                    }
                }
                for field in ShortcutField::ALL {
                    let toggle_label = format!("Enable {}", field.label());
                    for toggle in ElementHandle::find_by_accessible_label(&window, &toggle_label) {
                        let right = toggle.absolute_position().x + toggle.size().width;
                        assert!(
                            right <= window_width,
                            "Toggle for '{}' right edge ({right}) exceeds window width ({window_width})",
                            field.label()
                        );
                        widest = widest.max(right);
                        toggles_seen.insert(toggle_label.clone());
                    }
                }
            }
            assert!(
                !keycap_edges.is_empty(),
                "no keycap was measured — the loop found nothing and would have passed silently"
            );
            assert!(
                !toggles_seen.is_empty(),
                "no toggle was measured — the loop found nothing and would have passed silently"
            );
            // Printed so the margin is visible rather than implied. The bound asserted above is
            // the WINDOW edge; the pane's own content edge sits ~20px inside it, past the
            // sidebar and the scroll area's right padding. So up to that much real overflow into
            // the padding still passes here. Tightening it needs the ScrollView's own
            // `viewport-width` exposed, which is a production change and is recorded on the
            // ticket rather than guessed at with a second hardcoded number.
            eprintln!(
                "MEASUREMENT: widest right edge {widest} against window width {window_width}; \
                 {} distinct keycap edges, {} distinct toggles",
                keycap_edges.len(),
                toggles_seen.len()
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn the_last_rows_tooltip_paints_above_the_key_check_panel() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Sourced directly from main_window.slint's declared normal window dimensions
            let window_width = window.get_normal_width();
            let window_height = window.get_normal_height();
            window
                .window()
                .set_size(slint::LogicalSize::new(window_width, window_height));

            // The last row is Overlapping Stack
            let last_field = ShortcutField::Stack;
            let last_label = last_field.label();
            let last_desc = last_field.description();
            let last_desc_label = theme::shortcut_description_label(last_label);

            // Scroll down to the bottom of the pane so the last row sits near the bottom
            scroll_by(&window, -1200.0);
            let stack_block = ElementHandle::find_by_accessible_label(&window, &last_desc_label)
                .next()
                .expect("Overlapping Stack title block found in rendered tree at bottom");
            let s_pos = stack_block.absolute_position();
            let s_sz = stack_block.size();

            // Hover Overlapping Stack's title block to surface its tooltip
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(
                        s_pos.x + s_sz.width / 2.0,
                        s_pos.y + s_sz.height / 2.0,
                    ),
                });

            let tooltip_elem = ElementHandle::find_by_accessible_label(&window, last_desc)
                .next()
                .expect("Tooltip element found on hover for last row");
            let tooltip_pos = tooltip_elem.absolute_position();
            let tooltip_sz = tooltip_elem.size();

            // Find Key Check element in the pinned band
            let keycheck_title = ElementHandle::find_by_accessible_label(&window, "Key check")
                .next()
                .expect("Key check panel element found in rendered tree");
            let kc_pos = keycheck_title.absolute_position();

            // The tooltip's position extends into / over the Key Check zone:
            let tooltip_bottom = tooltip_pos.y + tooltip_sz.height;
            let kc_band_top = kc_pos.y - 10.0;
            assert!(
                tooltip_bottom >= kc_band_top,
                "Last row's tooltip bottom ({tooltip_bottom}) must reach or extend into Key Check vertical band ({kc_band_top})",
            );

            // Paint-order verification: The tooltip element MUST appear later in document/traversal
            // order than the Key Check panel, ensuring it is painted ON TOP OF Key Check.
            let all_elems = window.root_element().query_descendants().find_all();
            let tooltip_idx = all_elems
                .iter()
                .position(|e| e.accessible_label().as_deref() == Some(last_desc))
                .expect("Tooltip in element tree");
            let keycheck_idx = all_elems
                .iter()
                .position(|e| e.accessible_label().as_deref() == Some("Key check"))
                .expect("Key check in element tree");

            assert!(
                tooltip_idx > keycheck_idx,
                "Tooltip (idx={tooltip_idx}) must be visited/painted LATER than Key Check panel (idx={keycheck_idx}) so it renders above it"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
