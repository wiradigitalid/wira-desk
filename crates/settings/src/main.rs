#![windows_subsystem = "windows"]

mod app;
mod daemon_watch;
mod hookbridge;
mod persistence;
mod theme;

slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use std::rc::Rc;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, FALSE};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LWIN, VK_RWIN};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, MessageBoxW, SetForegroundWindow, ShowWindow, MB_ICONINFORMATION, MB_OK,
    SW_RESTORE,
};

use shared::constants::SETTINGS_SINGLE_INSTANCE_MUTEX;
use shared::{config_path, migrate_appdata, Config};

use app::{format_shortcut_display, Pane, SaveFeedback, SettingsModel, ShortcutField};
use daemon_watch::{DaemonWatch, Startup};
use persistence::{resolve_launch_intent, LaunchIntent};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn map_slint_key(text: &str) -> String {
    // 1. Direct match on special characters and standard text representations
    match text {
        "`" | "~" => return "backtick".to_string(),
        "\r" | "\n" => return "enter".to_string(),
        "\t" => return "tab".to_string(),
        " " => return "space".to_string(),
        "\u{1b}" | "Escape" => return "escape".to_string(),
        "Left" | "ArrowLeft" | "←" => return "left".to_string(),
        "Right" | "ArrowRight" | "→" => return "right".to_string(),
        "Up" | "ArrowUp" | "↑" => return "up".to_string(),
        "Down" | "ArrowDown" | "↓" => return "down".to_string(),
        _ => {}
    }

    // 2. Match Slint Key Unicode private-use values (Key::LeftArrow = \u{F702}, Key::RightArrow = \u{F703}, etc.)
    if text.len() == 1 || text.chars().count() == 1 {
        if let Some(c) = text.chars().next() {
            let u = c as u32;
            match u {
                0xF702 => return "left".to_string(),
                0xF703 => return "right".to_string(),
                0xF700 => return "up".to_string(),
                0xF701 => return "down".to_string(),
                0x0009 => return "tab".to_string(),
                0x000D | 0x000A => return "enter".to_string(),
                0x001B => return "escape".to_string(),
                0x0020 => return "space".to_string(),
                0xF704 => return "f1".to_string(),
                0xF705 => return "f2".to_string(),
                0xF706 => return "f3".to_string(),
                0xF707 => return "f4".to_string(),
                0xF708 => return "f5".to_string(),
                0xF709 => return "f6".to_string(),
                0xF70A => return "f7".to_string(),
                0xF70B => return "f8".to_string(),
                0xF70C => return "f9".to_string(),
                0xF70D => return "f10".to_string(),
                0xF70E => return "f11".to_string(),
                0xF70F => return "f12".to_string(),
                _ => {}
            }
        }
    }

    // 3. Filter out modifier tokens, or return normalized lowercase single alphanumeric key
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    if matches!(
        lower.as_str(),
        "control"
            | "ctrl"
            | "alt"
            | "shift"
            | "win"
            | "meta"
            | "shiftleft"
            | "shiftright"
            | "controlleft"
            | "controlright"
            | "altleft"
            | "altright"
    ) || trimmed.chars().all(|c| c.is_control())
    {
        String::new()
    } else {
        lower
    }
}

