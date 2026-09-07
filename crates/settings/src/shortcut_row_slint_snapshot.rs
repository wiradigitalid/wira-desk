//! Slint snapshot and interaction tests for shortcut rows.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::{Pane, SettingsModel, ShortcutField};
    use crate::{bind_callbacks, sync_model_to_ui, MainWindow};
    use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
    use shared::Config;
    use slint::ComponentHandle;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc;
    use std::sync::Mutex;

    type WorkerJob = Box<dyn FnOnce() + Send>;
    type WorkerSender = mpsc::Sender<WorkerJob>;

    static UI_WORKER: Mutex<Option<WorkerSender>> = Mutex::new(None);

    pub(crate) fn run_on_ui_thread<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let mut worker = UI_WORKER.lock().unwrap_or_else(|e| e.into_inner());
        let tx = worker.get_or_insert_with(|| {
            let (tx, rx) = mpsc::channel::<WorkerJob>();
            std::thread::spawn(move || {
                let _ = slint::platform::set_platform(Box::new(TestingBackend::new(
                    TestingBackendOptions {
                        mock_time: false,
                        threading: true,
                    },
                )));
                while let Ok(job) = rx.recv() {
                    job();
                }
            });
            tx
        });

        let (res_tx, res_rx) = mpsc::channel();
        tx.send(Box::new(move || {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let _ = res_tx.send(res);
        }))
        .unwrap();

        match res_rx.recv().unwrap() {
            Ok(val) => val,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    fn setup_shortcuts_window() -> (MainWindow, Rc<RefCell<SettingsModel>>, std::path::PathBuf) {
        let main_window = MainWindow::new().expect("MainWindow creation");
        let mut path = std::env::temp_dir();
        path.push(format!(
            "wiradesk-shortcut-row-test-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::default();
        cfg.snapping.percent_left = 50;
        let model = Rc::new(RefCell::new(SettingsModel::new(cfg, false)));
        model.borrow_mut().set_pane(Pane::Shortcuts);

        bind_callbacks(&main_window, &model, Some(path.clone()));
        sync_model_to_ui(&main_window, &model.borrow());

        main_window
            .window()
            .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                position: slint::LogicalPosition::new(300.0, 300.0),
                delta_x: 0.0,
                delta_y: -600.0,
            });

        (main_window, model, path)
    }

    #[test]
    fn typed_percentage_commits_on_save_click() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Find the percentage input for snap percent left
            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let left_input = inputs.next().expect("Snap percentage input element found");

            // Type '70' into the percentage field without pressing Enter
            left_input.set_accessible_value("70");

            // Must NOT commit prematurely before departure
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "Value must not commit to draft before departure"
            );

            // Save is clicked directly while the field has focus / was typed into
            window.invoke_save_clicked();

            // The typed value '70' must be committed on departure (save click), not discarded
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                70,
                "Typing 70 then clicking Save must commit 70 to the draft without requiring Enter"
            );
            assert_eq!(
                model.borrow().saved.snapping.percent_left,
                70,
                "Typing 70 then clicking Save must save 70 to the config"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn typed_percentage_commits_on_stepper_click() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Focus the percentage field
            let mut fields =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage field");
            let field_btn = fields.next().expect("Snap percentage field element found");
            field_btn.invoke_accessible_default_action();

            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let left_input = inputs.next().expect("Snap percentage input element found");

            // Type '70' into the percentage field without pressing Enter
            left_input.set_accessible_value("70");

            // Click '+' stepper
            let mut plus_buttons =
                ElementHandle::find_by_accessible_label(&window, "Increase snap percentage");
            let plus_btn = plus_buttons
                .next()
                .expect("Increase snap percentage button found");
            plus_btn.invoke_accessible_default_action();

            // Clicking '+' after typing 70 must compute from the typed value (70 + 1 = 71), not from 50 (yielding 51)
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                71,
                "Typing 70 then clicking '+' must yield 71, not 51"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn typed_percentage_commits_on_focus_change_to_another_row() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Focus the percentage field
            let mut fields =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage field");
            let field_btn = fields.next().expect("Snap percentage field element found");
            field_btn.invoke_accessible_default_action();

            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let left_input = inputs.next().expect("Snap percentage input element found");

            // Type '70' into the percentage field without pressing Enter
            left_input.set_accessible_value("70");

            // Must NOT commit prematurely before departure
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "Value must not commit to draft before departure"
            );

            // Focus changes away from the field: clicking another row's shortcut button
            window.invoke_start_capture(ShortcutField::Switcher as i32);

            // Moving focus away must commit the typed percentage to the draft
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                70,
                "Moving focus to another row must commit the typed percentage value"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn typed_percentage_commits_on_tab_navigation() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Focus the percentage field
            let mut fields =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage field");
            let field_btn = fields.next().expect("Snap percentage field element found");
            field_btn.invoke_accessible_default_action();

            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let left_input = inputs.next().expect("Snap percentage input element found");

            // Type '70' into the percentage field without pressing Enter
            left_input.set_accessible_value("70");

            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "Value must not commit to draft before departure"
            );

            // Press Tab to navigate focus away
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Tab.into(),
                });

            // Tab navigation commits the typed percentage
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                70,
                "Tab navigation away from percentage field must commit the typed value"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn an_out_of_range_percentage_committed_via_departure_is_refused() {
        use crate::app::SaveFeedback;
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            let mut fields =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage field");
            let field_btn = fields.next().expect("Snap percentage field element found");
            field_btn.invoke_accessible_default_action();

            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let left_input = inputs.next().expect("Snap percentage input element found");

            // Type out-of-range '150' into percentage field
            left_input.set_accessible_value("150");

            // Save is clicked
            window.invoke_save_clicked();

            // Refused with actionable error
            if let SaveFeedback::Error(msg) = &model.borrow().feedback {
                assert!(
                    msg.contains("Left edge snap percentage") && msg.contains("between 1% and 99%"),
                    "Error message must be actionable: {msg}"
                );
            } else {
                panic!(
                    "Expected SaveFeedback::Error, got {:?}",
                    model.borrow().feedback
                );
            }

            // Saved config remains untouched at 50
            assert_eq!(model.borrow().saved.snapping.percent_left, 50);

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn typed_percentage_survives_cross_row_stepper_click() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Focus Row A (snap percent left) field
            let mut fields =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage field");
            let field_a_btn = fields
                .next()
                .expect("Snap percentage field element found for Row A");
            field_a_btn.invoke_accessible_default_action();

            let mut inputs =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input");
            let row_a_input = inputs.next().expect("Snap percentage input for Row A");
            let _row_b_input = inputs.next().expect("Snap percentage input for Row B");

            // Type '70' into Row A's percentage field without pressing Enter
            row_a_input.set_accessible_value("70");

            // Verify Row A's draft has not committed yet before departure
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "Row A value must not commit to draft before departure"
            );

            // Click '+' stepper on a DIFFERENT row (Row B, snap percent right)
            let mut plus_buttons =
                ElementHandle::find_by_accessible_label(&window, "Increase snap percentage");
            let _row_a_plus = plus_buttons
                .next()
                .expect("Increase snap percentage button for Row A");
            let row_b_plus = plus_buttons
                .next()
                .expect("Increase snap percentage button for Row B");
            row_b_plus.invoke_accessible_default_action();

            // Row A's pending typed value '70' must survive and commit to the draft,
            // while Row B's value is stepped (50 + 1 = 51)
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                70,
                "Row A typed value (70) must be committed when Row B stepper is clicked"
            );
            assert_eq!(
                model.borrow().draft.snapping.percent_right,
                51,
                "Row B stepped value must be 51"
            );

            // Save is clicked afterwards: both values must be saved to disk
            window.invoke_save_clicked();
            assert_eq!(
                model.borrow().saved.snapping.percent_left,
                70,
                "Row A value (70) must be saved to config"
            );
            assert_eq!(
                model.borrow().saved.snapping.percent_right,
                51,
                "Row B value (51) must be saved to config"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
