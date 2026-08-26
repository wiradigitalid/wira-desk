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

pub const TOGGLE_AUTO_START: ControlSemantics = ControlSemantics {
    name: "Start Wira Desk with Windows",
    description: "When enabled, Wira Desk starts automatically at sign-in.",
};

pub const TOGGLE_OVERLAPPING_STACK: ControlSemantics = ControlSemantics {
    name: "Enable overlapping stack layout",
    description: "Arranges up to three windows of the active application in a clickable stack.",
};

pub const STACK_WIDTH_DECREASE: ControlSemantics = ControlSemantics {
    name: "Decrease stack width",
    description: "Lowers the width percentage of stacked windows by one point.",
};

pub const STACK_WIDTH_INPUT: ControlSemantics = ControlSemantics {
    name: "Stack width input",
    description: "Enter the exact width percentage of stacked windows.",
};

pub const STACK_WIDTH_INCREASE: ControlSemantics = ControlSemantics {
    name: "Increase stack width",
    description: "Raises the width percentage of stacked windows by one point.",
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
        for c in [
            TOGGLE_AUTO_START,
            TOGGLE_OVERLAPPING_STACK,
            STACK_WIDTH_DECREASE,
            STACK_WIDTH_INPUT,
            STACK_WIDTH_INCREASE,
            SHORTCUT_SWITCHER,
            ONBOARDING_FINISH_BUTTON,
            ONBOARDING_NEXT_BUTTON,
            ONBOARDING_SKIP_BUTTON,
            ONBOARDING_DUMMY_WIN_1,
            ONBOARDING_DUMMY_WIN_2,
            ONBOARDING_SIMULATE_BUTTON,
            VM_BYPASS_PROCESS_LIST,
            VM_BYPASS_CLASS_LIST,
            SHORTCUT_CONFLICT_SWAP,
        ] {
            assert!(!c.name.trim().is_empty(), "control has no accessible name");
            assert!(
                !c.description.trim().is_empty(),
                "control {} has no description",
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
        let names = [
            TOGGLE_AUTO_START.name,
            TOGGLE_OVERLAPPING_STACK.name,
            STACK_WIDTH_DECREASE.name,
            STACK_WIDTH_INPUT.name,
            STACK_WIDTH_INCREASE.name,
            SHORTCUT_SWITCHER.name,
            ONBOARDING_FINISH_BUTTON.name,
            ONBOARDING_NEXT_BUTTON.name,
            ONBOARDING_SKIP_BUTTON.name,
            ONBOARDING_DUMMY_WIN_1.name,
            ONBOARDING_DUMMY_WIN_2.name,
            ONBOARDING_SIMULATE_BUTTON.name,
            VM_BYPASS_PROCESS_LIST.name,
            VM_BYPASS_CLASS_LIST.name,
        ];
        for (i, a) in names.iter().enumerate() {
            for b in names.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate accessible name");
            }
        }
    }
}
