//! Slint snapshot and layout tests for the Shortcuts pane.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::ShortcutField;
    use crate::shortcut_row_slint_snapshot::tests::{run_on_ui_thread, setup_shortcuts_window};
    use crate::theme;
    use i_slint_backend_testing::ElementHandle;
    use slint::ComponentHandle;

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
        scroll_by(window, 1200.0); // back to the top
        if let Some(el) = ElementHandle::find_by_accessible_label(window, label).next() {
            return Some(el);
        }
        for delta_y in [-200.0, -400.0, -600.0, -800.0, -1000.0] {
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

            for heading in ShortcutField::GROUPS {
                assert!(
                    find_scrolling_down(&window, heading).is_some(),
                    "Group heading '{heading}' must be findable in the rendered accessible tree"
                );
            }

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
            let mut keycaps_measured = 0;
            let mut toggles_measured = 0;
            for delta_y in [1200.0, -300.0, -600.0, -900.0, -1200.0] {
                scroll_by(&window, delta_y);
                for keycap in
                    ElementHandle::find_by_accessible_label(&window, theme::SHORTCUT_KEYCAP.name)
                {
                    let right = keycap.absolute_position().x + keycap.size().width;
                    assert!(
                        right <= window_width,
                        "Keycap right edge ({right}) exceeds window width ({window_width})"
                    );
                    keycaps_measured += 1;
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
                        toggles_measured += 1;
                    }
                }
            }
            assert!(
                keycaps_measured > 0,
                "no keycap was measured — the loop found nothing and would have passed silently"
            );
            assert!(
                toggles_measured > 0,
                "no toggle was measured — the loop found nothing and would have passed silently"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
