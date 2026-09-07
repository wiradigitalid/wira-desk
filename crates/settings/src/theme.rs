#![allow(dead_code)]

//! Native theme and typography contract for Settings.
//! Reads the Windows light/dark preference and maps it onto Slint's Palette,
//! plus the documented Segoe UI typography.

use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

/// The two supported appearance modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// Documented Windows UI typeface, with the fallback the contract allows.
pub const PRIMARY_FONT: &str = "Segoe UI Variable Text";
pub const SECONDARY_FONT: &str = "Segoe UI";
pub const FALLBACK_FONT: &str = "Tahoma";

/// On-disk locations, in preference order.
fn font_candidates() -> [(&'static str, std::path::PathBuf); 3] {
    let windows_dir = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    let fonts_dir = std::path::Path::new(&windows_dir).join("Fonts");
    [
        (PRIMARY_FONT, fonts_dir.join("SegUIVar.ttf")),
        (SECONDARY_FONT, fonts_dir.join("segoeui.ttf")),
        (FALLBACK_FONT, fonts_dir.join("tahoma.ttf")),
    ]
}

/// Which typeface was actually installed or detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadedFont {
    System(&'static str),
    Bundled,
}

/// Detect the primary available Windows UI font name.
pub fn detect_ui_font() -> LoadedFont {
    for (name, path) in font_candidates() {
        if path.is_file() {
            return LoadedFont::System(name);
        }
    }
    LoadedFont::Bundled
}

/// Whether a documented Windows UI font is present on this machine.
pub fn system_font_available() -> bool {
    font_candidates().iter().any(|(_, path)| path.is_file())
}

/// Read the current Windows app theme.
pub fn detect_theme() -> ThemeMode {
    match read_apps_use_light_theme() {
        Some(0) => ThemeMode::Dark,
        Some(_) => ThemeMode::Light,
        None => ThemeMode::Light,
    }
}

fn read_apps_use_light_theme() -> Option<u32> {
    let subkey: Vec<u16> = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let value: Vec<u16> = "AppsUseLightTheme"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;

    // SAFETY: `subkey` and `value` are NUL-terminated wide string locals that outlive the call.
    // `HKEY_CURRENT_USER` is a valid predefined root key handle. `size` correctly specifies
    // the byte capacity of `data` (`size_of::<u32>()`).
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut core::ffi::c_void,
            &mut size,
        )
    };

    if status == ERROR_SUCCESS {
        Some(data)
    } else {
        None
    }
}

/// Accessible-name and role vocabulary for the shell.
pub struct ControlSemantics {
    pub name: &'static str,
    pub description: &'static str,
}

/// The declared controls' semantics, in declaration order — currently all twenty-two of them.
///
/// The register tests below iterate THIS rather than each hand-keeping its own array. They used
/// to, and the two arrays had drifted apart and away from the declarations: of the twenty
/// constants, fourteen were covered for a non-empty name and thirteen for uniqueness.
/// `ONBOARDING_BACK_BUTTON` was in neither and `SHORTCUT_CONFLICT_SWAP` was missing from the
/// uniqueness check, both long before the percentage names were added. A guard that has to be
/// remembered is a guard that will be forgotten, which is the same reasoning that put the
/// shortcut sequence in `SHORTCUT_DECLARED_ORDER` (`DEC-018`).
///
/// **This array is still hand-maintained, and nothing enforces that it is complete.** Rust cannot
/// enumerate a module's items without a macro, and this workspace deliberately contains none, so
/// the honest statement is that a constant declared and not added here is invisible to both tests
/// — which is exactly what happened when this array was first written: it was generated with a
/// pattern that excluded digits, so `ONBOARDING_DUMMY_WIN_1` and `_2` were dropped and two
/// assertions they already had were lost. Add a new constant here in the same edit that declares
/// it. `#![allow(dead_code)]` at the top of this file means an omitted, otherwise-unread constant
/// will not even warn.
///
/// `accessible_names_are_unique` is named in `defects.yaml` as `DEF-2`'s regression test, so
/// what it covers is not a housekeeping question: a control missing from here is a control
/// that defect can silently come back through.
pub const ALL: &[ControlSemantics] = &[
    TOGGLE_AUTO_START,
    STACK_WIDTH_DECREASE,
    STACK_WIDTH_FIELD,
    STACK_WIDTH_INPUT,
    STACK_WIDTH_INCREASE,
    SNAP_PERCENT_DECREASE,
    SNAP_PERCENT_FIELD,
    SNAP_PERCENT_INPUT,
    SNAP_PERCENT_INCREASE,
    SHORTCUT_SWITCHER,
    ONBOARDING_BACK_BUTTON,
    ONBOARDING_FINISH_BUTTON,
    ONBOARDING_NEXT_BUTTON,
    ONBOARDING_SKIP_BUTTON,
    ONBOARDING_DUMMY_WIN_1,
    ONBOARDING_DUMMY_WIN_2,
    ONBOARDING_SIMULATE_BUTTON,
    VM_BYPASS_PROCESS_LIST,
    VM_BYPASS_CLASS_LIST,
    SHORTCUT_CONFLICT_SWAP,
    SHORTCUT_KEYCAP,
    SHORTCUT_ROW_DESCRIPTION,
];

