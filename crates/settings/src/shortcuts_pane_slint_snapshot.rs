//! Slint snapshot and layout tests for the Shortcuts pane.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::ShortcutField;
    use crate::shortcut_row_slint_snapshot::tests::{run_on_ui_thread, setup_shortcuts_window};
    use i_slint_backend_testing::ElementHandle;
    use slint::ComponentHandle;

    fn scroll_through_all_groups(window: &crate::MainWindow) {
        for delta_y in [1200.0, -200.0, -400.0, -600.0, -800.0, -1000.0] {
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y,
                });
        }
    }

    #[test]
    fn all_five_group_headings_are_in_the_rendered_tree() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll to the very top first so the initial groups are in view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            for heading in ShortcutField::GROUPS {
                let mut found = ElementHandle::find_by_accessible_label(&window, heading).next();
                if found.is_none() {
                    // Slint testing backend instantiates elements in a scroll area only once
                    // scrolled into view. Scroll incrementally down to reveal subsequent groups.
                    for delta_y in [-200.0, -400.0, -600.0, -800.0, -1000.0] {
                        window.window().dispatch_event(
                            slint::platform::WindowEvent::PointerScrolled {
                                position: slint::LogicalPosition::new(300.0, 300.0),
                                delta_x: 0.0,
                                delta_y,
                            },
                        );
                        if let Some(el) =
                            ElementHandle::find_by_accessible_label(&window, heading).next()
                        {
                            found = Some(el);
                            break;
                        }
                    }
                }
                assert!(
                    found.is_some(),
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

            // Scroll through the entire pane so that all 5 groups and their rows are instantiated
            scroll_through_all_groups(&window);

            // Verify all 5 group headings fit within the window width
            for heading in ShortcutField::GROUPS {
                let mut headings = ElementHandle::find_by_accessible_label(&window, heading);
                if let Some(h) = headings.next() {
                    let pos = h.absolute_position();
                    let sz = h.size();
                    assert!(
                        pos.x + sz.width <= window_width,
                        "Heading '{heading}' right edge ({}) exceeds window width ({window_width})",
                        pos.x + sz.width
                    );
                }
            }

            // Verify all right-hand cluster elements (keycaps, toggles) fit within window_width
            for keycap in ElementHandle::find_by_accessible_label(&window, "Shortcut keycap") {
                let pos = keycap.absolute_position();
                let sz = keycap.size();
                assert!(
                    pos.x + sz.width <= window_width,
                    "Keycap right edge ({}) exceeds window width ({window_width})",
                    pos.x + sz.width
                );
            }

            for field in ShortcutField::ALL {
                let toggle_label = format!("Enable {}", field.label());
                for toggle in ElementHandle::find_by_accessible_label(&window, &toggle_label) {
                    let pos = toggle.absolute_position();
                    let sz = toggle.size();
                    assert!(
                        pos.x + sz.width <= window_width,
                        "Toggle for '{}' right edge ({}) exceeds window width ({window_width})",
                        field.label(),
                        pos.x + sz.width
                    );
                }
            }

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
