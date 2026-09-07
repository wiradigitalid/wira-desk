//! Slint snapshot and interaction tests for layout pane.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::{Pane, SaveFeedback, SettingsModel};
    use crate::shortcut_row_slint_snapshot::tests::run_on_ui_thread;
    use crate::{bind_callbacks, sync_model_to_ui, MainWindow};
    use i_slint_backend_testing::ElementHandle;
    use shared::Config;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn setup_layout_window() -> (MainWindow, Rc<RefCell<SettingsModel>>, std::path::PathBuf) {
        let main_window = MainWindow::new().expect("MainWindow creation");
        let mut path = std::env::temp_dir();
        path.push(format!(
            "wiradesk-layout-pane-test-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::default();
        cfg.layout.stack_width_percent = 50;
        let model = Rc::new(RefCell::new(SettingsModel::new(cfg, false)));
        model.borrow_mut().set_pane(Pane::Layout);

        bind_callbacks(&main_window, &model, Some(path.clone()));
        sync_model_to_ui(&main_window, &model.borrow());

        (main_window, model, path)
    }

    #[test]
    fn out_of_range_stack_width_is_refused_not_clamped() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_layout_window();

            // Check in-pane title: must be "Layout", not "Layout & Snapping"
            let mut headings =
                ElementHandle::find_by_accessible_label(&window, "Layout pane heading");
            let heading_elem = headings.next().expect("Layout pane heading element found");
            assert_eq!(
                heading_elem.accessible_value(),
                Some("Layout".into()),
                "In-pane title must read 'Layout', not 'Layout & Snapping'"
            );

            // Focus the stack width field
            let mut fields = ElementHandle::find_by_accessible_label(&window, "Stack width field");
            let field_btn = fields.next().expect("Stack width field element found");
            field_btn.invoke_accessible_default_action();

            // Find stack width input element
            let mut inputs = ElementHandle::find_by_accessible_label(&window, "Stack width input");
            let width_input = inputs.next().expect("Stack width input element found");

            // Type out-of-range percentage 150
            width_input.set_accessible_value("150");

            // Value must not commit to draft before departure
            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                50,
                "Stack width percentage must not commit before departure"
            );

            // Save is clicked
            window.invoke_save_clicked();

            // 1. Must NOT be silently clamped to 100 in the model draft
            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                150,
                "Stack width percentage must not be silently clamped to 100 on input"
            );

            // 2. The out-of-range value must be refused with an actionable message
            if let SaveFeedback::Error(msg) = &model.borrow().feedback {
                assert!(
                    msg.contains("Stack width percentage") && msg.contains("between 10% and 100%"),
                    "Feedback message must be actionable: {msg}"
                );
            } else {
                panic!(
                    "Expected SaveFeedback::Error for out-of-range stack width percentage, got {:?}",
                    model.borrow().feedback
                );
            }

            // 3. Saved config must remain untouched at 50
            assert_eq!(
                model.borrow().saved.layout.stack_width_percent,
                50,
                "Saved config must not be overwritten when out-of-range"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