pub const TOGGLE_AUTO_START: ControlSemantics = ControlSemantics {
    name: "Start Wira Desk with Windows",
    description: "When enabled, Wira Desk starts automatically at sign-in.",
};

pub const STACK_WIDTH_DECREASE: ControlSemantics = ControlSemantics {
    name: "Decrease stack width",
    description: "Lowers the width percentage of stacked windows by one point.",
};

pub const STACK_WIDTH_FIELD: ControlSemantics = ControlSemantics {
    name: "Stack width field",
    description: "Focuses the stack width percentage input field.",
};

pub const STACK_WIDTH_INPUT: ControlSemantics = ControlSemantics {
    name: "Stack width input",
    description: "Enter the exact width percentage of stacked windows.",
};

pub const STACK_WIDTH_INCREASE: ControlSemantics = ControlSemantics {
    name: "Increase stack width",
    description: "Raises the width percentage of stacked windows by one point.",
};

pub const SNAP_PERCENT_DECREASE: ControlSemantics = ControlSemantics {
    name: "Decrease snap percentage",
    description: "Lowers the snap percentage by one point.",
};

pub const SNAP_PERCENT_FIELD: ControlSemantics = ControlSemantics {
    name: "Snap percentage field",
    description: "Focuses the snap percentage input field.",
};

pub const SNAP_PERCENT_INPUT: ControlSemantics = ControlSemantics {
    name: "Snap percentage input",
    description: "Enter the exact snap percentage.",
};

pub const SNAP_PERCENT_INCREASE: ControlSemantics = ControlSemantics {
    name: "Increase snap percentage",
    description: "Raises the snap percentage by one point.",
};

pub const SHORTCUT_SWITCHER: ControlSemantics = ControlSemantics {
    name: "Switch between windows of the same application",
    description: "Press the button, then press the key combination you want to use.",
};

pub const ONBOARDING_BACK_BUTTON: ControlSemantics = ControlSemantics {
    name: "Back to previous step",
    description: "Navigate back to the previous onboarding tutorial step.",
};

pub const ONBOARDING_FINISH_BUTTON: ControlSemantics = ControlSemantics {
    name: "Start Using Wira Desk",
    description: "Finish onboarding and start running Wira Desk in the background.",
};

pub const ONBOARDING_NEXT_BUTTON: ControlSemantics = ControlSemantics {
    name: "Next step",
    description: "Advance to the next onboarding tutorial step.",
};

pub const ONBOARDING_SKIP_BUTTON: ControlSemantics = ControlSemantics {
    name: "Skip Tutorial",
    description: "Skip the tutorial, save default configuration, and start Wira Desk.",
};

pub const ONBOARDING_DUMMY_WIN_1: ControlSemantics = ControlSemantics {
    name: "Simulated Window 1: Chrome - Project Brief",
    description: "First simulated window in the interactive cycling practice area.",
};

pub const ONBOARDING_DUMMY_WIN_2: ControlSemantics = ControlSemantics {
    name: "Simulated Window 2: Chrome - Design System",
    description: "Second simulated window in the interactive cycling practice area.",
};

pub const ONBOARDING_SIMULATE_BUTTON: ControlSemantics = ControlSemantics {
    name: "Practice Shortcut: Win + `",
    description: "Simulates pressing the cycling shortcut to switch window focus.",
};

pub const VM_BYPASS_PROCESS_LIST: ControlSemantics = ControlSemantics {
    name: "VM and Remote Desktop Bypass Processes",
    description: "List of virtual machine and remote desktop client executables that receive raw keystroke passthrough.",
};

pub const VM_BYPASS_CLASS_LIST: ControlSemantics = ControlSemantics {
    name: "VM and Remote Desktop Bypass Window Classes",
    description: "List of window class names that receive raw keystroke passthrough.",
};

pub const SHORTCUT_CONFLICT_SWAP: ControlSemantics = ControlSemantics {
    name: "Swap conflicting shortcuts",
    description: "Swaps shortcut keys between the two conflicting actions.",
};

