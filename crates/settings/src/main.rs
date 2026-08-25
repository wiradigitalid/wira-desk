#![windows_subsystem = "windows"]

mod app;
mod logo_data;
mod persistence;
mod theme;

use eframe::egui;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, FALSE};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

use shared::constants::SETTINGS_SINGLE_INSTANCE_MUTEX;
use shared::{config_path, migrate_appdata, Config};

use app::{Pane, SaveFeedback, SettingsModel, ShortcutField};
use persistence::{resolve_launch_intent, LaunchIntent};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn format_shortcut_display(raw: &str) -> String {
    if raw.is_empty() {
        return "None".to_string();
    }
    raw.split('+')
        .map(|token| {
            let lower = token.to_lowercase();
            match lower.trim() {
                "win" => "Win".to_string(),
                "ctrl" => "Ctrl".to_string(),
                "alt" => "Alt".to_string(),
                "shift" => "Shift".to_string(),
                "backtick" => "`".to_string(),
                "enter" => "Enter".to_string(),
                "tab" => "Tab".to_string(),
                "space" => "Space".to_string(),
                "escape" => "Esc".to_string(),
                "left" => "←".to_string(),
                "right" => "→".to_string(),
                "up" => "↑".to_string(),
                "down" => "↓".to_string(),
                "f1" => "F1".to_string(),
                "f2" => "F2".to_string(),
                "f3" => "F3".to_string(),
                "f4" => "F4".to_string(),
                "f5" => "F5".to_string(),
                "f6" => "F6".to_string(),
                "f7" => "F7".to_string(),
                "f8" => "F8".to_string(),
                "f9" => "F9".to_string(),
                "f10" => "F10".to_string(),
                "f11" => "F11".to_string(),
                "f12" => "F12".to_string(),
                _ => token.trim().to_uppercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn main() -> eframe::Result {
    migrate_appdata();

    // Enforce single-instance for settings executable
    let mutex_name = wide(SETTINGS_SINGLE_INSTANCE_MUTEX);
    // SAFETY: `mutex_name` is NUL-terminated wide string that outlives the call.
    let mutex = unsafe { CreateMutexW(std::ptr::null(), FALSE, mutex_name.as_ptr()) };
    // SAFETY: `GetLastError` takes no parameters and reads thread-local error state from the previous Win32 call.
    if mutex == 0 || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // Bring existing Settings window to foreground if present
        let title = wide("Wira Desk");
        // SAFETY: `title` is a NUL-terminated wide string and handles are validated before use.
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

    let intent = resolve_launch_intent(std::env::args());
    let saved = Config::load_or_default(&config_path());

    let (width, height) = if intent == LaunchIntent::Onboarding {
        (580.0, 380.0)
    } else {
        (680.0, 590.0)
    };

    let icon_data = egui::IconData {
        rgba: include_bytes!("logo_64.rgba").to_vec(),
        width: 64,
        height: 64,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([540.0, 400.0])
            .with_title("Wira Desk")
            .with_icon(icon_data)
            .with_decorations(false)
            .with_transparent(false),
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
            let frame_bg = if self.model.theme == theme::ThemeMode::Dark {
                theme::COLOR_BG_MICA
            } else {
                egui::Color32::from_rgb(0xf3, 0xf3, 0xf3)
            };
            egui::Frame::new().fill(frame_bg).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                self.settings_ui(ui);
            });
        }
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
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Full window top drag area: covers from true window (0,0) down through 48px
        let win_rect = ui.max_rect();
        let top_drag_rect =
            egui::Rect::from_min_size(win_rect.min, egui::vec2(win_rect.width(), 48.0));
        let drag_response = ui.interact(
            top_drag_rect,
            ui.id().with("onboarding_top_window_drag"),
            egui::Sense::drag(),
        );
        if drag_response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Full modal surface with single crisp Fluent 2 border
        egui::Frame::new()
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin {
                left: 28,
                right: 28,
                top: 18,
                bottom: 24,
            })
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());

                // 1. Progress Indicator Bar (3 horizontal segments spanning 100% available width)
                let total_steps = 3;
                let current_step_num = match step {
                    app::OnboardingStep::Welcome => 1,
                    app::OnboardingStep::TrySwitching => 2,
                    app::OnboardingStep::Done => 3,
                };

                let total_w = ui.available_width();
                let gap = 10.0;
                let seg_w = (total_w - ((total_steps - 1) as f32 * gap)) / total_steps as f32;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for s in 1..=total_steps {
                        let is_current = s <= current_step_num;
                        let stroke_color = if is_current {
                            theme::COLOR_ACCENT_PRIMARY
                        } else {
                            theme::COLOR_BG_KEYCAP
                        };
                        let (rect, _response) = ui.allocate_exact_size(
                            egui::vec2(seg_w, 4.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, stroke_color);
                    }
                });

                ui.add_space(16.0);

                // 2. Step Heading (Left-aligned, Bold 20pt)
                ui.label(
                    egui::RichText::new(step.heading())
                        .strong()
                        .size(20.0)
                        .color(theme::COLOR_TEXT_PRIMARY),
                );
                ui.add_space(6.0);

                // 3. Step Description (Secondary text, 13.5pt, clean line height)
                ui.label(
                    egui::RichText::new(step.body())
                        .size(13.5)
                        .color(theme::COLOR_TEXT_SECONDARY),
                );

                ui.add_space(14.0);

                // 4. Middle Content Cards per Step (Natural frame filling without overflow)
                match step {
                    app::OnboardingStep::Welcome => {
                        egui::Frame::new()
                            .fill(theme::COLOR_BG_SUBTLE)
                            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::symmetric(16, 12))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("⚡").size(18.0));
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("Same-Application Spatial Cycling")
                                                    .strong()
                                                    .size(13.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    "Press Win + ` to cycle focus between windows of the active app with zero visual delay.",
                                                )
                                                .size(12.0)
                                                .color(theme::COLOR_TEXT_SECONDARY),
                                            );
                                        });
                                    });

                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🔒").size(18.0));
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("Multi-Monitor & Desktop Isolation")
                                                    .strong()
                                                    .size(13.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    "Window switching stays locked strictly to your active physical display and desktop.",
                                                )
                                                .size(12.0)
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
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::symmetric(14, 12))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    let gap = 12.0;
                                    let win_w = (ui.available_width() - gap) / 2.0;

                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = gap;

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
                                                if w1_active { 1.5 } else { 1.0 },
                                                w1_border,
                                            ))
                                            .corner_radius(egui::CornerRadius::same(8))
                                            .inner_margin(egui::Margin::ZERO)
                                            .show(ui, |ui| {
                                                ui.set_width(win_w);
                                                ui.set_height(72.0);
                                                ui.vertical(|ui| {
                                                    egui::Frame::new()
                                                        .fill(theme::COLOR_BG_KEYCAP)
                                                        .inner_margin(egui::Margin::symmetric(10, 4))
                                                        .show(ui, |ui| {
                                                            ui.set_width(ui.available_width());
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("Document 1")
                                                                        .size(11.0)
                                                                        .strong()
                                                                        .color(if w1_active {
                                                                            theme::COLOR_ACCENT_PRIMARY
                                                                        } else {
                                                                            theme::COLOR_TEXT_SECONDARY
                                                                        }),
                                                                );
                                                                ui.with_layout(
                                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                                    |ui| {
                                                                        ui.label(
                                                                            egui::RichText::new("✕")
                                                                                .size(9.5)
                                                                                .color(theme::COLOR_TEXT_TERTIARY),
                                                                        );
                                                                    },
                                                                );
                                                            });
                                                        });

                                                    ui.add_space(4.0);
                                                    ui.horizontal(|ui| {
                                                        ui.add_space(8.0);
                                                        ui.vertical(|ui| {
                                                            ui.label(
                                                                egui::RichText::new("Project Brief.docx")
                                                                    .size(11.5)
                                                                    .strong()
                                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(if w1_active {
                                                                    "Active Window"
                                                                } else {
                                                                    "Background"
                                                                })
                                                                .size(10.5)
                                                                .color(theme::COLOR_TEXT_SECONDARY),
                                                            );
                                                        });
                                                    });
                                                });
                                            });
                                        resp1.response.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Other,
                                                true,
                                                theme::ONBOARDING_DUMMY_WIN_1.name,
                                            )
                                        });

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
                                                if w2_active { 1.5 } else { 1.0 },
                                                w2_border,
                                            ))
                                            .corner_radius(egui::CornerRadius::same(8))
                                            .inner_margin(egui::Margin::ZERO)
                                            .show(ui, |ui| {
                                                ui.set_width(win_w);
                                                ui.set_height(72.0);
                                                ui.vertical(|ui| {
                                                    egui::Frame::new()
                                                        .fill(theme::COLOR_BG_KEYCAP)
                                                        .inner_margin(egui::Margin::symmetric(10, 4))
                                                        .show(ui, |ui| {
                                                            ui.set_width(ui.available_width());
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new("Document 2")
                                                                        .size(11.0)
                                                                        .strong()
                                                                        .color(if w2_active {
                                                                            theme::COLOR_ACCENT_PRIMARY
                                                                        } else {
                                                                            theme::COLOR_TEXT_SECONDARY
                                                                        }),
                                                                );
                                                                ui.with_layout(
                                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                                    |ui| {
                                                                        ui.label(
                                                                            egui::RichText::new("✕")
                                                                                .size(9.5)
                                                                                .color(theme::COLOR_TEXT_TERTIARY),
                                                                        );
                                                                    },
                                                                );
                                                            });
                                                        });

                                                    ui.add_space(4.0);
                                                    ui.horizontal(|ui| {
                                                        ui.add_space(8.0);
                                                        ui.vertical(|ui| {
                                                            ui.label(
                                                                egui::RichText::new("Design System.docx")
                                                                    .size(11.5)
                                                                    .strong()
                                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(if w2_active {
                                                                    "Active Window"
                                                                } else {
                                                                    "Background"
                                                                })
                                                                .size(10.5)
                                                                .color(theme::COLOR_TEXT_SECONDARY),
                                                            );
                                                        });
                                                    });
                                                });
                                            });
                                        resp2.response.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Other,
                                                true,
                                                theme::ONBOARDING_DUMMY_WIN_2.name,
                                            )
                                        });
                                    });

                                    ui.add_space(8.0);

                                    // Practice trigger button & feedback
                                    ui.horizontal(|ui| {
                                        let sim_btn = ui.add(
                                            egui::Button::new(
                                                egui::RichText::new("Win + ` (Click to simulate)")
                                                    .monospace()
                                                    .strong()
                                                    .size(11.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            )
                                            .fill(theme::COLOR_BG_KEYCAP)
                                            .corner_radius(egui::CornerRadius::same(6)),
                                        );
                                        sim_btn.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Button,
                                                true,
                                                theme::ONBOARDING_SIMULATE_BUTTON.name,
                                            )
                                        });
                                        if sim_btn.clicked() {
                                            self.model.toggle_onboarding_simulation();
                                        }

                                        if self.model.onboarding_focus_index == 1 {
                                            ui.add_space(8.0);
                                            ui.colored_label(
                                                theme::COLOR_SUCCESS,
                                                "✔ Focus shifted to Document 2",
                                            );
                                        }
                                    });
                                });
                            });
                    }
                    app::OnboardingStep::Done => {
                        egui::Frame::new()
                            .fill(theme::COLOR_BG_SUBTLE)
                            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::symmetric(16, 12))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🚀").size(20.0));
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("System Tray Resident")
                                                    .strong()
                                                    .size(13.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    "Wira Desk runs quietly in the background. Right-click the tray icon anytime for Settings.",
                                                )
                                                .size(12.0)
                                                .color(theme::COLOR_TEXT_SECONDARY),
                                            );
                                        });
                                    });

                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("⌨").size(18.0));
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("Fully Customizable")
                                                    .strong()
                                                    .size(13.5)
                                                    .color(theme::COLOR_TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    "Customize shortcuts, window snapping parameters, and VM passthrough rules anytime.",
                                                )
                                                .size(12.0)
                                                .color(theme::COLOR_TEXT_SECONDARY),
                                            );
                                        });
                                    });
                                });
                            });
                    }
                }

                // 5. 100% MATHEMATICALLY PINNED FOOTER BUTTONS AT EXACT COORDINATE
                let btn_height = 34.0;
                let footer_y = ui.max_rect().bottom() - btn_height;

                match step {
                    app::OnboardingStep::Welcome => {
                        let left_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().left(), footer_y),
                            egui::vec2(105.0, btn_height),
                        );
                        let skip_resp = ui.put(
                            left_rect,
                            egui::Button::new(
                                egui::RichText::new("Skip Tutorial")
                                    .strong()
                                    .size(12.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            )
                            .fill(theme::COLOR_BG_CARD_HOVER)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(6)),
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
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        let right_w = 90.0;
                        let right_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().right() - right_w, footer_y),
                            egui::vec2(right_w, btn_height),
                        );
                        let next_resp = ui.put(
                            right_rect,
                            egui::Button::new(
                                egui::RichText::new("Next")
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                            )
                            .fill(theme::COLOR_ACCENT_PRIMARY)
                            .corner_radius(egui::CornerRadius::same(6)),
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
                    }
                    app::OnboardingStep::TrySwitching => {
                        let left_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().left(), footer_y),
                            egui::vec2(90.0, btn_height),
                        );
                        let back_resp = ui.put(
                            left_rect,
                            egui::Button::new(
                                egui::RichText::new("← Back")
                                    .strong()
                                    .size(12.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            )
                            .fill(theme::COLOR_BG_CARD_HOVER)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(6)),
                        );
                        back_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                theme::ONBOARDING_BACK_BUTTON.name,
                            )
                        });
                        if back_resp.clicked() {
                            self.model.onboarding = Some(app::OnboardingStep::Welcome);
                        }

                        let right_w = 90.0;
                        let right_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().right() - right_w, footer_y),
                            egui::vec2(right_w, btn_height),
                        );
                        let next_resp = ui.put(
                            right_rect,
                            egui::Button::new(
                                egui::RichText::new("Next")
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                            )
                            .fill(theme::COLOR_ACCENT_PRIMARY)
                            .corner_radius(egui::CornerRadius::same(6)),
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
                    }
                    app::OnboardingStep::Done => {
                        let left_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().left(), footer_y),
                            egui::vec2(90.0, btn_height),
                        );
                        let back_resp = ui.put(
                            left_rect,
                            egui::Button::new(
                                egui::RichText::new("← Back")
                                    .strong()
                                    .size(12.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            )
                            .fill(theme::COLOR_BG_CARD_HOVER)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(6)),
                        );
                        back_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                theme::ONBOARDING_BACK_BUTTON.name,
                            )
                        });
                        if back_resp.clicked() {
                            self.model.onboarding = Some(app::OnboardingStep::TrySwitching);
                        }

                        let right_w = 170.0;
                        let right_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().right() - right_w, footer_y),
                            egui::vec2(right_w, btn_height),
                        );
                        let finish_resp = ui.put(
                            right_rect,
                            egui::Button::new(
                                egui::RichText::new("Start Using Wira Desk")
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                            )
                            .fill(theme::COLOR_ACCENT_PRIMARY)
                            .corner_radius(egui::CornerRadius::same(6)),
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
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
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
        let stops = app::focus_order(self.model.pane);
        let mut drawn: Vec<&'static str> = Vec::new();

        ui.vertical(|ui| {
            // 1. Fluent 2 Modern Header Bar (Custom Frameless Window Titlebar)
            let win_width = ui.available_width();
            let header_bar_rect =
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(win_width, 36.0));

            // Drag handler covers the non-button area of the titlebar
            let drag_rect = egui::Rect::from_min_max(
                header_bar_rect.min,
                egui::pos2(header_bar_rect.right() - 88.0, header_bar_rect.bottom()),
            );
            let drag_resp = ui.interact(
                drag_rect,
                ui.id().with("settings_titlebar_drag"),
                egui::Sense::drag(),
            );
            if drag_resp.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            egui::Frame::new()
                .fill(theme::COLOR_BG_MICA)
                .inner_margin(egui::Margin {
                    left: 14,
                    right: 0,
                    top: 0,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(36.0);
                    ui.horizontal_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            // Master Logo Pixel Rendering (18x18 Full-size)
                            let (icon_rect, _) = ui
                                .allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                            if ui.is_rect_visible(icon_rect) {
                                for y in 0..18 {
                                    for x in 0..18 {
                                        let idx = y * 18 + x;
                                        let [r, g, b, a] = logo_data::APP_LOGO_18_RGBA[idx];
                                        if a > 0 {
                                            let pixel_pos = egui::pos2(
                                                icon_rect.left() + x as f32,
                                                icon_rect.top() + y as f32,
                                            );
                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_size(
                                                    pixel_pos,
                                                    egui::vec2(1.0, 1.0),
                                                ),
                                                egui::CornerRadius::ZERO,
                                                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                                            );
                                        }
                                    }
                                }
                            }
                            ui.label(
                                egui::RichText::new("Wira Desk — Settings")
                                    .strong()
                                    .size(13.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let btn_size = egui::vec2(46.0, 36.0);

                            // Close Button (Full Titlebar Height)
                            let (close_rect, close_resp) =
                                ui.allocate_exact_size(btn_size, egui::Sense::click());
                            if close_resp.clicked() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.is_rect_visible(close_rect) {
                                let (fill, glyph_col) = if close_resp.hovered() {
                                    (
                                        egui::Color32::from_rgb(0xc4, 0x2b, 0x1c),
                                        egui::Color32::WHITE,
                                    )
                                } else {
                                    (egui::Color32::TRANSPARENT, theme::COLOR_TEXT_PRIMARY)
                                };
                                if fill != egui::Color32::TRANSPARENT {
                                    ui.painter().rect_filled(
                                        close_rect,
                                        egui::CornerRadius::ZERO,
                                        fill,
                                    );
                                }
                                let center = close_rect.center();
                                let d = 5.0;
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - d, center.y - d),
                                        egui::pos2(center.x + d, center.y + d),
                                    ],
                                    egui::Stroke::new(1.2, glyph_col),
                                );
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x + d, center.y - d),
                                        egui::pos2(center.x - d, center.y + d),
                                    ],
                                    egui::Stroke::new(1.2, glyph_col),
                                );
                            }

                            // Minimize Button (Full Titlebar Height)
                            let (min_rect, min_resp) =
                                ui.allocate_exact_size(btn_size, egui::Sense::click());
                            if min_resp.clicked() {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            if ui.is_rect_visible(min_rect) {
                                let (fill, glyph_col) = if min_resp.hovered() {
                                    (theme::COLOR_BG_CARD_HOVER, theme::COLOR_TEXT_PRIMARY)
                                } else {
                                    (egui::Color32::TRANSPARENT, theme::COLOR_TEXT_PRIMARY)
                                };
                                if fill != egui::Color32::TRANSPARENT {
                                    ui.painter().rect_filled(
                                        min_rect,
                                        egui::CornerRadius::ZERO,
                                        fill,
                                    );
                                }
                                let center = min_rect.center();
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - 5.0, center.y),
                                        egui::pos2(center.x + 5.0, center.y),
                                    ],
                                    egui::Stroke::new(1.2, glyph_col),
                                );
                            }
                        });
                    });
                });

            // 2. Middle Body: Left Sidebar + Right Content Area
            let footer_height = 56.0;
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

                                    let (kind, label_text) = match pane {
                                        Pane::General => (0, "General"),
                                        Pane::Shortcuts => (1, "Shortcuts"),
                                        Pane::Layout => (2, "Layout & Snapping"),
                                        Pane::VmExceptions => (3, "VM & Exceptions"),
                                        Pane::About => (4, "About"),
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

                                        let icon_color = if selected {
                                            theme::COLOR_ACCENT_PRIMARY
                                        } else {
                                            theme::COLOR_TEXT_SECONDARY
                                        };

                                        let text_color = if selected {
                                            theme::COLOR_TEXT_PRIMARY
                                        } else {
                                            theme::COLOR_TEXT_SECONDARY
                                        };

                                        // Draw crisp vector icon for each sidebar navigation tab
                                        let icon_box = egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 14.0, rect.center().y - 7.0),
                                            egui::vec2(14.0, 14.0),
                                        );
                                        draw_sidebar_icon(ui, kind, icon_box, icon_color);

                                        let text_pos =
                                            egui::pos2(rect.left() + 36.0, rect.center().y - 7.5);
                                        ui.painter().text(
                                            text_pos,
                                            egui::Align2::LEFT_TOP,
                                            label_text,
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

            // 3. Fixed Footer Status & Action Bar (Clean Single-Line Center Aligned)
            egui::Frame::new()
                .fill(theme::COLOR_BG_CARD)
                .inner_margin(egui::Margin::symmetric(16, 10))
                .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_centered(|ui| {
                        // Left status indicator + text
                        ui.horizontal(|ui| {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            let is_error = matches!(self.model.feedback, SaveFeedback::Error(_));
                            let dot_color = if is_error {
                                ui.visuals().error_fg_color
                            } else {
                                theme::COLOR_SUCCESS
                            };

                            ui.painter()
                                .circle_filled(dot_rect.center(), 4.0, dot_color);
                            ui.add_space(6.0);

                            let (status_text, text_color) = match &self.model.feedback {
                                SaveFeedback::None => {
                                    if self.model.has_any_conflict() {
                                        let msg = if self.model.any_swappable_conflict() {
                                            "⚠️ Shortcut conflict detected. Resolve with Swap ⇄ or a different key."
                                        } else {
                                            "⚠️ Shortcut conflict detected. Give one action a different key."
                                        };
                                        (msg, theme::COLOR_WARNING)
                                    } else {
                                        ("Wira Desk is Active", theme::COLOR_TEXT_PRIMARY)
                                    }
                                }
                                SaveFeedback::Saved { reload_signalled } => {
                                    let msg = if *reload_signalled {
                                        "Settings saved and applied"
                                    } else {
                                        "Settings saved for next launch"
                                    };
                                    (msg, theme::COLOR_SUCCESS)
                                }
                                SaveFeedback::Error(msg) => {
                                    (msg.as_str(), ui.visuals().error_fg_color)
                                }
                            };

                            ui.label(
                                egui::RichText::new(status_text)
                                    .strong()
                                    .size(12.5)
                                    .color(text_color),
                            );
                        });

                        // Right action buttons (Revert & Save Changes)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            drawn.push("Save");
                            drawn.push("Revert");

                            // Save is never gated on a standing conflict (DEC-001 /
                            // LBR-ST-8): a disabled button would have to explain
                            // which of six fields disabled it, which the pane
                            // cannot do without the user already having found
                            // the conflicting pair. An unusable draft is refused
                            // here instead, by `save()`, with a message naming
                            // both sides.
                            let save_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Save Changes")
                                        .strong()
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(0x10, 0x12, 0x16)),
                                )
                                .fill(theme::COLOR_ACCENT_PRIMARY)
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(115.0, 32.0)),
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
                                .min_size(egui::vec2(80.0, 32.0)),
                            );
                            if revert_btn.clicked() {
                                self.model.revert();
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

        ui.label(
            egui::RichText::new("General Settings")
                .strong()
                .size(18.5)
                .color(theme::COLOR_TEXT_PRIMARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Startup integration and multi-monitor spatial settings.")
                .size(12.5)
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

                let control_width = 44.0;
                let gap = 16.0;
                let text_slot_w = (ui.available_width() - control_width - gap).max(180.0);

                // Row 1: Auto-start
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(text_slot_w, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Auto-start on Boot")
                                    .strong()
                                    .size(13.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(c.description)
                                    .size(12.0)
                                    .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
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
                    ui.allocate_ui(egui::vec2(text_slot_w, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Current Monitor Only")
                                    .strong()
                                    .size(13.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Switches windows only on your active display without jumping to other monitors.",
                                )
                                .size(12.0)
                                .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut locked = true;
                        let resp = fluent_toggle_switch_disabled(
                            ui,
                            &mut locked,
                            "Current Monitor Only",
                        );
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
                    ui.allocate_ui(egui::vec2(text_slot_w, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Current Virtual Desktop Only")
                                    .strong()
                                    .size(13.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Keeps window switching strictly inside your active Windows virtual desktop.",
                                )
                                .size(12.0)
                                .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut isolated = true;
                        let resp = fluent_toggle_switch_disabled(
                            ui,
                            &mut isolated,
                            "Current Virtual Desktop Only",
                        );
                        resp.on_hover_text("Virtual desktop isolation is enabled by default.");
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Row 4: UX Honesty Mode
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(text_slot_w, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Show Unresponsive Windows")
                                    .strong()
                                    .size(13.5)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Brings frozen or hanging windows to the front so you can see their status.",
                                )
                                .size(12.0)
                                .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut honesty = true;
                        let resp =
                            fluent_toggle_switch_disabled(ui, &mut honesty, "Show Unresponsive Windows");
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
        ui.label(
            egui::RichText::new("Shortcuts Configuration")
                .strong()
                .size(18.5)
                .color(theme::COLOR_TEXT_PRIMARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(
                "Click any shortcut button to record physical key combinations directly.",
            )
            .size(12.5)
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                let control_width = 145.0;
                let gap = 16.0;
                let text_slot_w = (ui.available_width() - control_width - gap).max(180.0);

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
                    let conflict = self.model.find_conflict(field);

                    ui.horizontal(|ui| {
                        ui.allocate_ui(egui::vec2(text_slot_w, 0.0), |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(field.label()).strong().size(13.5));
                                    if let Some(conf_field) = conflict {
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(format!("⚠️ Conflicts with {}", conf_field.label()))
                                                .size(11.5)
                                                .color(theme::COLOR_WARNING)
                                                .strong(),
                                        );
                                    }
                                });
                                let subtext = match field {
                                    ShortcutField::Switcher => {
                                        "Switches between windows of your active app on this monitor."
                                    }
                                    ShortcutField::Fallback => {
                                        "Alternative shortcut if Win key is used by another app."
                                    }
                                    ShortcutField::SnapLeft => {
                                        "Snaps the active window to the left half of this monitor."
                                    }
                                    ShortcutField::SnapRight => {
                                        "Snaps the active window to the right half of this monitor."
                                    }
                                    ShortcutField::SnapMaximize => {
                                        "Expands the active window to fill this monitor."
                                    }
                                    ShortcutField::Stack => {
                                        "Arranges windows of this app in a clickable stack."
                                    }
                                };
                                ui.label(
                                    egui::RichText::new(subtext)
                                        .size(12.0)
                                        .color(theme::COLOR_TEXT_SECONDARY),
                                );
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let text = if listening {
                                "🔴 Listening…".to_string()
                            } else {
                                format_shortcut_display(&current)
                            };

                            let btn_color = if listening {
                                ui.visuals().selection.stroke.color
                            } else if conflict.is_some() {
                                egui::Color32::from_rgb(0x38, 0x2A, 0x14) // Dark amber background for conflict
                            } else {
                                theme::COLOR_BG_KEYCAP
                            };

                            let stroke_color = if conflict.is_some() {
                                theme::COLOR_WARNING
                            } else {
                                theme::COLOR_STROKE_CARD
                            };

                            let response = ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(text).strong().size(12.5).color(
                                            if listening {
                                                egui::Color32::WHITE
                                            } else if conflict.is_some() {
                                                theme::COLOR_WARNING
                                            } else {
                                                theme::COLOR_TEXT_PRIMARY
                                            },
                                        ),
                                    )
                                    .fill(btn_color)
                                    .stroke(egui::Stroke::new(1.0, stroke_color))
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .min_size(egui::vec2(135.0, 30.0)),
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

                            // Offer Swap only on the row whose capture actually
                            // caused the conflict — that is the only field with
                            // a displaced chord on record for `swap_shortcuts`
                            // to give back to its partner (`can_swap`). The
                            // partner's own row shows the ⚠️ label but no
                            // button: swapping from there has nothing to swap.
                            if let Some(conf_field) = conflict {
                                if self.model.can_swap(field) {
                                    ui.add_space(4.0);
                                    let swap_btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("Swap ⇄")
                                                .size(11.5)
                                                .strong()
                                                .color(theme::COLOR_TEXT_PRIMARY),
                                        )
                                        .fill(theme::COLOR_BG_CARD_HOVER)
                                        .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .min_size(egui::vec2(54.0, 26.0)),
                                    );
                                    swap_btn.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::WidgetType::Button,
                                            true,
                                            theme::SHORTCUT_CONFLICT_SWAP.name,
                                        )
                                    });
                                    if swap_btn.clicked() {
                                        self.model.swap_shortcuts(field, conf_field);
                                    }
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

        ui.label(
            egui::RichText::new("Layout & Snapping")
                .strong()
                .size(18.5)
                .color(theme::COLOR_TEXT_PRIMARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(
                "DPI-aware window snapping and compact overlapping stack arrangement.",
            )
            .size(12.5)
            .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .fill(theme::COLOR_BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::COLOR_STROKE_CARD))
            .corner_radius(egui::CornerRadius::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                let gap = 16.0;

                // Row 1: Toggle Stack (Control width 44.0)
                let text_slot_w1 = (ui.available_width() - 44.0 - gap).max(180.0);
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(text_slot_w1, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(c.name).strong().size(13.5));
                            ui.label(
                                egui::RichText::new(c.description)
                                    .size(12.0)
                                    .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
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

                // Row 2: Stack Width Slider (Control width 180.0)
                let text_slot_w2 = (ui.available_width() - 180.0 - gap).max(180.0);
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(text_slot_w2, 0.0), |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Stack Width Ratio").strong().size(13.5));
                            ui.label(
                                egui::RichText::new(
                                    "Percentage of screen width allocated to stacked windows.",
                                )
                                .size(12.0)
                                .color(theme::COLOR_TEXT_SECONDARY),
                            );
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let slider_resp = ui.add(
                            egui::Slider::new(
                                &mut self.model.draft.layout.stack_width_percent,
                                10..=100,
                            )
                            .show_value(true)
                            .suffix("%"),
                        );
                        slider_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::DragValue,
                                true,
                                theme::STACK_WIDTH_SLIDER.name,
                            )
                        });
                    });
                });
            });
    }

    fn vm_exceptions_pane(&mut self, ui: &mut egui::Ui, drawn: &mut Vec<&'static str>) {
        drawn.push(theme::VM_BYPASS_PROCESS_LIST.name);
        drawn.push(theme::VM_BYPASS_CLASS_LIST.name);

        ui.label(
            egui::RichText::new("VM & Remote Desktop Exceptions")
                .strong()
                .size(18.5)
                .color(theme::COLOR_TEXT_PRIMARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(
                "Exempted virtual machine and remote desktop apps so keys pass directly inside.",
            )
            .size(12.5)
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
                    egui::RichText::new("Excluded Applications (Executables):")
                        .strong()
                        .size(13.5),
                );
                ui.add_space(4.0);
                for proc in &self.model.draft.vm_bypass.bypass_processes {
                    ui.label(
                        egui::RichText::new(format!("  •  {proc}"))
                            .size(12.5)
                            .color(theme::COLOR_TEXT_SECONDARY),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("Excluded Window Types (Classes):")
                        .strong()
                        .size(13.5),
                );
                ui.add_space(4.0);
                for class in &self.model.draft.vm_bypass.bypass_classes {
                    ui.label(
                        egui::RichText::new(format!("  •  {class}"))
                            .size(12.5)
                            .color(theme::COLOR_TEXT_SECONDARY),
                    );
                }
            });
    }

    fn about_pane(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Wira Desk")
                .strong()
                .size(18.5)
                .color(theme::COLOR_TEXT_PRIMARY),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(concat!("Version ", env!("CARGO_PKG_VERSION")))
                .color(theme::COLOR_ACCENT_PRIMARY)
                .strong()
                .size(13.0),
        );
        ui.label(
            egui::RichText::new(match &self.font {
                theme::LoadedFont::System(name) => format!("Typeface: {name}"),
                theme::LoadedFont::Bundled => "Typeface: Bundled Fallback".to_string(),
            })
            .size(12.0)
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
                    egui::RichText::new(
                        "Wira Desk switches smoothly between windows of the active application on your current monitor.",
                    )
                    .size(13.0)
                    .color(theme::COLOR_TEXT_SECONDARY),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Designed to be invisible, fast, and resource-efficient for all-day multitasking.",
                    )
                    .size(12.0)
                    .color(theme::COLOR_TEXT_TERTIARY),
                );
            });
    }
}

