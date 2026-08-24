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

    let (width, height) = if intent == LaunchIntent::Onboarding {
        (540.0, 420.0)
    } else {
        (660.0, 580.0)
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([500.0, 380.0])
            .with_title("Wira Desk Settings")
            .with_decorations(false),
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

        // Fill entire window background with Mica Dark/Light base
        let frame_bg = if self.model.theme == theme::ThemeMode::Dark {
            theme::COLOR_BG_MICA
        } else {
            egui::Color32::from_rgb(0xf3, 0xf3, 0xf3)
        };

        egui::Frame::new().fill(frame_bg).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            if self.model.onboarding.is_some() {
                self.onboarding_ui(ui);
            } else {
                self.settings_ui(ui);
            }
        });
    }
}

impl SettingsApp {
    fn onboarding_ui(&mut self, ui: &mut egui::Ui) {
        let Some(step) = self.model.onboarding else {
            return;
        };

        // Escape cancels/skips tutorial from any step (CAP-OB-4, FR-20)
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.model.skip_onboarding();
            self.finish_onboarding();
            return;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            // Centered Modal Window Shell (Matching prototype.html)
            egui::Frame::new()
                .fill(theme::COLOR_BG_CARD)
                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(24, 20))
                .show(ui, |ui| {
                    ui.set_width(480.0);

                    // Modal Titlebar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Wira Desk — Setup Wizard")
                                .strong()
                                .size(12.5)
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Escape to skip")
                                    .small()
                                    .color(theme::COLOR_TEXT_TERTIARY),
                            );
                        });
                    });
                    ui.add_space(8.0);

                    // Step Progress Indicator Bar (Step 1 of 3, Step 2 of 3, Step 3 of 3)
                    ui.horizontal(|ui| {
                        let total_steps = 3;
                        let current_step_num = match step {
                            app::OnboardingStep::Welcome => 1,
                            app::OnboardingStep::TrySwitching => 2,
                            app::OnboardingStep::Done => 3,
                        };

                        for s in 1..=total_steps {
                            let is_current = s <= current_step_num;
                            let stroke_color = if is_current {
                                theme::COLOR_ACCENT_PRIMARY
                            } else {
                                theme::COLOR_BG_KEYCAP
                            };
                            let width = (ui.available_width() - ((total_steps - s) as f32 * 8.0))
                                / (total_steps - s + 1) as f32;
                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(width.max(20.0), 3.5),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, stroke_color);
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(16.0);

                    // Step Heading & Description
                    ui.label(
                        egui::RichText::new(step.heading())
                            .strong()
                            .size(20.0)
                            .color(theme::COLOR_TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(step.body())
                            .size(13.0)
                            .color(theme::COLOR_TEXT_SECONDARY),
                    );
                    ui.add_space(14.0);

                    // Step-specific Interactive Content
                    match step {
                        app::OnboardingStep::Welcome => {
                            egui::Frame::new()
                                .fill(theme::COLOR_BG_SUBTLE)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::symmetric(14, 12))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("⚡").size(16.0));
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(
                                                        "Same-Application Isolation",
                                                    )
                                                    .strong()
                                                    .size(13.0)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                                );
                                                ui.label(
                                                    egui::RichText::new(
                                                        "Only cycles through windows of the currently active app, leaving other apps untouched.",
                                                    )
                                                    .small()
                                                    .color(theme::COLOR_TEXT_SECONDARY),
                                                );
                                            });
                                        });
                                        ui.add_space(8.0);
                                        ui.separator();
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🔒").size(16.0));
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(
                                                        "Spatial Preservation Lock",
                                                    )
                                                    .strong()
                                                    .size(13.0)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                                );
                                                ui.label(
                                                    egui::RichText::new(
                                                        "Window focus stays strictly on the active physical monitor and Virtual Desktop.",
                                                    )
                                                    .small()
                                                    .color(theme::COLOR_TEXT_SECONDARY),
                                                );
                                            });
                                        });
                                    });
                                });
                        }
                        app::OnboardingStep::TrySwitching => {
                            let simulated_key_triggered = ui.input(|i| {
                                i.key_pressed(egui::Key::Backtick)
                                    || (i.modifiers.command && i.key_pressed(egui::Key::Backtick))
                            });
                            if simulated_key_triggered {
                                self.model.toggle_onboarding_simulation();
                            }

                            egui::Frame::new()
                                .fill(theme::COLOR_BG_SUBTLE)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::symmetric(14, 12))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.vertical_centered(|ui| {
                                        // 2 dummy window mockups
                                        ui.horizontal(|ui| {
                                            let available_w = ui.available_width();
                                            let win_w = (available_w - 12.0) / 2.0;

                                            // Dummy Window 1
                                            let w1_active = self.model.onboarding_focus_index == 0;
                                            let w1_border = if w1_active {
                                                theme::COLOR_ACCENT_PRIMARY
                                            } else {
                                                theme::COLOR_STROKE_CARD
                                            };
                                            let w1_bg = if w1_active {
                                                theme::COLOR_BG_CARD_HOVER
                                            } else {
                                                theme::COLOR_BG_CARD
                                            };

                                            let resp1 = egui::Frame::new()
                                                .fill(w1_bg)
                                                .stroke(egui::Stroke::new(
                                                    if w1_active { 2.0 } else { 1.0 },
                                                    w1_border,
                                                ))
                                                .corner_radius(egui::CornerRadius::same(6))
                                                .inner_margin(egui::Margin::symmetric(10, 8))
                                                .show(ui, |ui| {
                                                    ui.set_min_size(egui::vec2(win_w, 75.0));
                                                    ui.label(
                                                        egui::RichText::new(if w1_active {
                                                            "🪟 [FOCUSED] Project Brief"
                                                        } else {
                                                            "🪟 Project Brief"
                                                        })
                                                        .strong()
                                                        .color(theme::COLOR_TEXT_PRIMARY),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("Chrome — Active Window")
                                                            .small()
                                                            .color(theme::COLOR_TEXT_SECONDARY),
                                                    );
                                                });
                                            resp1.response.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::Other,
                                                    true,
                                                    theme::ONBOARDING_DUMMY_WIN_1.name,
                                                )
                                            });

                                            ui.add_space(8.0);

                                            // Dummy Window 2
                                            let w2_active = self.model.onboarding_focus_index == 1;
                                            let w2_border = if w2_active {
                                                theme::COLOR_ACCENT_PRIMARY
                                            } else {
                                                theme::COLOR_STROKE_CARD
                                            };
                                            let w2_bg = if w2_active {
                                                theme::COLOR_BG_CARD_HOVER
                                            } else {
                                                theme::COLOR_BG_CARD
                                            };

                                            let resp2 = egui::Frame::new()
                                                .fill(w2_bg)
                                                .stroke(egui::Stroke::new(
                                                    if w2_active { 2.0 } else { 1.0 },
                                                    w2_border,
                                                ))
                                                .corner_radius(egui::CornerRadius::same(6))
                                                .inner_margin(egui::Margin::symmetric(10, 8))
                                                .show(ui, |ui| {
                                                    ui.set_min_size(egui::vec2(win_w, 75.0));
                                                    ui.label(
                                                        egui::RichText::new(if w2_active {
                                                            "🪟 [FOCUSED] Design System"
                                                        } else {
                                                            "🪟 Design System"
                                                        })
                                                        .strong()
                                                        .color(theme::COLOR_TEXT_PRIMARY),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("Chrome — Sibling Window")
                                                            .small()
                                                            .color(theme::COLOR_TEXT_SECONDARY),
                                                    );
                                                });
                                            resp2.response.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::Other,
                                                    true,
                                                    theme::ONBOARDING_DUMMY_WIN_2.name,
                                                )
                                            });
                                        });

                                        ui.add_space(10.0);

                                        // Simulated Keypress Trigger Button
                                        let btn_response = ui.add(
                                            egui::Button::new(
                                                egui::RichText::new(
                                                    "👉 Press: Win + ` (or click here)",
                                                )
                                                .strong()
                                                .monospace()
                                                .color(theme::COLOR_TEXT_PRIMARY),
                                            )
                                            .fill(theme::COLOR_BG_KEYCAP)
                                            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                            .corner_radius(egui::CornerRadius::same(6))
                                            .min_size(egui::vec2(220.0, 32.0)),
                                        );
                                        btn_response.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Button,
                                                true,
                                                theme::ONBOARDING_SIMULATE_BUTTON.name,
                                            )
                                        });
                                        if btn_response.clicked() {
                                            self.model.toggle_onboarding_simulation();
                                        }

                                        if self.model.onboarding_simulated_success {
                                            ui.add_space(6.0);
                                            ui.colored_label(
                                                theme::COLOR_SUCCESS,
                                                "✔ Great! Focus shifted instantaneously without HUD latency.",
                                            );
                                        }
                                    });
                                });
                        }
                        app::OnboardingStep::Done => {
                            egui::Frame::new()
                                .fill(theme::COLOR_BG_SUBTLE)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::symmetric(14, 12))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🚀").size(20.0));
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("System Tray Resident")
                                                    .strong()
                                                    .size(13.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    "Right-click the Wira Desk tray icon anytime to open Settings, view logs, or toggle Auto-Start.",
                                                )
                                                .small()
                                                .color(theme::COLOR_TEXT_SECONDARY),
                                            );
                                        });
                                    });
                                });
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Navigation Footer Buttons
                    ui.horizontal(|ui| {
                        if step == app::OnboardingStep::Done {
                            let finish_resp = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Finish & Start Using Wira Desk")
                                        .strong()
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                                )
                                .fill(theme::COLOR_ACCENT_PRIMARY)
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(220.0, 32.0)),
                            );
                            finish_resp.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    theme::ONBOARDING_FINISH_BUTTON.name,
                                )
                            });
                            if finish_resp.clicked() {
                                self.finish_onboarding();
                            }
                        } else {
                            let skip_resp = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Skip Tutorial")
                                        .size(12.5)
                                        .color(theme::COLOR_TEXT_PRIMARY),
                                )
                                .fill(theme::COLOR_BG_KEYCAP)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(100.0, 30.0)),
                            );
                            skip_resp.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    theme::ONBOARDING_SKIP_BUTTON.name,
                                )
                            });
                            if skip_resp.clicked() {
                                self.model.skip_onboarding();
                                self.finish_onboarding();
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let next_resp = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("Next →")
                                                .strong()
                                                .size(12.5)
                                                .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                                        )
                                        .fill(theme::COLOR_ACCENT_PRIMARY)
                                        .corner_radius(egui::CornerRadius::same(6))
                                        .min_size(egui::vec2(90.0, 30.0)),
                                    );
                                    next_resp.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::WidgetType::Button,
                                            true,
                                            theme::ONBOARDING_NEXT_BUTTON.name,
                                        )
                                    });
                                    if next_resp.clicked() {
                                        self.model.advance_onboarding();
                                    }
                                },
                            );
                        }
                    });
                });
        });
    }

    /// Both "finished" and "skipped" write a valid configuration, which is what
    /// stops onboarding repeating on the next launch.
    fn finish_onboarding(&mut self) {
        self.model.save(&config_path());
        self.model.onboarding = None;
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let stops = app::focus_order(self.model.pane);
        let mut drawn: Vec<&'static str> = Vec::new();

        ui.vertical(|ui| {
            // 1. Fluent 2 Modern Header Bar (Custom Frameless Window Titlebar)
            egui::Frame::new()
                .fill(theme::COLOR_BG_MICA)
                .inner_margin(egui::Margin::symmetric(16, 10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🪟 Wira Desk — Settings")
                                .strong()
                                .size(13.5)
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(13.0)
                                        .color(theme::COLOR_TEXT_SECONDARY),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(28.0, 24.0)),
                            );
                            if close_btn.clicked() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }

                            let min_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("—")
                                        .size(12.0)
                                        .color(theme::COLOR_TEXT_SECONDARY),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(28.0, 24.0)),
                            );
                            if min_btn.clicked() {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        });
                    });
                });

            // 2. Middle Body: Left Sidebar + Right Content Area
            let footer_height = 54.0;
            let body_height = (ui.available_height() - footer_height).max(280.0);

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), body_height),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    // Left Vertical Sidebar Frame
                    egui::Frame::new()
                        .fill(theme::COLOR_BG_SIDEBAR)
                        .inner_margin(egui::Margin::symmetric(8, 12))
                        .show(ui, |ui| {
                            ui.set_width(175.0);
                            ui.set_min_height(ui.available_height());
                            ui.vertical(|ui| {
                                for pane in stops.iter().filter_map(|l| Pane::from_label(l)) {
                                    drawn.push(pane.label());
                                    let selected = self.model.pane == pane;

                                    let icon_text = match pane {
                                        Pane::General => "⚙  General",
                                        Pane::Shortcuts => "⌨  Shortcuts",
                                        Pane::Layout => "🗂  Layout & Snapping",
                                        Pane::VmExceptions => "🖥  VM & Exceptions",
                                        Pane::About => "ℹ  About",
                                    };

                                    let desired_size = egui::vec2(165.0, 36.0);
                                    let (rect, response) =
                                        ui.allocate_exact_size(desired_size, egui::Sense::click());

                                    if response.clicked() {
                                        self.model.set_pane(pane);
                                    }

                                    if ui.is_rect_visible(rect) {
                                        let fill_color = if selected {
                                            theme::COLOR_BG_CARD
                                        } else if response.hovered() {
                                            theme::COLOR_BG_CARD_HOVER
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };

                                        ui.painter().rect_filled(
                                            rect,
                                            egui::CornerRadius::same(6),
                                            fill_color,
                                        );

                                        if selected {
                                            // Blue vertical active indicator pill on left edge
                                            let indicator_rect = egui::Rect::from_min_size(
                                                egui::pos2(rect.left() + 2.0, rect.top() + 8.0),
                                                egui::vec2(3.5, 20.0),
                                            );
                                            ui.painter().rect_filled(
                                                indicator_rect,
                                                egui::CornerRadius::same(2),
                                                theme::COLOR_ACCENT_PRIMARY,
                                            );
                                        }

                                        let text_color = if selected {
                                            theme::COLOR_TEXT_PRIMARY
                                        } else {
                                            theme::COLOR_TEXT_SECONDARY
                                        };

                                        let text_pos =
                                            egui::pos2(rect.left() + 14.0, rect.center().y - 7.0);
                                        ui.painter().text(
                                            text_pos,
                                            egui::Align2::LEFT_TOP,
                                            icon_text,
                                            egui::FontId::proportional(13.0),
                                            text_color,
                                        );
                                    }

                                    ui.add_space(3.0);
                                }
                            });
                        });

                    ui.add_space(14.0);

                    // Right Main Content Panel
                    egui::Frame::new()
                        .fill(theme::COLOR_BG_MICA)
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height(ui.available_height());
                            ui.vertical(|ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| match self.model.pane {
                                    Pane::General => self.general_pane(ui, &mut drawn),
                                    Pane::Shortcuts => self.shortcuts_pane(ui, &stops, &mut drawn),
                                    Pane::Layout => self.layout_pane(ui, &mut drawn),
                                    Pane::VmExceptions => self.vm_exceptions_pane(ui, &mut drawn),
                                    Pane::About => self.about_pane(ui),
                                });
                            });
                        });
                },
            );

            // 3. Fixed Footer Status & Action Bar
            egui::Frame::new()
                .fill(theme::COLOR_BG_CARD)
                .inner_margin(egui::Margin::symmetric(16, 10))
                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        // Left daemon status dot
                        ui.painter().circle_filled(
                            egui::pos2(ui.cursor().min.x + 4.0, ui.cursor().center().y),
                            4.0,
                            theme::COLOR_SUCCESS,
                        );
                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new("Daemon running elevated (Active)")
                                .small()
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );

                        // Right action buttons (Revert & Save Changes)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            drawn.push("Save");
                            drawn.push("Revert");

                            let save_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Save Changes")
                                        .strong()
                                        .size(12.5)
                                        .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                                )
                                .fill(theme::COLOR_ACCENT_PRIMARY)
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(110.0, 30.0)),
                            );
                            if save_btn.clicked() {
                                self.model.save(&config_path());
                            }

                            let revert_btn = ui.add_enabled(
                                self.model.is_dirty(),
                                egui::Button::new(
                                    egui::RichText::new("Revert")
                                        .size(12.5)
                                        .color(theme::COLOR_TEXT_PRIMARY),
                                )
                                .fill(theme::COLOR_BG_KEYCAP)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(80.0, 30.0)),
                            );
                            if revert_btn.clicked() {
                                self.model.revert();
                            }

                            match &self.model.feedback {
                                SaveFeedback::None => {}
                                SaveFeedback::Saved { reload_signalled } => {
                                    let msg = if *reload_signalled {
                                        "Settings saved and applied."
                                    } else {
                                        "Settings saved. Applies on next launch."
                                    };
                                    ui.colored_label(theme::COLOR_SUCCESS, msg);
                                }
                                SaveFeedback::Error(msg) => {
                                    ui.colored_label(ui.visuals().error_fg_color, msg);
                                }
                            }
                        });
                    });
                });

            #[cfg(debug_assertions)]
            if let Some(mismatch) = app::focus_order_mismatch(self.model.pane, &drawn) {
                ui.add_space(4.0);
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("focus order drift: {mismatch}"),
                );
            }
        });
    }

    fn general_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        let c = theme::TOGGLE_AUTO_START;
        drawn.push(c.name);

        ui.heading("General Settings");
        ui.label(
            egui::RichText::new(
                "Daemon status, OS startup integration, and spatial isolation integrity.",
            )
            .small()
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(12.0);

        // Group Card Container (Matching prototype.html with 4 settings rows)
        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Row 1: Auto-start
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Auto-start on Boot")
                                .strong()
                                .size(13.0)
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(c.description)
                                .small()
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        fluent_toggle_switch(ui, &mut self.model.draft.general.auto_start, c.name);
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Row 2: Spatial Preservation (Per-Monitor Lock)
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Spatial Preservation (Per-Monitor Lock)")
                                .strong()
                                .size(13.0)
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(
                                "Locks window cycling strictly to the currently active physical monitor.",
                            )
                            .small()
                            .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut locked = true;
                        let resp =
                            fluent_toggle_switch(ui, &mut locked, "Spatial Preservation Lock");
                        resp.on_hover_text(
                            "Spatial lock is an architectural guarantee in Wira Desk.",
                        );
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Row 3: Virtual Desktop Isolation
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Virtual Desktop Isolation")
                                .strong()
                                .size(13.0)
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(
                                "Prevents jumping across active Windows Virtual Desktop boundaries.",
                            )
                            .small()
                            .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut isolated = true;
                        let resp =
                            fluent_toggle_switch(ui, &mut isolated, "Virtual Desktop Isolation");
                        resp.on_hover_text("Virtual desktop isolation is enabled by default.");
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Row 4: UX Honesty Mode
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("UX Honesty Mode")
                                .strong()
                                .size(13.0)
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(
                                "Brings hanging or not-responding windows forward transparently.",
                            )
                            .small()
                            .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut honesty = true;
                        let resp = fluent_toggle_switch(ui, &mut honesty, "UX Honesty Mode");
                        resp.on_hover_text(
                            "Ensures hanging windows are exposed cleanly to the user.",
                        );
                    });
                });
            });
    }

    fn shortcuts_pane(
        &mut self,
        ui: &mut egui::Ui,
        stops: &[&'static str],
        drawn: &mut Vec<&'static str>,
    ) {
        ui.heading("Shortcuts Configuration");
        ui.label(
            egui::RichText::new(
                "Click any shortcut button to record physical key combinations directly.",
            )
            .small()
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                for (idx, field) in stops
                    .iter()
                    .filter_map(|l| ShortcutField::from_label(l))
                    .enumerate()
                {
                    if idx > 0 {
                        ui.separator();
                    }

                    drawn.push(field.label());
                    let current = field.get(&self.model.draft).to_string();
                    let listening = self.model.capture.is_listening_for(field);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(field.label()).strong().size(13.0));
                            let subtext = match field {
                                ShortcutField::Switcher => {
                                    "Rotates through windows of the active application (per-monitor)."
                                }
                                ShortcutField::Fallback => {
                                    "Fallback rotation chord when main key is intercepted."
                                }
                                ShortcutField::SnapLeft => "DPI-aware left 50% split placement.",
                                ShortcutField::SnapRight => "DPI-aware right 50% split placement.",
                                ShortcutField::SnapMaximize => {
                                    "Maximizes window across active work area."
                                }
                                ShortcutField::Stack => {
                                    "Arranges same-app windows in clickable overlapping stack."
                                }
                            };
                            ui.label(
                                egui::RichText::new(subtext)
                                    .small()
                                    .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let text = if listening {
                                "🔴 Listening…".to_string()
                            } else {
                                current.clone()
                            };

                            let btn_color = if listening {
                                ui.visuals().selection.stroke.color
                            } else {
                                theme::COLOR_BG_KEYCAP
                            };

                            let response = ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(text).strong().size(12.0).color(
                                            if listening {
                                                egui::Color32::WHITE
                                            } else {
                                                theme::COLOR_TEXT_PRIMARY
                                            },
                                        ),
                                    )
                                    .fill(btn_color)
                                    .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .min_size(egui::vec2(120.0, 28.0)),
                                )
                                .on_hover_text(theme::SHORTCUT_SWITCHER.description);

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
                    });
                }
            });

        if let app::CaptureState::Listening(field) = self.model.capture.clone() {
            ui.add_space(8.0);
            ui.colored_label(theme::COLOR_ACCENT_PRIMARY, theme::LISTENING_ANNOUNCEMENT);
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.model.cancel_capture();
            } else if let Some(combo) = captured_combination(ui.ctx()) {
                if let Err(err) = self.model.accept_capture(&combo) {
                    self.model.feedback = SaveFeedback::Error(app::describe(field.label(), err));
                }
            }
        }
    }

    fn layout_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        let c = theme::TOGGLE_OVERLAPPING_STACK;
        drawn.push(c.name);
        drawn.push(theme::STACK_WIDTH_SLIDER.name);
        drawn.push(theme::STACK_WIDTH_INPUT.name);

        ui.heading("Layout & Snapping");
        ui.label(
            egui::RichText::new(
                "DPI-aware window snapping and compact overlapping stack arrangement.",
            )
            .small()
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Row 1: Toggle Stack
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(c.name).strong().size(13.0));
                        ui.label(
                            egui::RichText::new(c.description)
                                .small()
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        fluent_toggle_switch(
                            ui,
                            &mut self.model.draft.layout.enable_overlapping_stack,
                            c.name,
                        );
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Row 2: Stack Width Slider
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Stack Width Ratio").strong().size(13.0));
                        ui.label(
                            egui::RichText::new(
                                "Percentage of screen width allocated to stacked windows.",
                            )
                            .small()
                            .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut self.model.draft.layout.stack_width_percent,
                                10..=100,
                            )
                            .text(theme::STACK_WIDTH_SLIDER.name)
                            .suffix("%"),
                        );
                    });
                });
            });
    }

    fn vm_exceptions_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        drawn.push(theme::VM_BYPASS_PROCESS_LIST.name);
        drawn.push(theme::VM_BYPASS_CLASS_LIST.name);

        ui.heading("VM & Remote Desktop Exceptions");
        ui.label(
            egui::RichText::new(
                "Applications exempted so keyboard shortcuts pass directly to the virtual guest.",
            )
            .small()
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.label(
                    egui::RichText::new("Bypass Executables:")
                        .strong()
                        .size(13.0),
                );
                ui.add_space(4.0);
                for proc in &self.model.draft.vm_bypass.bypass_processes {
                    ui.label(
                        egui::RichText::new(format!("  •  {proc}"))
                            .color(theme::COLOR_TEXT_SECONDARY),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("Bypass Window Classes:")
                        .strong()
                        .size(13.0),
                );
                ui.add_space(4.0);
                for class in &self.model.draft.vm_bypass.bypass_classes {
                    ui.label(
                        egui::RichText::new(format!("  •  {class}"))
                            .color(theme::COLOR_TEXT_SECONDARY),
                    );
                }
            });
    }

    fn about_pane(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wira Desk (WiraDex)");
        ui.label(
            egui::RichText::new(concat!("Version ", env!("CARGO_PKG_VERSION")))
                .color(theme::COLOR_ACCENT_PRIMARY)
                .strong(),
        );
        ui.label(match &self.font {
            theme::LoadedFont::System(name) => format!("Typeface: {name}"),
            theme::LoadedFont::Bundled => "Typeface: Bundled Fallback".to_string(),
        });
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    egui::RichText::new(
                        "Wira Desk switches between windows of the application you are already using, rather than every window on the system.",
                    )
                    .color(theme::COLOR_TEXT_SECONDARY),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Decoupled Architecture: The background daemon runs with zero UI overhead, while this configuration shell opens only on demand.",
                    )
                    .small()
                    .color(theme::COLOR_TEXT_TERTIARY),
                );
            });
    }
}

