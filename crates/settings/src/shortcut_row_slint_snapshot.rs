//! Slint snapshot and interaction tests for shortcut rows.

#[cfg(test)]
pub(crate) mod tests {
    use crate::app::{Pane, SaveFeedback, SettingsModel, ShortcutField};
    use crate::theme;
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

    pub(crate) fn setup_shortcuts_window(
    ) -> (MainWindow, Rc<RefCell<SettingsModel>>, std::path::PathBuf) {
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

    /// Type a value into the focused percentage field the way a person does: one character
    /// event per digit, through the window's real event queue.
    ///
    /// This is the whole point of `DEF-5`. Every other percentage test in this file reaches the
    /// value through `set_accessible_value`, which is the UI Automation `RangeValuePattern` path
    /// — a different code path from a keystroke arriving at Slint's `TextInput`. Those tests
    /// prove the commit-on-departure LOGIC is right once a value has landed in `typed_text`;
    /// none of them proves a keystroke can put one there, and on the live build it could not.
    ///
    /// Focusing is deliberately NOT part of what this helper exercises: it uses the same
    /// accessible default action the sibling tests use, so that a failure here is a failure of
    /// character entry and not of the click-to-focus mechanics. If the root-cause pass finds the
    /// defect is in focus after all, this helper is the wrong shape and should say so loudly
    /// rather than be quietly widened.
    fn type_digits(window: &crate::MainWindow, digits: &str) {
        for ch in digits.chars() {
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::SharedString::from(ch.to_string()),
                });
        }
    }

    #[test]
    fn a_real_keystroke_sequence_commits_a_typed_percentage() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            let field = ElementHandle::find_by_accessible_label(&window, "Snap percentage field")
                .next()
                .expect("Snap percentage field element found");
            field.invoke_accessible_default_action();

            // Clear what is there, then type "70" as two character events.
            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }
            type_digits(&window, "70");

            // The digits must have reached the row's own typed state. Read it back the way a
            // screen reader would, so the assertion does not depend on internals.
            let input = ElementHandle::find_by_accessible_label(&window, "Snap percentage input")
                .next()
                .expect("Snap percentage input element found");
            let seen = input.accessible_value().unwrap_or_default();
            eprintln!("MEASUREMENT: field reads {seen:?} after typing 70");
            assert_eq!(
                seen.as_str(),
                "70",
                "typing '7' then '0' must land in the field; it reads {seen:?}"
            );

            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "a typed value must not commit before departure"
            );

            window.invoke_save_clicked();

            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                70,
                "typing 70 and clicking Save must commit 70 to the draft"
            );
            assert_eq!(
                model.borrow().saved.snapping.percent_left,
                70,
                "typing 70 and clicking Save must save 70 to the config"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn a_multi_digit_keystroke_sequence_keeps_each_intermediate_digit_before_departure() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // --- 1. Snap to custom family (1-99) ---
            let field = ElementHandle::find_by_accessible_label(&window, "Snap percentage field")
                .next()
                .expect("Snap percentage field element found");
            field.invoke_accessible_default_action();

            let input = ElementHandle::find_by_accessible_label(&window, "Snap percentage input")
                .next()
                .expect("Snap percentage input element found");

            // Clear field
            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }

            // Type first digit '5'
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::SharedString::from("5"),
                });
            let intermediate_1 = input.accessible_value().unwrap_or_default();
            assert_eq!(
                intermediate_1.as_str(),
                "5",
                "Intermediate single digit '5' must not be reverted before departure"
            );

            // A live KeyCheck update while input has focus must not clobber the in-progress text
            crate::sync_key_check(&window, &_model.borrow());
            assert_eq!(
                input.accessible_value().unwrap_or_default().as_str(),
                "5",
                "Live KeyCheck update while input has focus must not clobber in-progress typed text"
            );

            // Type second digit '5' to make '55'
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::SharedString::from("5"),
                });
            let intermediate_2 = input.accessible_value().unwrap_or_default();
            assert_eq!(
                intermediate_2.as_str(),
                "55",
                "Full value '55' must be present after second digit"
            );

            // Test backspace down to single digit '5'
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Backspace.into(),
                });
            let after_backspace = input.accessible_value().unwrap_or_default();
            assert_eq!(
                after_backspace.as_str(),
                "5",
                "Field must read '5' after backspace, not revert to previous value"
            );

            let _ = std::fs::remove_file(&save_path);

            // --- 2. Stack family (10-100) on a clean window ---
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll down to bring the Overlapping Stack row into view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            let stack_field = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_FIELD.name,
            )
            .next()
            .expect("Stack width field element found");
            stack_field.invoke_accessible_default_action();

            let stack_input = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_INPUT.name,
            )
            .next()
            .expect("Stack width input element found");

            // Clear what is there
            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }

            // Type first digit '7' (momentarily below Stack min 10)
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::SharedString::from("7"),
                });
            let stack_intermediate_1 = stack_input.accessible_value().unwrap_or_default();
            assert_eq!(
                stack_intermediate_1.as_str(),
                "7",
                "Stack intermediate single digit '7' must not revert even if below min 10"
            );

            // Type second digit '5' to make '75'
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::SharedString::from("5"),
                });
            let stack_intermediate_2 = stack_input.accessible_value().unwrap_or_default();
            assert_eq!(
                stack_intermediate_2.as_str(),
                "75",
                "Stack field must read '75' after second digit"
            );

            // Test backspace deletion down to '7' (momentarily below min 10)
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Backspace.into(),
                });
            let stack_after_bs = stack_input.accessible_value().unwrap_or_default();
            assert_eq!(
                stack_after_bs.as_str(),
                "7",
                "Stack field must survive backspace deletion down to out-of-range '7' before departure"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn revert_discards_in_progress_typed_percentage_even_if_focused() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            let field = ElementHandle::find_by_accessible_label(&window, "Snap percentage field")
                .next()
                .expect("Snap percentage field element found");
            field.invoke_accessible_default_action();

            let input = ElementHandle::find_by_accessible_label(&window, "Snap percentage input")
                .next()
                .expect("Snap percentage input element found");

            // Clear what is there, then type '75'
            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }
            type_digits(&window, "75");

            assert_eq!(
                input.accessible_value().unwrap_or_default().as_str(),
                "75",
                "Field must show '75' while being typed"
            );

            // User clicks Revert without blurring the field
            window.invoke_revert_clicked();

            let input_after =
                ElementHandle::find_by_accessible_label(&window, "Snap percentage input")
                    .next()
                    .expect("Snap percentage input element found after revert");

            // The field must return to the saved value 50, NOT retain 75
            let reverted_val = input_after.accessible_value().unwrap_or_default();
            assert_eq!(
                reverted_val.as_str(),
                "50",
                "Clicking Revert must restore saved value in the field even while focused; got {reverted_val:?}"
            );

            // Now blur the field (e.g. by advancing focus or clicking elsewhere)
            window.invoke_start_capture(ShortcutField::Switcher as i32);

            // The draft must still be 50, NOT 75
            assert_eq!(
                model.borrow().draft.snapping.percent_left,
                50,
                "Subsequent blur must not commit the abandoned typed value 75"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn a_real_keystroke_sequence_out_of_range_is_refused() {
        use crate::app::SaveFeedback;
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            let field = ElementHandle::find_by_accessible_label(&window, "Snap percentage field")
                .next()
                .expect("Snap percentage field element found");
            field.invoke_accessible_default_action();

            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }
            // 0 is below MIN_SNAP_PERCENT. The fix that makes digits land MUST NOT also make an
            // out-of-range typed value silently clamp or silently save: the refusal is half of
            // what `SPEC-2-01` promised, and it is the half a fix for this defect could quietly
            // drop while looking correct.
            type_digits(&window, "0");

            window.invoke_save_clicked();

            assert!(
                matches!(model.borrow().feedback, SaveFeedback::Error(_)),
                "a typed out-of-range percentage must be refused with an actionable error, \
                 not clamped and not saved"
            );
            assert_eq!(
                model.borrow().saved.snapping.percent_left,
                50,
                "a refused value must leave the saved config untouched"
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

    #[test]
    fn toggling_action_enable_switch_updates_draft_and_saves() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // All actions start enabled
            assert!(model.borrow().draft.snapping.snap_percent_left_enabled);

            // Find the toggle switch for SnapPercentLeft
            let label = format!("Enable {}", ShortcutField::SnapPercentLeft.label());
            let mut toggles = ElementHandle::find_by_accessible_label(&window, &label);
            let switch = toggles
                .next()
                .expect("SnapPercentLeft enable toggle switch found");

            // Toggle off
            switch.invoke_accessible_default_action();

            assert!(
                !model.borrow().draft.snapping.snap_percent_left_enabled,
                "Toggling switch off must disable SnapPercentLeft in draft"
            );
            assert!(
                model.borrow().is_dirty(),
                "Disabling an action must mark draft dirty"
            );

            // Save is clicked
            window.invoke_save_clicked();

            assert!(
                !model.borrow().saved.snapping.snap_percent_left_enabled,
                "Saving must persist disabled action state"
            );

            // Toggle back on
            switch.invoke_accessible_default_action();
            assert!(
                model.borrow().draft.snapping.snap_percent_left_enabled,
                "Toggling switch on must enable SnapPercentLeft in draft"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn out_of_range_stack_width_is_refused_not_clamped() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Scroll down to bring the Overlapping Stack row into view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            // Focus the stack width field on the Overlapping Stack row
            let mut fields = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_FIELD.name,
            );
            let field_btn = fields.next().expect("Stack width field element found");
            field_btn.invoke_accessible_default_action();

            // Find stack width input element
            let mut inputs = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_INPUT.name,
            );
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

    #[test]
    fn stack_row_percent_commits_on_save_click() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Scroll down to bring the Overlapping Stack row into view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            // Find the percentage input for the Overlapping Stack row
            let mut inputs = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_INPUT.name,
            );
            let stack_input = inputs.next().expect("Stack width input element found");

            // Type '65' into the percentage field without pressing Enter
            stack_input.set_accessible_value("65");

            // Must NOT commit prematurely before departure
            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                50,
                "Stack width value must not commit to draft before departure"
            );

            // Save is clicked directly while the field has focus / was typed into
            window.invoke_save_clicked();

            // The typed value '65' must be committed on departure (save click), not discarded
            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                65,
                "Typing 65 then clicking Save must commit 65 to draft without requiring Enter"
            );
            assert_eq!(
                model.borrow().saved.layout.stack_width_percent,
                65,
                "Typing 65 then clicking Save must save 65 to config"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn a_real_keystroke_sequence_commits_a_typed_stack_percentage() {
        run_on_ui_thread(|| {
            let (window, model, save_path) = setup_shortcuts_window();

            // Scroll down to bring the Overlapping Stack row into view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            let field = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_FIELD.name,
            )
            .next()
            .expect("Stack width field element found");
            field.invoke_accessible_default_action();

            // Clear what is there, then type "65" as two character events.
            for _ in 0..3 {
                window
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: slint::platform::Key::Backspace.into(),
                    });
            }
            type_digits(&window, "65");

            let input = ElementHandle::find_by_accessible_label(
                &window,
                crate::theme::STACK_WIDTH_INPUT.name,
            )
            .next()
            .expect("Stack width input element found");
            let seen = input.accessible_value().unwrap_or_default();
            assert_eq!(
                seen.as_str(),
                "65",
                "typing '6' then '5' must land in the stack width field; it reads {seen:?}"
            );

            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                50,
                "a typed stack width value must not commit before departure"
            );

            window.invoke_save_clicked();

            assert_eq!(
                model.borrow().draft.layout.stack_width_percent,
                65,
                "typing 65 and clicking Save must commit 65 to the draft"
            );
            assert_eq!(
                model.borrow().saved.layout.stack_width_percent,
                65,
                "typing 65 and clicking Save must save 65 to the config"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn description_renders_as_a_tooltip_not_a_visible_line() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll to the top so Switcher row is at the top
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            let desc = ShortcutField::Switcher.description();

            // 1. When pointer and focus are elsewhere, the description is NOT rendered in the tree
            let initial_desc = ElementHandle::find_by_accessible_label(&window, desc).next();
            assert!(
                initial_desc.is_none(),
                "Description must not be rendered as an always-visible line when pointer and focus are elsewhere"
            );

            // 2. Reachable by keyboard focus: Tab navigation reaches the title block and surfaces tooltip
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Tab.into(),
                });

            let focused_desc = ElementHandle::find_by_accessible_label(&window, desc).next();
            assert!(
                focused_desc.is_some(),
                "Description must surface as a tooltip on keyboard focus"
            );

            // 3. Clear focus away via keyboard Tab: advancing focus to next row hides description
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(0.0, 0.0),
                });
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Tab.into(),
                });
            let cleared_desc = ElementHandle::find_by_accessible_label(&window, desc).next();
            assert!(
                cleared_desc.is_none(),
                "Description must cease rendering when focus advances away from the title block"
            );

            // 4. Reachable by mouse hover: hovering the title block surfaces the tooltip
            let switcher_desc_label =
                theme::shortcut_description_label(ShortcutField::Switcher.label());
            let mut title_blocks =
                ElementHandle::find_by_accessible_label(&window, &switcher_desc_label);
            let switcher_title = title_blocks
                .next()
                .expect("Shortcut description block found");
            let title_pos = switcher_title.absolute_position();
            let title_sz = switcher_title.size();
            let hover_pos = slint::LogicalPosition::new(
                title_pos.x + title_sz.width / 2.0,
                title_pos.y + title_sz.height / 2.0,
            );
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: hover_pos,
                });

            let hovered_desc = ElementHandle::find_by_accessible_label(&window, desc).next();
            assert!(
                hovered_desc.is_some(),
                "Description must surface as a tooltip on mouse hover"
            );

            // 5. Moving pointer away hides the tooltip again
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(0.0, 0.0),
                });
            let cleared_desc = ElementHandle::find_by_accessible_label(&window, desc).next();
            assert!(
                cleared_desc.is_none(),
                "Description must cease rendering when pointer leaves the title block"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn tooltip_does_not_overlap_the_row_title() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll to top
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            let desc = ShortcutField::Switcher.description();
            let title_label = ShortcutField::Switcher.label();
            let switcher_desc_label = theme::shortcut_description_label(title_label);

            let switcher_block =
                ElementHandle::find_by_accessible_label(&window, &switcher_desc_label)
                    .next()
                    .expect("Switcher description block found");
            let block_pos = switcher_block.absolute_position();
            let block_sz = switcher_block.size();

            // 1. Mouse hover: tooltip surfaces and does not intersect title bounds
            let hover_pos = slint::LogicalPosition::new(
                block_pos.x + block_sz.width / 2.0,
                block_pos.y + block_sz.height / 2.0,
            );
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: hover_pos,
                });

            let title_elem = ElementHandle::find_by_accessible_label(&window, title_label)
                .next()
                .expect("Title element found");
            let title_pos = title_elem.absolute_position();
            let title_sz = title_elem.size();

            let tooltip_elem = ElementHandle::find_by_accessible_label(&window, desc)
                .next()
                .expect("Tooltip element found on hover");
            let tooltip_pos = tooltip_elem.absolute_position();
            let tooltip_sz = tooltip_elem.size();

            // Assert no intersection in 2D bounding boxes (y interval does not overlap title y interval)
            let title_bottom = title_pos.y + title_sz.height;
            let tooltip_bottom = tooltip_pos.y + tooltip_sz.height;
            let y_overlaps = tooltip_pos.y < title_bottom && tooltip_bottom > title_pos.y;
            let x_overlaps = tooltip_pos.x < (title_pos.x + title_sz.width)
                && (tooltip_pos.x + tooltip_sz.width) > title_pos.x;
            assert!(
                !(x_overlaps && y_overlaps),
                "Hover tooltip bounds ({tooltip_pos:?}, {tooltip_sz:?}) must not intersect title bounds ({title_pos:?}, {title_sz:?})"
            );

            // 2. Clear hover and verify with keyboard Tab focus
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(0.0, 0.0),
                });
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                    text: slint::platform::Key::Tab.into(),
                });

            let kb_tooltip = ElementHandle::find_by_accessible_label(&window, desc)
                .next()
                .expect("Tooltip element found on keyboard focus");
            let kb_tooltip_pos = kb_tooltip.absolute_position();
            let kb_tooltip_sz = kb_tooltip.size();

            let kb_y_overlaps = kb_tooltip_pos.y < title_bottom
                && (kb_tooltip_pos.y + kb_tooltip_sz.height) > title_pos.y;
            let kb_x_overlaps = kb_tooltip_pos.x < (title_pos.x + title_sz.width)
                && (kb_tooltip_pos.x + kb_tooltip_sz.width) > title_pos.x;
            assert!(
                !(kb_x_overlaps && kb_y_overlaps),
                "Focus tooltip bounds must not intersect title bounds"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn tooltip_height_fits_its_own_text() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // 1. Check a standard short description (Switcher: "Cycles windows of the active app on this monitor.")
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            let desc = ShortcutField::Switcher.description();
            let title_label = ShortcutField::Switcher.label();
            let switcher_desc_label = theme::shortcut_description_label(title_label);

            let switcher_block =
                ElementHandle::find_by_accessible_label(&window, &switcher_desc_label)
                    .next()
                    .expect("Switcher description block found");
            let block_pos = switcher_block.absolute_position();
            let block_sz = switcher_block.size();

            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(
                        block_pos.x + block_sz.width / 2.0,
                        block_pos.y + block_sz.height / 2.0,
                    ),
                });

            let tooltip_elem = ElementHandle::find_by_accessible_label(&window, desc)
                .next()
                .expect("Tooltip element found for Switcher");
            let tooltip_sz = tooltip_elem.size();

            // Natural line height for 12px Segoe UI caption is ~14-16px, never the old unconstrained 42px
            assert!(
                tooltip_sz.height <= 20.0,
                "Tooltip rendered height ({}) must fit its single-line text, not a stretched box (was 42px)",
                tooltip_sz.height
            );

            // 2. Check the LONGEST description string shipped in this pane (SnapPercentBottom: 65 chars)
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(0.0, 0.0),
                });
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -700.0,
                });

            let longest_desc = ShortcutField::SnapPercentBottom.description();
            let longest_label = ShortcutField::SnapPercentBottom.label();
            let longest_block_label = theme::shortcut_description_label(longest_label);

            let longest_block =
                ElementHandle::find_by_accessible_label(&window, &longest_block_label)
                    .next()
                    .expect("SnapPercentBottom description block found");
            let l_pos = longest_block.absolute_position();
            let l_sz = longest_block.size();

            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition::new(
                        l_pos.x + l_sz.width / 2.0,
                        l_pos.y + l_sz.height / 2.0,
                    ),
                });

            let l_tooltip = ElementHandle::find_by_accessible_label(&window, longest_desc)
                .next()
                .expect("Tooltip element found for longest description");
            let l_tooltip_sz = l_tooltip.size();

            assert!(
                l_tooltip_sz.height <= 20.0,
                "Longest description tooltip height ({}) must still fit single-line height, not wrap/stretch",
                l_tooltip_sz.height
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn row_height_is_identical_toggle_on_and_toggle_off() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll to the top so Switching group is in view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            let switcher_label = ShortcutField::Switcher.label();
            let switcher_kc_label = theme::shortcut_keycap_label(switcher_label);
            let fallback_label = ShortcutField::Fallback.label();
            let fallback_kc_label = theme::shortcut_keycap_label(fallback_label);

            let get_keycap_y = |label: &str| -> f32 {
                ElementHandle::find_by_accessible_label(&window, label)
                    .next()
                    .unwrap_or_else(|| panic!("keycap for '{label}' not found"))
                    .absolute_position()
                    .y
            };

            // 1. Initial state (both toggles on)
            let kc0_on = get_keycap_y(&switcher_kc_label);
            let kc1_on = get_keycap_y(&fallback_kc_label);
            let pitch_on = kc1_on - kc0_on;
            eprintln!("Toggle ON: kc0={kc0_on}, kc1={kc1_on}, pitch={pitch_on}");

            // 2. Toggle Switcher off
            let toggle_label = format!("Enable {switcher_label}");
            let toggle_elem = ElementHandle::find_by_accessible_label(&window, &toggle_label)
                .next()
                .expect("Switcher toggle switch found");
            toggle_elem.invoke_accessible_default_action();

            let kc0_off = get_keycap_y(&switcher_kc_label);
            let kc1_off = get_keycap_y(&fallback_kc_label);
            let pitch_off = kc1_off - kc0_off;
            eprintln!("Toggle OFF: kc0={kc0_off}, kc1={kc1_off}, pitch={pitch_off}");

            const TOLERANCE: f32 = 0.5;
            assert!(
                (pitch_off - pitch_on).abs() <= TOLERANCE,
                "Row pitch must be identical toggle-on ({pitch_on}px) and toggle-off ({pitch_off}px)"
            );

            // 3. Confirm "Disabled" caption is rendered when toggle is off
            let disabled_caption =
                ElementHandle::find_by_accessible_label(&window, "Disabled").next();
            assert!(
                disabled_caption.is_some(),
                "'Disabled' caption must be rendered in the tree when toggle is off"
            );

            // 4. Toggle back on and confirm row pitch returns
            toggle_elem.invoke_accessible_default_action();
            let kc0_back = get_keycap_y(&switcher_kc_label);
            let kc1_back = get_keycap_y(&fallback_kc_label);
            let pitch_back = kc1_back - kc0_back;
            assert!(
                (pitch_back - pitch_on).abs() <= TOLERANCE,
                "Row pitch after toggling back on ({pitch_back}px) must match initial ({pitch_on}px)"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    /// The row pitch of one group, measured between two of its visible group headings.
    ///
    /// No element spans a row, so a row's height cannot be read directly. Consecutive keycaps sit
    /// at a fixed offset inside their rows, so the distance between two keycaps IS the height of
    /// the row between them — but only for keycaps in the SAME group: a gap across a group
    /// boundary additionally spans a heading and the card's padding.
    ///
    /// Which rows are instantiated depends on the scroll position, and the visible run does not
    /// begin at the first declared row. So the group is bounded by reading its own heading and the
    /// next one out of the rendered tree, and both MUST be present — an earlier version derived
    /// the boundaries by index arithmetic over the declared sequence, went red under a mutation
    /// that changed only which rows were in view, and blamed the description for it.
    fn group_row_pitches(window: &MainWindow, group: &str, next_group: &str) -> Vec<f32> {
        let heading_y = |label: &str| -> f32 {
            ElementHandle::find_by_accessible_label(window, label)
                .next()
                .unwrap_or_else(|| {
                    panic!("heading '{label}' is not in view, so no row of '{group}' is bracketed")
                })
                .absolute_position()
                .y
        };
        let (top, bottom) = (heading_y(group), heading_y(next_group));
        let mut rows: Vec<f32> = ShortcutField::ALL
            .into_iter()
            .filter(|f| f.group() == group)
            .filter_map(|f| {
                let label = theme::shortcut_keycap_label(f.label());
                let y = ElementHandle::find_by_accessible_label(window, &label)
                    .next()
                    .map(|k| k.absolute_position().y);
                y
            })
            .filter(|y| *y > top && *y < bottom)
            .collect();
        rows.sort_by(|a, b| a.partial_cmp(b).expect("keycap positions are comparable"));
        assert!(
            rows.len() >= 3,
            "group '{group}' shows {} rows before '{next_group}'; need 3 for two pitches",
            rows.len()
        );
        eprintln!("MEASUREMENT: group '{group}' keycap ys={rows:?}");
        rows.windows(2).map(|w| w[1] - w[0]).collect()
    }

    /// Record every key the window forwards to Rust, by registering the callback the test itself.
    ///
    /// `bind_callbacks` does not wire `key_pressed_event` — `main()` does, at `main.rs:721`, which
    /// is `DEF-8` — so in a test that callback is unset and a forward from the markup is a silent
    /// no-op. That is a fact about `main()`'s wiring, NOT about what a test can reach: a test can
    /// register its own listener, and then the markup's forwarding contract is directly
    /// observable. Two earlier records of mine said no automated test could reach this path at
    /// all; that was too strong, and these two tests are the correction.
    ///
    /// What is still untested is `main()`'s own registration. `DEF-8` carries that.
    fn record_forwarded_keys(window: &crate::MainWindow) -> Rc<RefCell<Vec<String>>> {
        let seen = Rc::new(RefCell::new(Vec::<String>::new()));
        let sink = Rc::clone(&seen);
        window.on_key_pressed_event(move |text, _ctrl, _alt, _shift, _meta| {
            sink.borrow_mut().push(text.to_string());
        });
        seen
    }

    fn press(window: &crate::MainWindow, text: slint::SharedString) {
        window
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed { text });
    }

    #[test]
    fn tab_is_forwarded_to_rust_before_it_is_rejected_for_focus_traversal() {
        // `DEC-005` (applied) reads the key check as a correlation of what the daemon's hook saw
        // against what the WINDOW saw. `key_handler` returns `reject` for Tab so Slint's focus
        // traversal can run, and if it did that BEFORE forwarding, the window's half of that pair
        // would be false for every Tab-containing chord — `Ctrl+Alt+Tab` would report "another
        // application claimed it" about an application that does not exist. Tab is also a
        // bindable chord key: `map_slint_key` maps `"\t"` and `U+0009` to `"tab"` in two arms.
        //
        // So the order is the assertion: forwarded first, rejected second.
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();
            let seen = record_forwarded_keys(&window);

            press(&window, slint::SharedString::from("x"));
            press(&window, slint::platform::Key::Tab.into());

            let keys = seen.borrow().clone();
            eprintln!("MEASUREMENT: forwarded keys {keys:?}");
            assert!(
                keys.iter().any(|k| k == "x"),
                "an ordinary key must reach the window callback, got {keys:?}"
            );
            assert!(
                keys.iter().any(|k| k == "\t"),
                "Tab must be forwarded to Rust before it is rejected for focus traversal, \
                 otherwise DEC-005's window signal is false for every Tab chord; got {keys:?}"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn an_unconsumed_key_still_reaches_rust_after_focus_moves_into_a_row() {
        // `DEF-9`. `key_handler` used to be a childless SIBLING of the content, and Slint bubbles a
        // key up the focused element's ANCESTORS — so the moment anything else took focus, every
        // one of the four behaviours behind this callback went dead: the Key Check diagnostic,
        // Escape-cancels-capture, chord capture, and onboarding step 2. The user-visible symptom
        // was a row stuck in "Listening…" with Escape unable to cancel it.
        //
        // One Tab press is the shortest route to that state, and it is the route `SPEC-4-03`
        // created. The percentage-field route (`pct_input.focus()`) predates it.
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();
            let seen = record_forwarded_keys(&window);

            // Move focus off `key_handler` the way a user does.
            press(&window, slint::platform::Key::Tab.into());
            seen.borrow_mut().clear();

            // A key no focused element consumes must still arrive, by bubbling.
            press(&window, slint::SharedString::from("y"));

            let keys = seen.borrow().clone();
            eprintln!("MEASUREMENT: after focus moved, forwarded keys {keys:?}");
            assert!(
                keys.iter().any(|k| k == "y"),
                "an unconsumed key must bubble to the window's FocusScope once focus has moved \
                 into a row, or chord capture and Escape are both dead there; got {keys:?}"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }

    #[test]
    fn row_height_is_independent_of_description_length() {
        // The criterion is that removing the always-visible description leaves the row's height
        // deterministic. Two earlier versions of this check could not fail for that reason:
        //
        //  - the first compared two *title blocks*, which after this change are structurally
        //    identical by construction — one `Text`, no conflict, both enabled — so no change to
        //    row height, padding or `min-height` could have made them differ;
        //  - the second compared real row pitches, but relied on the four descriptions of the
        //    group in view differing in length. They are "Snaps the window to the {left,right,
        //    top,bottom} edge at its configured percentage." — same length to within a word. A
        //    description rendered inline would wrap identically on all four and the pitches would
        //    stay uniform, so the defect would pass.
        //
        // So the length is not hoped for, it is IMPOSED: one row's description is replaced with a
        // pathologically long one and the group's pitches must not move.
        //
        // WHAT THIS CAN AND CANNOT SEE, established by running the mutations rather than by
        // reasoning, because two of the three readings above looked like proof and were not:
        //
        //  - Restoring the always-visible wrapping description does NOT move the pitch, and the
        //    injection above does not either. The component's header comment says why: Slint
        //    computes the row's preferred height at the text's UNWRAPPED width, so a wrapping
        //    description reports a one-line height, the row stays 50px, and the extra lines
        //    overflow the row instead of growing it. That was the historic defect - text drawn
        //    over the next row's divider - and it is invisible to geometry. The description's
        //    absence is proven by `description_renders_as_a_tooltip_not_a_visible_line`, which
        //    reads the tree; the overflow risk is a smoke-test item, recorded on the ticket.
        //  - It DOES fail when row height genuinely varies across a group. Verified by keying
        //    `min-height` to `description.character-count`: the pitches went [63, 57, 57] and
        //    the assertion named the group and both numbers. That is the property in the name,
        //    and the mutation is the most direct possible statement of it.
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();
            let (group, next_group) = (ShortcutField::GROUPS[3], ShortcutField::GROUPS[4]);

            let before = group_row_pitches(&window, group, next_group);

            // Impose the length. `rows_snap_custom` is the same model `sync_model_to_ui` fills,
            // so this is the production data path with one field made hostile.
            let rows: Vec<crate::ShortcutRowData> =
                slint::Model::iter(&window.get_rows_snap_custom()).collect();
            let mut hostile = rows.clone();
            hostile[1].description = slint::SharedString::from(
                "This description is deliberately long enough to wrap onto several lines at any                  plausible row width, which is the whole point of it: if the row still renders                  its description inline, this row grows and its neighbours do not.",
            );
            window.set_rows_snap_custom(slint::ModelRc::new(slint::VecModel::from(hostile)));

            let after = group_row_pitches(&window, group, next_group);
            eprintln!("MEASUREMENT: pitches before={before:?} after={after:?}");

            const TOLERANCE: f32 = 1.0;
            let baseline = before[0];
            for (label, pitches) in [("before", &before), ("after", &after)] {
                for (i, p) in pitches.iter().enumerate() {
                    assert!(
                        (p - baseline).abs() <= TOLERANCE,
                        "{label} the long description, pitch {i} of '{group}' is {p}, not {baseline}"
                    );
                }
            }

            let _ = std::fs::remove_file(&save_path);
        });
    }
    #[test]
    fn control_cluster_and_toggle_share_one_vertical_centre() {
        run_on_ui_thread(|| {
            let (window, _model, save_path) = setup_shortcuts_window();

            // Scroll to the top so Switcher row is in view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: 1200.0,
                });

            // Find keycap and toggle for Switcher
            let switcher_kc_label = theme::shortcut_keycap_label(ShortcutField::Switcher.label());
            let mut keycaps = ElementHandle::find_by_accessible_label(&window, &switcher_kc_label);
            let keycap = keycaps.next().expect("First row keycap found");

            let label = format!("Enable {}", ShortcutField::Switcher.label());
            let mut toggles = ElementHandle::find_by_accessible_label(&window, &label);
            let toggle = toggles.next().expect("First row toggle switch found");

            let keycap_pos = keycap.absolute_position();
            let keycap_sz = keycap.size();
            let keycap_centre_y = keycap_pos.y + keycap_sz.height / 2.0;

            let toggle_pos = toggle.absolute_position();
            let toggle_sz = toggle.size();
            let toggle_centre_y = toggle_pos.y + toggle_sz.height / 2.0;

            eprintln!(
                "MEASUREMENT: keycap y={}, h={}, centre_y={}; toggle y={}, h={}, centre_y={}; diff={}",
                keycap_pos.y, keycap_sz.height, keycap_centre_y,
                toggle_pos.y, toggle_sz.height, toggle_centre_y,
                (keycap_centre_y - toggle_centre_y).abs()
            );

            // Stated tolerance: within 1.0 logical pixel
            const TOLERANCE: f32 = 1.0;
            assert!(
                (keycap_centre_y - toggle_centre_y).abs() <= TOLERANCE,
                "Keycap centre ({keycap_centre_y}) and toggle centre ({toggle_centre_y}) must share one vertical centre within {TOLERANCE}px, but differed by {}px",
                (keycap_centre_y - toggle_centre_y).abs()
            );

            // Also test a row with percent control (SnapPercentLeft) to verify alignment holds
            // when bounded numeric stepper controls are present in the cluster.
            // Scroll down so Snap to custom group is in view.
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            let snap_label = format!("Enable {}", ShortcutField::SnapPercentLeft.label());
            let mut snap_toggles = ElementHandle::find_by_accessible_label(&window, &snap_label);
            let snap_toggle = snap_toggles.next().expect("Snap toggle switch found");
            let snap_toggle_pos = snap_toggle.absolute_position();
            let snap_toggle_sz = snap_toggle.size();
            let snap_toggle_centre_y = snap_toggle_pos.y + snap_toggle_sz.height / 2.0;

            let snap_kc_label =
                theme::shortcut_keycap_label(ShortcutField::SnapPercentLeft.label());
            let snap_keycap = ElementHandle::find_by_accessible_label(&window, &snap_kc_label)
                .find(|k| (k.absolute_position().y - snap_toggle_pos.y).abs() < 10.0)
                .expect("SnapPercentLeft keycap found");
            let snap_kc_pos = snap_keycap.absolute_position();
            let snap_kc_sz = snap_keycap.size();
            let snap_kc_centre_y = snap_kc_pos.y + snap_kc_sz.height / 2.0;

            assert!(
                (snap_kc_centre_y - snap_toggle_centre_y).abs() <= TOLERANCE,
                "Percent row keycap centre ({snap_kc_centre_y}) and toggle centre ({snap_toggle_centre_y}) must share vertical centre within {TOLERANCE}px"
            );

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