/// Draw crisp vector icon for sidebar navigation tabs (General, Shortcuts, Layout, VM, About)
fn draw_sidebar_icon(ui: &mut egui::Ui, kind: usize, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.3, color);

    match kind {
        // 0: General (Gear)
        0 => {
            let center = rect.center();
            painter.circle_stroke(center, 3.5, stroke);
            painter.circle_stroke(center, 5.5, egui::Stroke::new(1.0, color));
        }
        // 1: Shortcuts (Keyboard)
        1 => {
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(2),
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 3.0, rect.top() + 4.5),
                    egui::pos2(rect.left() + 5.0, rect.top() + 4.5),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 8.0, rect.top() + 4.5),
                    egui::pos2(rect.left() + 10.0, rect.top() + 4.5),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 4.0, rect.bottom() - 3.5),
                    egui::pos2(rect.right() - 4.0, rect.bottom() - 3.5),
                ],
                stroke,
            );
        }
        // 2: Layout & Snapping (Split Windows)
        2 => {
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(2),
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top()),
                    egui::pos2(rect.center().x, rect.bottom()),
                ],
                stroke,
            );
        }
        // 3: VM & Exceptions (Display Screen)
        3 => {
            let screen_rect =
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + 10.0));
            painter.rect_stroke(
                screen_rect,
                egui::CornerRadius::same(2),
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, screen_rect.bottom()),
                    egui::pos2(rect.center().x, rect.bottom()),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 3.0, rect.bottom()),
                    egui::pos2(rect.right() - 3.0, rect.bottom()),
                ],
                stroke,
            );
        }
        // 4: About (Info 'i')
        _ => {
            painter.circle_stroke(rect.center(), 6.5, stroke);
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y - 3.0),
                1.0,
                color,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.center().y - 0.5),
                    egui::pos2(rect.center().x, rect.center().y + 3.5),
                ],
                stroke,
            );
        }
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