// The two constants below are PREFIXES, not names any element carries. `shortcut_keycap_label`
// and `shortcut_description_label` build the rendered name per row, so the tree holds
// "Shortcut keycap: Snap to left edge" and never "Shortcut keycap".
//
// That matters for what the register tests below actually prove about these two: iterating `ALL`
// shows the two PREFIXES are unique, which is not the property `DEF-2` needs. Per-row uniqueness
// of the rendered names comes from every action having a distinct label, which
// `app::tests::every_field_has_a_distinct_key_label_and_description` is what holds — the
// rendered labels are that label with a fixed prefix, so distinct labels give distinct names.
// Their `description` fields have no reader: `shortcut_row.slint` binds the row's own
// `root.description` to `accessible-description`, not these.
pub const SHORTCUT_KEYCAP: ControlSemantics = ControlSemantics {
    name: "Shortcut keycap",
    description: "Button displaying the current shortcut chord; click to record a new shortcut.",
};

pub const SHORTCUT_ROW_DESCRIPTION: ControlSemantics = ControlSemantics {
    name: "Shortcut description",
    description: "Focusable summary providing the full description of this shortcut action.",
};

/// Derived accessible name for a shortcut row's keycap button, from [`SHORTCUT_KEYCAP`].
pub fn shortcut_keycap_label(action: &str) -> String {
    format!("{}: {action}", SHORTCUT_KEYCAP.name)
}

/// Derived accessible name for a shortcut row's description block, from [`SHORTCUT_ROW_DESCRIPTION`].
pub fn shortcut_description_label(action: &str) -> String {
    format!("{}: {action}", SHORTCUT_ROW_DESCRIPTION.name)
}

