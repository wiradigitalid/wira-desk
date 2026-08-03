#![windows_subsystem = "windows"]

mod app;
mod persistence;
mod theme;

use eframe::egui;

use shared::{config_path, migrate_appdata, Config};

use app::{Pane, SaveFeedback, SettingsModel, ShortcutField};
use persistence::{resolve_launch_intent, LaunchIntent};

fn main() -> eframe::Result {
    migrate_appdata();
    let intent = resolve_launch_intent(std::env::args());
    let saved = Config::load_or_default(&config_path());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 420.0])
            .with_min_inner_size([420.0, 320.0])
            .with_title("Wira Desk Settings"),
        ..Default::default()
    };

    eframe::run_native(
        "Wira Desk Settings",
        options,
        Box::new(move |cc| {
            let model = SettingsModel::new(saved, intent == LaunchIntent::Onboarding);
            // Install the Windows UI font once; theme changes later go through
            // `theme::apply`, which does not rebuild the font atlas.
            let font = theme::initialize(&cc.egui_ctx, model.theme);
            Ok(Box::new(SettingsApp { model, font }))
        }),
    )
}

struct SettingsApp {
    model: SettingsModel,
    /// Which typeface was installed. Surfaced in About so a fallback is
    /// visible rather than silently assumed.
    font: theme::LoadedFont,
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Without an explicit schedule, eframe only calls `ui` again in
        // response to input or animation — so if the window sits open but
        // idle (no mouse motion, no keyboard), the theme poll below never
        // runs again and an OS theme change made during that idle stretch
        // would sit invisible until some unrelated repaint happened to fire.
        // Requesting a low-frequency repaint keeps this check alive so the
        // "picked up... whenever the window is open" claim in
        // `theme::apply`'s doc comment is actually guaranteed.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));

        // Pick up an OS theme change while the window is open.
        let current = theme::detect_theme();
        if current != self.model.theme {
            self.model.theme = current;
            theme::apply(ui.ctx(), current);
            ui.ctx().request_repaint();
        }

        if self.model.onboarding.is_some() {
            self.onboarding_ui(ui);
        } else {
            self.settings_ui(ui);
        }
    }
}

impl SettingsApp {
    fn onboarding_ui(&mut self, ui: &mut egui::Ui) {
        let Some(step) = self.model.onboarding else {
            return;
        };

        ui.heading(step.heading());
        ui.add_space(8.0);
        ui.label(step.body());
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if step == app::OnboardingStep::Done {
                if ui.button("Finish").clicked() {
                    self.finish_onboarding();
                }
            } else {
                if ui.button("Next").clicked() {
                    self.model.advance_onboarding();
                }
                if ui.button("Skip Tutorial").clicked() {
                    self.model.skip_onboarding();
                }
            }
        });
    }

    /// Both "finished" and "skipped" write a valid configuration, which is what
    /// stops onboarding repeating on the next launch.
    fn finish_onboarding(&mut self) {
        self.model.save(&config_path());
        self.model.onboarding = None;
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        // The tab bar and the Shortcuts field list are drawn by iterating
        // `focus_order`'s declared sequence itself (filtered back into
        // `Pane`/`ShortcutField` values), rather than a second, independent
        // iteration over `Pane::ALL`/`ShortcutField::ALL` that merely happens
        // to agree with it today. That makes the declared order and the drawn
        // order structurally the same thing instead of two things a debug
        // assertion has to keep catching drift between.
        let stops = app::focus_order(self.model.pane);
        let mut drawn: Vec<&'static str> = Vec::new();

        ui.horizontal(|ui| {
            for pane in stops.iter().filter_map(|l| Pane::from_label(l)) {
                drawn.push(pane.label());
                let selected = self.model.pane == pane;
                if ui.selectable_label(selected, pane.label()).clicked() {
                    self.model.set_pane(pane);
                }
            }
        });
        ui.separator();

        match self.model.pane {
            Pane::General => self.general_pane(ui, &mut drawn),
            Pane::Shortcuts => self.shortcuts_pane(ui, &stops, &mut drawn),
            Pane::Layout => self.layout_pane(ui, &mut drawn),
            Pane::About => self.about_pane(ui),
        }

        ui.add_space(12.0);
        ui.separator();
        self.actions(ui, &mut drawn);

        #[cfg(debug_assertions)]
        if let Some(mismatch) = app::focus_order_mismatch(self.model.pane, &drawn) {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("focus order drift: {mismatch}"),
            );
        }
    }

    fn general_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        let c = theme::TOGGLE_AUTO_START;
        drawn.push(c.name);
        ui.checkbox(&mut self.model.draft.general.auto_start, c.name)
            .on_hover_text(c.description);
    }

    fn shortcuts_pane(
        &mut self,
        ui: &mut egui::Ui,
        stops: &[&'static str],
        drawn: &mut Vec<&'static str>,
    ) {
        for field in stops.iter().filter_map(|l| ShortcutField::from_label(l)) {
            drawn.push(field.label());
            let current = field.get(&self.model.draft).to_string();
            let listening = self.model.capture.is_listening_for(field);

            ui.horizontal(|ui| {
                ui.label(field.label());
                let text = if listening {
                    "Listening…".to_string()
                } else {
                    current.clone()
                };
                let response = ui
                    .button(text)
                    .on_hover_text(theme::SHORTCUT_SWITCHER.description);

                // The accessible value carries the listening state, so it is
                // not communicated by visual text alone.
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        self.model.capture.announcement(field, &current),
                    )
                });

                if response.clicked() {
                    if listening {
                        self.model.cancel_capture();
                    } else {
                        self.model.begin_capture(field);
                    }
                }
            });
        }

        if let app::CaptureState::Listening(field) = self.model.capture.clone() {
            ui.add_space(6.0);
            ui.label(theme::LISTENING_ANNOUNCEMENT);
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.model.cancel_capture();
            } else if let Some(combo) = captured_combination(ui.ctx()) {
                // A rejected capture must be surfaced the same way a rejected
                // Save is ('s accessible error description) — silently
                // discarding it would leave the user pressing the same keys
                // with no indication anything happened.
                if let Err(err) = self.model.accept_capture(&combo) {
                    self.model.feedback = SaveFeedback::Error(app::describe(field.label(), err));
                }
            }
        }
    }

    fn layout_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        let c = theme::TOGGLE_OVERLAPPING_STACK;
        drawn.push(c.name);
        drawn.push("Stack width percent");
        ui.checkbox(
            &mut self.model.draft.layout.enable_overlapping_stack,
            c.name,
        )
        .on_hover_text(c.description);
        ui.add(
            egui::Slider::new(&mut self.model.draft.layout.stack_width_percent, 10..=100)
                .text("Stack width percent"),
        );
    }

    fn about_pane(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wira Desk");
        ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));
        ui.label(match &self.font {
            theme::LoadedFont::System(name) => format!("Typeface: {name}"),
            theme::LoadedFont::Bundled => "Typeface: bundled fallback".to_string(),
        });
        ui.add_space(6.0);
        ui.label(
            "Wira Desk switches between windows of the application you are already using, \
             rather than every window on the system.",
        );
    }

    fn actions(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        drawn.push("Save");
        drawn.push("Revert");
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.model.save(&config_path());
            }
            if ui
                .add_enabled(self.model.is_dirty(), egui::Button::new("Revert"))
                .clicked()
            {
                self.model.revert();
            }
        });

        match &self.model.feedback {
            SaveFeedback::None => {}
            SaveFeedback::Saved { reload_signalled } => {
                let msg = if *reload_signalled {
                    "Settings saved and applied."
                } else {
                    "Settings saved. They apply the next time Wira Desk starts."
                };
                ui.colored_label(ui.visuals().hyperlink_color, msg);
            }
            SaveFeedback::Error(msg) => {
                ui.colored_label(ui.visuals().error_fg_color, msg);
            }
        }
    }
}