/// Render a Fluent 2 toggle switch in disabled/locked ON state (Windows 11 standard).
fn fluent_toggle_switch_disabled(
    ui: &mut egui::Ui,
    on: &mut bool,
    accessible_name: &str,
) -> egui::Response {
    let desired_size = egui::vec2(40.0, 20.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *on, accessible_name)
    });

    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();

        // Muted gray track and thumb for distinct locked/read-only indication
        let bg_color = if visuals.dark_mode {
            egui::Color32::from_rgb(0x32, 0x37, 0x42)
        } else {
            egui::Color32::from_rgb(0xd4, 0xd8, 0xe2)
        };

        let stroke_color = if visuals.dark_mode {
            egui::Color32::from_rgba_premultiplied(255, 255, 255, 20)
        } else {
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 20)
        };

        ui.painter().rect(
            rect,
            egui::CornerRadius::same(10),
            bg_color,
            egui::Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Outside,
        );

        let center = egui::pos2(rect.right() - 10.0, rect.center().y);
        let circle_color = if visuals.dark_mode {
            egui::Color32::from_rgb(0x7b, 0x84, 0x96)
        } else {
            egui::Color32::from_rgb(0x8c, 0x93, 0xa0)
        };

        ui.painter().circle_filled(center, 6.0, circle_color);
    }

    response
}