/// Render a Fluent 2 pill-shaped toggle switch.
fn fluent_toggle_switch(ui: &mut egui::Ui, on: &mut bool, accessible_name: &str) -> egui::Response {
    let desired_size = egui::vec2(40.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *on, accessible_name)
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.visuals();

        let bg_color = if *on {
            theme::COLOR_ACCENT_PRIMARY
        } else if visuals.dark_mode {
            egui::Color32::from_rgb(0x28, 0x2a, 0x30)
        } else {
            egui::Color32::from_rgb(0xe0, 0xe4, 0xec)
        };

        let stroke_color = if *on {
            theme::COLOR_ACCENT_PRIMARY
        } else if visuals.dark_mode {
            egui::Color32::from_rgba_premultiplied(255, 255, 255, 50)
        } else {
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 50)
        };

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(10),
            bg_color,
            egui::Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Outside,
        );

        let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        let circle_color = if *on {
            egui::Color32::WHITE
        } else if visuals.dark_mode {
            egui::Color32::from_rgb(0xa0, 0xa6, 0xb4)
        } else {
            egui::Color32::from_rgb(0x5c, 0x63, 0x70)
        };

        ui.painter().circle_filled(center, 6.0, circle_color);
    }

    response
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
        if m.command {
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
    /// Consumes the build script's report, so a path through `build.rs` that returns
    /// without embedding the resource *and* without saying so stops this crate from
    /// compiling. Unlike the daemon there is no skip switch here, so on Windows the only
    /// honest outcome is `embedded` — anything else means the resource did not compile.
    #[test]
    fn build_script_reports_what_it_did_with_the_resource() {
        const STATE: &str = env!("WIRADESK_SETTINGS_RESOURCE_STATE");
        assert!(
            matches!(STATE, "embedded" | "not-windows"),
            "unrecognised resource state {STATE:?} - build.rs reported something this test \
             does not know how to interpret, which means one of them is out of date"
        );

        #[cfg(windows)]
        assert_eq!(
            STATE, "embedded",
            "the Settings resource was not embedded. There is no opt-out for this crate, so \
             the only way to reach this is a resource compilation that did not happen - which \
             ships a binary with no icon and no version metadata"
        );
    }

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