fn is_win_key_down() -> bool {
    // SAFETY: `GetAsyncKeyState` takes a standard Win32 virtual key code and reads
    // the instantaneous physical key state from the OS keyboard driver.
    unsafe {
        ((GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0)
            || ((GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0)
    }
}

fn sync_model_to_ui(window: &MainWindow, model: &SettingsModel) {
    // Mode
    let is_onboarding = model.onboarding.is_some();
    window.set_is_onboarding(is_onboarding);

    if let Some(step) = model.onboarding {
        let step_num = match step {
            app::OnboardingStep::Welcome => 1,
            app::OnboardingStep::TrySwitching => 2,
            app::OnboardingStep::Done => 3,
        };
        window.set_onboarding_step(step_num);
        window.set_onboarding_focus_index(model.onboarding_focus_index as i32);
        window.set_onboarding_sim_success(model.onboarding_simulated_success);
    } else {
        // Navigation Pane
        let pane_idx = match model.pane {
            Pane::General => 0,
            Pane::Shortcuts => 1,
            Pane::Layout => 2,
            Pane::VmExceptions => 3,
            Pane::About => 4,
        };
        window.set_current_pane(pane_idx);

        // General
        window.set_auto_start(model.draft.general.auto_start);

        // Shortcuts
        // The Shortcuts pane is built from `ShortcutField::ALL`, the one declared sequence
        // (`LBR-ST-14`). Nothing here names an individual action, so an action added to that
        // array appears in the pane, in the right group, with its conflict and swap state,
        // without a line changing in this file — which is the property the previous
        // one-property-per-field shape could not offer.
        let row_of = |field: ShortcutField| ShortcutRowData {
            index: field as i32,
            title: slint::SharedString::from(field.label()),
            description: slint::SharedString::from(field.description()),
            shortcut: slint::SharedString::from(format_shortcut_display(field.get(&model.draft))),
            conflict_name: slint::SharedString::from(
                model.find_conflict(field).map(|f| f.label()).unwrap_or(""),
            ),
            can_swap: model.can_swap(field),
        };
        let group_rows = |heading: &str| -> slint::ModelRc<ShortcutRowData> {
            let rows: Vec<ShortcutRowData> = ShortcutField::ALL
                .into_iter()
                .filter(|f| f.group() == heading)
                .map(row_of)
                .collect();
            slint::ModelRc::new(slint::VecModel::from(rows))
        };
        window.set_rows_switching(group_rows("Switching"));
        window.set_rows_snap(group_rows("Snap & resize"));
        window.set_rows_move(group_rows("Move & arrange"));

        // Listening index is the action's position in the declared sequence, which is exactly
        // its discriminant — so the pane, the focus order, and the collision precedence all
        // read the same number rather than three hand-kept mappings.
        let listening_idx = match &model.capture {
            app::CaptureState::Idle => -1,
            app::CaptureState::Listening(field) => *field as i32,
        };
        window.set_listening_field(listening_idx);

        // KeyCheck Diagnostic State
        window.set_kc_mod_ctrl(model.key_check.mod_ctrl);
        window.set_kc_mod_win(model.key_check.mod_win);
        window.set_kc_mod_alt(model.key_check.mod_alt);
        window.set_kc_mod_shift(model.key_check.mod_shift);
        window.set_kc_last_display(slint::SharedString::from(&model.key_check.last_display));
        window.set_kc_last_canonical(slint::SharedString::from(&model.key_check.last_canonical));
        window.set_kc_verdict(model.key_check.verdict as i32);
        window.set_kc_beat(model.key_check.beat);

        // Layout
        window.set_enable_stack(model.draft.layout.enable_overlapping_stack);
        window.set_stack_width_percent(model.draft.layout.stack_width_percent as i32);

        // About
        window.set_app_version(slint::SharedString::from(env!("CARGO_PKG_VERSION")));
        let typeface_name = match theme::detect_ui_font() {
            theme::LoadedFont::System(name) => format!("{name} (System Loaded)"),
            theme::LoadedFont::Bundled => "Bundled Fallback".to_string(),
        };
        window.set_app_typeface(slint::SharedString::from(typeface_name));

        // Save Bar / Footer
        window.set_is_dirty(model.is_dirty());
        let has_conflicts = model.has_any_conflict();
        window.set_has_conflict(has_conflicts);

        let (status_text, is_err, is_warn, is_succ) = match &model.feedback {
            SaveFeedback::None => {
                if has_conflicts {
                    let msg = if model.any_swappable_conflict() {
                        "⚠ Shortcut conflict detected. Resolve with Swap ⇄ or edit key."
                    } else {
                        "⚠ Shortcut conflict detected. Edit key to resolve."
                    };
                    (msg, false, true, false)
                } else {
                    ("Wira Desk is Active", false, false, false)
                }
            }
            SaveFeedback::Saved { reload_signalled } => {
                let msg = if *reload_signalled {
                    "Settings saved and applied"
                } else {
                    "Settings saved for next launch"
                };
                (msg, false, false, true)
            }
            SaveFeedback::Error(msg) => (msg.as_str(), true, false, false),
        };

        window.set_status_message(slint::SharedString::from(status_text));
        window.set_status_is_error(is_err);
        window.set_status_is_warning(is_warn);
        window.set_status_is_success(is_succ);
    }
}

fn main() -> Result<(), slint::PlatformError> {
    migrate_appdata();

    // Enforce single-instance for settings executable
    let mutex_name = wide(SETTINGS_SINGLE_INSTANCE_MUTEX);
    // SAFETY: `mutex_name` is a NUL-terminated wide string local that outlives the call.
    let mutex = unsafe { CreateMutexW(std::ptr::null(), FALSE, mutex_name.as_ptr()) };
    // SAFETY: `GetLastError` reads the thread-local Win32 error state immediately following `CreateMutexW`.
    if mutex == 0 || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let title = wide("Wira Desk");
        // SAFETY: `title` is a NUL-terminated wide string local. `FindWindowW` returns a window handle
        // or 0, and non-zero handles are forwarded safely to `ShowWindow` and `SetForegroundWindow`.
        unsafe {
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if hwnd != 0 {
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
            if mutex != 0 {
                CloseHandle(mutex);
            }
        }
        return Ok(());
    }

    // Settings does not open without a daemon to configure. Checked before the
    // window is built rather than on the first watch tick, so a user who runs
    // the executable directly gets an explanation instead of a window that
    // appears and vanishes half a second later.
    if daemon_watch::startup_decision(
        daemon_watch::daemon_is_running(),
        daemon_watch::allow_no_daemon(),
    ) == Startup::RefuseNoDaemon
    {
        let text = wide(daemon_watch::NO_DAEMON_MESSAGE);
        let caption = wide("Wira Desk Settings");
        // SAFETY: `text` and `caption` are NUL-terminated wide strings in locals
        // that outlive the call, and a null owner handle is the documented way to
        // show an unowned modal box — which is what this is, since no window
        // exists yet. The return value is the button pressed; there is only one.
        unsafe {
            MessageBoxW(
                0,
                text.as_ptr(),
                caption.as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        // The single-instance handle is deliberately left open, exactly as on
        // the normal path: it is released when the process exits.
        return Ok(());
    }

    let intent = resolve_launch_intent(std::env::args());
    let saved = Config::load_or_default(&config_path());

    let main_window = MainWindow::new()?;
    let model = Rc::new(std::cell::RefCell::new(SettingsModel::new(
        saved,
        intent == LaunchIntent::Onboarding,
    )));

    // Theme sync
    let is_dark = theme::detect_theme() == theme::ThemeMode::Dark;
    main_window.global::<Palette>().set_is_dark(is_dark);

    main_window.show()?;

    // Center on the current (or primary) monitor. Deferred to a single-shot
    // timer rather than run inline right after `show()`: measured directly
    // that `current_monitor()`/`primary_monitor()` both still answer `None`
    // at that point, because winit has not yet associated the window with a
    // monitor — that only happens once the event loop is actually pumping
    // messages, which does not start until `run()` below. A timer's
    // callback only fires once that loop is running, which is exactly the
    // delay needed. Without this the window opens wherever winit's own
    // default placement puts it — near the top-left corner, not the centre
    // a user expects a first-run or reopened window to land in.
    let debug_path = shared::log_path()
        .parent()
        .unwrap()
        .join("wiradesk-settings-center-debug.txt");
    let _ = std::fs::write(&debug_path, "timer registered\n");
    let window_weak_for_center = main_window.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(0), move || {
        let _ = std::fs::write(&debug_path, "timer fired\n");
        let Some(w) = window_weak_for_center.upgrade() else {
            let _ = std::fs::write(&debug_path, "timer fired, window upgrade failed\n");
            return;
        };
        w.window().with_winit_window(|win| {
            let current = win.current_monitor();
            let primary = win.primary_monitor();
            let chosen = current.clone().or_else(|| primary.clone());
            let window_size = win.outer_size();
            let position_before = win.outer_position();
            let _ = std::fs::write(
                &debug_path,
                format!(
                    "current_monitor={:?}\nprimary_monitor={:?}\nwindow_size={window_size:?}\nposition_before={position_before:?}\n",
                    current.map(|m| (m.position(), m.size())),
                    primary.map(|m| (m.position(), m.size())),
                ),
            );
            let Some(monitor) = chosen else {
                return;
            };
            let monitor_pos = monitor.position();
            let monitor_size = monitor.size();
            let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
            let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;
            win.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        });
    });

    // Initial state sync
    sync_model_to_ui(&main_window, &model.borrow());

    // Callbacks: Window Operations
    {
        let window_weak = main_window.as_weak();
        main_window.on_window_drag_requested(move || {
            if let Some(w) = window_weak.upgrade() {
                w.window().with_winit_window(|win| {
                    let _ = win.drag_window();
                });
            }
        });
    }
    {
        let window_weak = main_window.as_weak();
        main_window.on_window_minimize_clicked(move || {
            if let Some(w) = window_weak.upgrade() {
                w.window().set_minimized(true);
            }
        });
    }
    {
        let window_weak = main_window.as_weak();
        main_window.on_window_close_clicked(move || {
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
            }
        });
    }

    // Callbacks: Navigation & General
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_pane_selected(move |idx| {
            let mut m = model_rc.borrow_mut();
            let pane = match idx {
                0 => Pane::General,
                1 => Pane::Shortcuts,
                2 => Pane::Layout,
                3 => Pane::VmExceptions,
                4 => Pane::About,
                _ => Pane::General,
            };
            m.set_pane(pane);
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_auto_start_toggled(move |val| {
            let mut m = model_rc.borrow_mut();
            m.draft.general.auto_start = val;
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: Shortcuts
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_start_capture(move |idx| {
            let mut m = model_rc.borrow_mut();
            let field = ShortcutField::from_index(idx);
            if m.capture.is_listening_for(field) {
                m.cancel_capture();
            } else {
                m.begin_capture(field);
            }
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_swap_shortcuts(move |idx| {
            let mut m = model_rc.borrow_mut();
            let field = ShortcutField::from_index(idx);
            if let Some(conf) = m.find_conflict(field) {
                m.swap_shortcuts(field, conf);
            }
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: Layout
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_stack_toggled(move |val| {
            let mut m = model_rc.borrow_mut();
            m.draft.layout.enable_overlapping_stack = val;
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_width_changed(move |val| {
            let mut m = model_rc.borrow_mut();
            m.draft.layout.stack_width_percent = val.clamp(10, 100) as u32;
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: Save & Revert
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_save_clicked(move || {
            let mut m = model_rc.borrow_mut();
            m.save(&config_path());
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_revert_clicked(move || {
            let mut m = model_rc.borrow_mut();
            m.revert();
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: Onboarding Wizard
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_onboarding_next(move || {
            let mut m = model_rc.borrow_mut();
            m.advance_onboarding();
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_onboarding_back(move || {
            let mut m = model_rc.borrow_mut();
            if let Some(step) = m.onboarding {
                let prev = match step {
                    app::OnboardingStep::Welcome => app::OnboardingStep::Welcome,
                    app::OnboardingStep::TrySwitching => app::OnboardingStep::Welcome,
                    app::OnboardingStep::Done => app::OnboardingStep::TrySwitching,
                };
                m.onboarding = Some(prev);
            }
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_onboarding_skip(move || {
            let mut m = model_rc.borrow_mut();
            m.skip_onboarding();
            m.save(&config_path());
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_onboarding_finish(move || {
            let mut m = model_rc.borrow_mut();
            m.save(&config_path());
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
            }
        });
    }
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_onboarding_simulate(move || {
            let mut m = model_rc.borrow_mut();
            m.toggle_onboarding_simulation();
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: Keyboard Input & Shortcut Recording
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_key_pressed_event(move |text, ctrl, alt, shift, meta| {
            let mut m = model_rc.borrow_mut();

            // 1. Update KeyCheck Live Diagnostic observer BEFORE any early return.
            // DEC-005 / Section 3.4: All keyboard events must be captured by KeyCheck
            // even if modifier-only or Escape so the live keycap and pill state remain truthful.
            let win_active = meta || is_win_key_down();
            m.key_check.update_modifiers(ctrl, win_active, alt, shift);

            let display_key = map_slint_key(text.as_str());

            if !display_key.is_empty() {
                let mut parts = Vec::new();
                if ctrl {
                    parts.push("ctrl");
                }
                if win_active {
                    parts.push("win");
                }
                if alt {
                    parts.push("alt");
                }
                if shift {
                    parts.push("shift");
                }
                parts.push(&display_key);
                let raw_combo = parts.join("+");
                let formatted = format_shortcut_display(&raw_combo);
                let canonical = shared::Shortcut::parse(&raw_combo)
                    .and_then(|sc| sc.to_canonical_string())
                    .unwrap_or(raw_combo);

                // Same probe the liveness watch uses, so what Key Check
                // reports and what keeps this window open cannot disagree.
                let daemon_running = daemon_watch::daemon_is_running();

                let vk = shared::shortcut::vk_from_name(&display_key);
                m.key_check.record_key(
                    &formatted,
                    &canonical,
                    daemon_running,
                    (ctrl, win_active, alt, shift),
                    vk,
                );
            }

            // 2. Escape key cancels capture or closes onboarding
            if text == "\u{1b}" || text == "Escape" {
                if matches!(m.capture, app::CaptureState::Listening(_)) {
                    m.cancel_capture();
                } else if m.onboarding.is_some() {
                    m.skip_onboarding();
                    m.save(&config_path());
                    if let Some(w) = window_weak.upgrade() {
                        let _ = w.hide();
                    }
                    return;
                }
                if let Some(w) = window_weak.upgrade() {
                    sync_model_to_ui(&w, &m);
                }
                return;
            }

            // 3. If Listening for shortcut field capture
            if let app::CaptureState::Listening(field) = m.capture {
                let mut key_name = map_slint_key(text.as_str());

                // Fallback check: if Win key was pressed and text is backtick/tilde
                if key_name.is_empty() {
                    // SAFETY: `GetAsyncKeyState` with VK_OEM_3 (0xC0) reads instantaneous key state safely from OS.
                    let is_backtick_down = unsafe { (GetAsyncKeyState(0xC0) as u16 & 0x8000) != 0 };
                    if is_backtick_down {
                        key_name = "backtick".to_string();
                    }
                }

                // If user pressed only modifier (or unhandled system token), do not validate yet
                if key_name.is_empty() {
                    if let Some(w) = window_weak.upgrade() {
                        sync_model_to_ui(&w, &m);
                    }
                    return;
                }

                let mut combo_parts = Vec::new();
                if ctrl {
                    combo_parts.push("ctrl");
                }
                if win_active {
                    combo_parts.push("win");
                }
                if alt {
                    combo_parts.push("alt");
                }
                if shift {
                    combo_parts.push("shift");
                }

                combo_parts.push(&key_name);
                let combo_str = combo_parts.join("+");
                if let Err(err) = m.accept_capture(&combo_str) {
                    m.feedback = SaveFeedback::Error(app::describe(field.label(), err));
                }

                if let Some(w) = window_weak.upgrade() {
                    sync_model_to_ui(&w, &m);
                }
            } else if m.onboarding == Some(app::OnboardingStep::TrySwitching) {
                // In Onboarding Step 2: physical Win + ` toggles dummy focus
                if win_active && (text == "`" || text == "~") {
                    m.toggle_onboarding_simulation();
                    if let Some(w) = window_weak.upgrade() {
                        sync_model_to_ui(&w, &m);
                    }
                }
            } else {
                // Sync KeyCheck live state changes when Idle
                if let Some(w) = window_weak.upgrade() {
                    sync_model_to_ui(&w, &m);
                }
            }
        });
    }

    // Callbacks: Key Released Event (releases modifier pills when physical keys are lifted)
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_key_released_event(move |_text, ctrl, alt, shift, meta| {
            let mut m = model_rc.borrow_mut();
            let win_active = meta || is_win_key_down();
            m.key_check.update_modifiers(ctrl, win_active, alt, shift);
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks: KeyCheck Beat Timer Done (turns off pulse scale after 150ms)
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        main_window.on_key_check_beat_done(move || {
            let mut m = model_rc.borrow_mut();
            m.key_check.clear_beat();
            if let Some(w) = window_weak.upgrade() {
                sync_model_to_ui(&w, &m);
            }
        });
    }

    // Callbacks / background bridge: daemon hook reports (`DEC-004` / `DEC-005`).
    //
    // The receiver window runs its own message loop on a dedicated thread —
    // it must, since it blocks on `GetMessageW` — and only ever forwards raw,
    // `Send`-safe chord data through a channel. Every model mutation still
    // happens here, on the UI thread, drained by a Slint timer rather than by
    // reaching into the model from the background thread.
    let chord_rx = hookbridge::spawn();
    let chord_timer = slint::Timer::default();
    {
        let model_rc = Rc::clone(&model);
        let window_weak = main_window.as_weak();
        chord_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(20),
            move || {
                let mut m = model_rc.borrow_mut();
                let mut dirty = false;
                while let Ok(chord) = chord_rx.try_recv() {
                    dirty = true;
                    m.key_check.record_hook_report(
                        chord.vk,
                        chord.ctrl,
                        chord.win,
                        chord.alt,
                        chord.shift,
                    );

                    // DEC-004: while a field is actually listening, the
                    // daemon's report is the source of truth for what to
                    // accept — Slint's key-pressed text never arrives at all
                    // for a chord the Windows shell owns (Win+E, Win+1, or
                    // any of the six shortcuts already configured), which is
                    // DEF-3's reported symptom. Idempotent alongside the
                    // text-derived path below: whichever arrives first wins,
                    // and the second call is a no-op because capture is no
                    // longer `Listening`.
                    if let app::CaptureState::Listening(field) = m.capture {
                        match shared::shortcut::name_from_vk(chord.vk) {
                            None => {
                                m.feedback = SaveFeedback::Error(format!(
                                    "Wira Desk does not recognize this key (code 0x{:02X}). \
                                     Try a different key.",
                                    chord.vk
                                ));
                            }
                            Some(key_name) => {
                                let mut parts = Vec::new();
                                if chord.ctrl {
                                    parts.push("ctrl");
                                }
                                if chord.win {
                                    parts.push("win");
                                }
                                if chord.alt {
                                    parts.push("alt");
                                }
                                if chord.shift {
                                    parts.push("shift");
                                }
                                parts.push(key_name.as_str());
                                let combo = parts.join("+");
                                if let Err(err) = m.accept_capture(&combo) {
                                    m.feedback =
                                        SaveFeedback::Error(app::describe(field.label(), err));
                                }
                            }
                        }
                    }
                }
                // Drive the grace-period correlation regardless of whether a
                // report arrived this tick — a pending report's timeout must
                // still expire even on a tick with nothing new.
                m.key_check.tick();
                if dirty || m.key_check.beat {
                    if let Some(w) = window_weak.upgrade() {
                        sync_model_to_ui(&w, &m);
                    }
                }
            },
        );
    }

    // Daemon liveness watch. Settings is bound to the daemon's lifetime — see
    // `daemon_watch` for why a window left open after the daemon exits would be
    // reporting things that are no longer true.
    let daemon_timer = slint::Timer::default();
    {
        let window_weak = main_window.as_weak();
        let mut watch = DaemonWatch::new(daemon_watch::daemon_is_running);
        daemon_timer.start(
            slint::TimerMode::Repeated,
            daemon_watch::POLL_INTERVAL,
            move || {
                if !watch.tick() {
                    return;
                }
                // Hide the window *and* quit the loop. Hiding alone relies on
                // Slint's quit-on-last-window-closed behaviour, and if that ever
                // stops holding — or the window has already been hidden by
                // something else — the process would keep running with no window
                // at all, which is the exact state this watch exists to prevent.
                if let Some(w) = window_weak.upgrade() {
                    let _ = w.hide();
                }
                let _ = slint::quit_event_loop();
            },
        );
    }

    main_window.run()?;
    Ok(())
}