fn is_key_down(vk: u32) -> bool {
    #[cfg(windows)]
    // SAFETY: `GetAsyncKeyState` is safe to call from any thread and only queries physical key state.
    unsafe {
        (windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vk as i32) as u16)
            & 0x8000
            != 0
    }
    #[cfg(not(windows))]
    {
        let _ = vk;
        false
    }
}

fn is_win_key_down() -> bool {
    #[cfg(windows)]
    {
        is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_LWIN as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_RWIN as u32)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn is_alt_key_down() -> bool {
    #[cfg(windows)]
    {
        is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_MENU as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_LMENU as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_RMENU as u32)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn is_ctrl_key_down() -> bool {
    #[cfg(windows)]
    {
        is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_LCONTROL as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_RCONTROL as u32)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn is_shift_key_down() -> bool {
    #[cfg(windows)]
    {
        is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_LSHIFT as u32)
            || is_key_down(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_RSHIFT as u32)
    }
    #[cfg(not(windows))]
    {
        false
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

        let win_pressed = m.mac_cmd || is_win_key_down();
        let alt_pressed = m.alt || is_alt_key_down();
        let ctrl_pressed = m.ctrl || is_ctrl_key_down();
        let shift_pressed = m.shift || is_shift_key_down();

        let mut parts: Vec<&str> = Vec::new();
        if ctrl_pressed {
            parts.push("ctrl");
        }
        if win_pressed {
            parts.push("win");
        }
        if alt_pressed {
            parts.push("alt");
        }
        if shift_pressed {
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