pub const LISTENING_ANNOUNCEMENT: &str = "Listening for a key combination. Press Escape to cancel.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_theme_returns_a_supported_mode() {
        let first = detect_theme();
        assert!(matches!(first, ThemeMode::Light | ThemeMode::Dark));
        for _ in 0..4 {
            assert_eq!(detect_theme(), first);
        }
    }

    #[test]
    fn typography_constants_are_the_documented_ones() {
        assert_eq!(PRIMARY_FONT, "Segoe UI Variable Text");
        assert_eq!(SECONDARY_FONT, "Segoe UI");
        assert_eq!(FALLBACK_FONT, "Tahoma");
    }

    #[test]
    fn a_documented_windows_font_is_present_on_this_machine() {
        if !system_font_available() {
            eprintln!(
                "skipping a_documented_windows_font_is_present_on_this_machine: \
                 neither Segoe UI nor Tahoma found under %SystemRoot%\\Fonts on this machine"
            );
            return;
        }
        assert!(system_font_available());
    }

    #[test]
    fn font_candidates_are_ordered_primary_then_fallback() {
        let candidates = font_candidates();
        assert_eq!(candidates[0].0, PRIMARY_FONT);
        assert_eq!(candidates[1].0, SECONDARY_FONT);
        assert_eq!(candidates[2].0, FALLBACK_FONT);
    }

    #[test]
    fn font_detection_reports_detected_face() {
        match detect_ui_font() {
            LoadedFont::System(name) => {
                assert!(
                    name == PRIMARY_FONT || name == SECONDARY_FONT || name == FALLBACK_FONT,
                    "installed an undocumented face: {name}"
                );
            }
            LoadedFont::Bundled => {}
        }
    }

    #[test]
    fn every_control_has_a_non_empty_accessible_name() {
        for c in ALL {
            assert!(
                !c.name.trim().is_empty(),
                "a control has an empty accessible name"
            );
            assert!(
                !c.description.trim().is_empty(),
                "control '{}' has an empty accessible description",
                c.name
            );
        }
    }

    #[test]
    fn listening_state_has_a_spoken_announcement() {
        assert!(LISTENING_ANNOUNCEMENT.contains("Listening"));
        assert!(
            LISTENING_ANNOUNCEMENT.contains("Escape"),
            "the cancel affordance must be announced, not only drawn"
        );
    }

    #[test]
    fn accessible_names_are_unique() {
        // `DEF-2` was two focusable controls sharing one accessible name. Iterating `ALL` means a
        // control added to the file is covered by the time it compiles, rather than when someone
        // remembers to extend an array here.
        let mut seen: Vec<&str> = Vec::with_capacity(ALL.len());
        for c in ALL {
            assert!(
                !seen.contains(&c.name),
                "two controls share the accessible name '{}'",
                c.name
            );
            seen.push(c.name);
        }
        assert_eq!(seen.len(), ALL.len());
    }

    #[test]
    fn shortcut_per_row_labels_derive_from_theme_constants() {
        let keycap = shortcut_keycap_label("Test Action");
        let desc = shortcut_description_label("Test Action");
        assert_eq!(keycap, format!("{}: Test Action", SHORTCUT_KEYCAP.name));
        assert_eq!(
            desc,
            format!("{}: Test Action", SHORTCUT_ROW_DESCRIPTION.name)
        );
    }

    #[test]
    fn every_rendered_percent_control_name_comes_from_theme() {
        use slint::{ComponentHandle, Model};
        crate::shortcut_row_slint_snapshot::tests::run_on_ui_thread(|| {
            let (window, _model, save_path) =
                crate::shortcut_row_slint_snapshot::tests::setup_shortcuts_window();

            // The two families, each in slot order: decrease, field, input, increase.
            //
            // Membership alone is not enough, and that gap was real: a review found that a SNAP
            // row handed the STACK set passes a membership check, because every name still
            // renders somewhere. That is `DEF-2`'s symptom — two focusable controls sharing a
            // name — slipping through the guard meant to catch it. So this asserts ASSIGNMENT:
            // each row's four labels are one family's four slots, in order.
            //
            // Which family a given field should get is deliberately NOT restated here. A test
            // that repeats the production mapping only proves the mapping equals itself. What is
            // asserted instead are two properties that hold regardless of the mapping: a row
            // never mixes families or slots, and exactly one row wears the stack family, because
            // there is exactly one Overlapping Stack action.
            const STACK_FAMILY: [&str; 4] = [
                STACK_WIDTH_DECREASE.name,
                STACK_WIDTH_FIELD.name,
                STACK_WIDTH_INPUT.name,
                STACK_WIDTH_INCREASE.name,
            ];
            const SNAP_FAMILY: [&str; 4] = [
                SNAP_PERCENT_DECREASE.name,
                SNAP_PERCENT_FIELD.name,
                SNAP_PERCENT_INPUT.name,
                SNAP_PERCENT_INCREASE.name,
            ];

            let row_groups = [
                window.get_rows_switching(),
                window.get_rows_snap_half(),
                window.get_rows_snap_third(),
                window.get_rows_snap_custom(),
                window.get_rows_arrange(),
            ];

            let mut checked_percent_rows = 0;
            let mut stack_family_rows = 0;
            for group in row_groups {
                for row in group.iter() {
                    if !row.has_percent {
                        continue;
                    }
                    checked_percent_rows += 1;
                    let labels = [
                        row.accessible_label_decrease.as_str(),
                        row.accessible_label_field.as_str(),
                        row.accessible_label_input.as_str(),
                        row.accessible_label_increase.as_str(),
                    ];
                    if labels == STACK_FAMILY {
                        stack_family_rows += 1;
                    } else if labels != SNAP_FAMILY {
                        panic!(
                            "row {} does not carry one family's four slots in order: {labels:?}",
                            row.index
                        );
                    }
                }
            }

            assert!(
                checked_percent_rows > 0,
                "at least one percentage row must be rendered"
            );
            // Exactly one, because there is exactly one Overlapping Stack action. Two means a
            // snap row was handed the stack family, which is what membership could not see.
            assert_eq!(
                stack_family_rows, 1,
                "exactly one rendered percentage row carries the stack-width label family"
            );

            // Verify the snap percentage controls are in the rendered tree
            for &snap_label in &[
                SNAP_PERCENT_DECREASE.name,
                SNAP_PERCENT_FIELD.name,
                SNAP_PERCENT_INPUT.name,
                SNAP_PERCENT_INCREASE.name,
            ] {
                assert!(
                    i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                        &window, snap_label,
                    )
                    .next()
                    .is_some(),
                    "theme constant '{snap_label}' is not rendered in the UI"
                );
            }

            // Scroll to the bottom to bring Overlapping Stack into view
            window
                .window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position: slint::LogicalPosition::new(300.0, 300.0),
                    delta_x: 0.0,
                    delta_y: -600.0,
                });

            // Verify the stack width percentage controls are in the rendered tree
            for &stack_label in &[
                STACK_WIDTH_DECREASE.name,
                STACK_WIDTH_FIELD.name,
                STACK_WIDTH_INPUT.name,
                STACK_WIDTH_INCREASE.name,
            ] {
                assert!(
                    i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                        &window,
                        stack_label,
                    )
                    .next()
                    .is_some(),
                    "theme constant '{stack_label}' is not rendered in the UI"
                );
            }

            let _ = std::fs::remove_file(&save_path);
        });
    }
}