/// Translate the current egui key state into a shortcut string.
/// Returns `None` until a non-modifier key is actually pressed, so holding
/// modifiers alone never commits a half-formed combination.
fn captured_combination(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| {
        let m = i.modifiers;
        let key = i.keys_down.iter().copied().find(|k| !is_modifier_key(*k))?;
        let name = key_name(key)?;

        let mut parts: Vec<&str> = Vec::new();
        if m.ctrl {
            parts.push("ctrl");
        }
        if m.command && !m.ctrl {
            parts.push("win");
        }
        if m.alt {
            parts.push("alt");
        }
        if m.shift {
            parts.push("shift");
        }
        if parts.is_empty() {
            return None;
        }
        parts.push(name);
        Some(parts.join("+"))
    })
}

fn is_modifier_key(_key: egui::Key) -> bool {
    // egui reports modifiers through `Modifiers`, not `keys_down`, so anything
    // present here is already a main key.
    false
}

fn key_name(key: egui::Key) -> Option<&'static str> {
    use egui::Key;
    Some(match key {
        Key::Backtick => "backtick",
        Key::Tab => "tab",
        Key::Enter => "enter",
        Key::Space => "space",
        Key::Escape => "escape",
        Key::ArrowLeft => "left",
        Key::ArrowRight => "right",
        Key::ArrowUp => "up",
        Key::ArrowDown => "down",
        Key::A => "a",
        Key::B => "b",
        Key::C => "c",
        Key::D => "d",
        Key::E => "e",
        Key::F => "f",
        Key::G => "g",
        Key::H => "h",
        Key::I => "i",
        Key::J => "j",
        Key::K => "k",
        Key::L => "l",
        Key::M => "m",
        Key::N => "n",
        Key::O => "o",
        Key::P => "p",
        Key::Q => "q",
        Key::R => "r",
        Key::S => "s",
        Key::T => "t",
        Key::U => "u",
        Key::V => "v",
        Key::W => "w",
        Key::X => "x",
        Key::Y => "y",
        Key::Z => "z",
        Key::F1 => "f1",
        Key::F2 => "f2",
        Key::F3 => "f3",
        Key::F4 => "f4",
        Key::F5 => "f5",
        Key::F6 => "f6",
        Key::F7 => "f7",
        Key::F8 => "f8",
        Key::F9 => "f9",
        Key::F10 => "f10",
        Key::F11 => "f11",
        Key::F12 => "f12",
        _ => return None,
    })
}

#[cfg(test)]
mod resource_tests {
    /// Mirrors the daemon's guard: the version lives in both `Cargo.toml` and the
    /// resource script, and only a test can keep the two from drifting apart.
    #[test]
    fn version_resource_matches_cargo_manifest() {
        let rc = include_str!("../wiradesk-settings.rc");
        let version = env!("CARGO_PKG_VERSION");

        let mut fields: Vec<&str> = version.split('.').collect();
        while fields.len() < 4 {
            fields.push("0");
        }
        let comma = fields.join(",");

        for expected in [
            format!("FILEVERSION {comma}"),
            format!("PRODUCTVERSION {comma}"),
            format!("VALUE \"FileVersion\", \"{version}\""),
            format!("VALUE \"ProductVersion\", \"{version}\""),
        ] {
            assert!(
                rc.contains(&expected),
                "wiradesk-settings.rc is missing {expected:?} - it drifted from Cargo.toml version {version}"
            );
        }
    }
}
