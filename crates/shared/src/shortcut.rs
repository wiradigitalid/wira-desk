//! Shared keyboard shortcut parsing and representation for daemon and settings.
//! Shortcut format: tokens separated by `+`, case-insensitive,
//! e.g. `"win+backtick"`, `"ctrl+win+left"`, `"ctrl+win+enter"`.
//! Supported modifiers: `win`, `ctrl`, `alt`, `shift`.
//! Virtual-key values are stored as `u16` (Win32 VK code) so `shared` does not
//! need to depend on `windows-sys`. The daemon matches them against `wParam`
//! from the low-level keyboard hook; settings uses them for display/validation.

/// Parsed shortcut: modifier combination plus one main key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub win: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Virtual-key code (Win32 VK_*) of the main key.
    pub vk: u16,
}

impl Shortcut {
    /// Parse from a shortcut string. Returns `None` when no valid main key is
    /// present or an unknown token appears.
    pub fn parse(s: &str) -> Option<Shortcut> {
        let mut sc = Shortcut {
            win: false,
            ctrl: false,
            alt: false,
            shift: false,
            vk: 0,
        };
        let mut has_key = false;

        for raw in s.split('+') {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                continue;
            }
            match token.as_str() {
                "win" | "meta" | "super" => sc.win = true,
                "ctrl" | "control" => sc.ctrl = true,
                "alt" => sc.alt = true,
                "shift" => sc.shift = true,
                other => {
                    // Non-modifier token: must be the main key, and only one is allowed.
                    let vk = vk_from_name(other)?;
                    if has_key {
                        return None; // more than one main key → invalid
                    }
                    sc.vk = vk;
                    has_key = true;
                }
            }
        }

        if has_key {
            Some(sc)
        } else {
            None
        }
    }

    /// Whether at least one modifier is pressed.
    pub fn has_modifier(&self) -> bool {
        self.win || self.ctrl || self.alt || self.shift
    }

    /// Single canonical representation shared by Settings and Hook.
    /// Modifier order is frozen as `ctrl+win+alt+shift` followed by the main key.
    /// Without a fixed order, `"win+ctrl+a"` and `"ctrl+win+a"` would produce two
    /// different strings for the same shortcut, and text-based comparison anywhere
    /// would silently be wrong.
    pub fn to_canonical_string(&self) -> Option<String> {
        let key = name_from_vk(self.vk)?;
        let mut parts: Vec<&str> = Vec::with_capacity(5);
        if self.ctrl {
            parts.push("ctrl");
        }
        if self.win {
            parts.push("win");
        }
        if self.alt {
            parts.push("alt");
        }
        if self.shift {
            parts.push("shift");
        }
        parts.push(key.as_str());
        Some(parts.join("+"))
    }
}

/// Inverse of [`vk_from_name`]: VK code → canonical name.
/// Returns `None` for unsupported VK codes so shortcuts that cannot be
/// represented are never stored in a broken form.
pub fn name_from_vk(vk: u16) -> Option<String> {
    let name = match vk {
        0xC0 => "backtick",
        0x09 => "tab",
        0x0D => "enter",
        0x20 => "space",
        0x1B => "escape",
        0x25 => "left",
        0x26 => "up",
        0x27 => "right",
        0x28 => "down",
        0x41..=0x5A => return Some(((vk as u8) as char).to_ascii_lowercase().to_string()),
        0x30..=0x39 => return Some(((vk as u8) as char).to_string()),
        0x70..=0x7B => return Some(format!("f{}", vk - 0x70 + 1)),
        _ => return None,
    };
    Some(name.to_string())
}

/// Map key name → Win32 virtual-key code.
/// Only the subset relevant to Wira Desk (backtick, arrows, enter, tab, letters, digits, function keys).
pub fn vk_from_name(name: &str) -> Option<u16> {
    let vk = match name {
        // Special characters
        "backtick" | "grave" | "tilde" | "`" => 0xC0, // VK_OEM_3
        "tab" => 0x09,                                // VK_TAB
        "enter" | "return" => 0x0D,                   // VK_RETURN
        "space" | "spacebar" => 0x20,                 // VK_SPACE
        "esc" | "escape" => 0x1B,                     // VK_ESCAPE
        // Arrows
        "left" => 0x25,  // VK_LEFT
        "up" => 0x26,    // VK_UP
        "right" => 0x27, // VK_RIGHT
        "down" => 0x28,  // VK_DOWN
        _ => {
            // Single letter a-z
            if name.len() == 1 {
                let c = name.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    return Some(c.to_ascii_uppercase() as u16);
                }
                if c.is_ascii_digit() {
                    return Some(c as u16); // '0'..='9' → VK 0x30..=0x39
                }
            }
            // Function keys F1-F12
            if let Some(rest) = name.strip_prefix('f') {
                if let Ok(n) = rest.parse::<u16>() {
                    if (1..=12).contains(&n) {
                        return Some(0x70 + (n - 1)); // VK_F1 = 0x70
                    }
                }
            }
            return None;
        }
    };
    Some(vk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_win_backtick() {
        let sc = Shortcut::parse("win+backtick").unwrap();
        assert!(sc.win && !sc.ctrl && !sc.alt && !sc.shift);
        assert_eq!(sc.vk, 0xC0);
    }

    #[test]
    fn parse_ctrl_win_left() {
        let sc = Shortcut::parse("ctrl+win+left").unwrap();
        assert!(sc.ctrl && sc.win);
        assert_eq!(sc.vk, 0x25);
    }

    #[test]
    fn parse_is_case_insensitive_and_trims() {
        let sc = Shortcut::parse(" ALT + Backtick ").unwrap();
        assert!(sc.alt);
        assert_eq!(sc.vk, 0xC0);
    }

    #[test]
    fn parse_enter_and_function_keys() {
        assert_eq!(Shortcut::parse("ctrl+win+enter").unwrap().vk, 0x0D);
        assert_eq!(Shortcut::parse("f5").unwrap().vk, 0x74);
        assert_eq!(Shortcut::parse("a").unwrap().vk, b'A' as u16);
    }

    #[test]
    fn parse_rejects_modifier_only_or_unknown() {
        assert!(Shortcut::parse("ctrl+win").is_none());
        assert!(Shortcut::parse("win+notarealkey").is_none());
        assert!(Shortcut::parse("").is_none());
    }

    #[test]
    fn parse_rejects_more_than_one_main_key() {
        assert!(Shortcut::parse("ctrl+a+b").is_none());
        assert!(Shortcut::parse("win+left+right").is_none());
    }

    // ── : one canonical representation ────────────────────────────

    #[test]
    fn canonical_modifier_order_is_frozen() {
        let sc = Shortcut::parse("shift+alt+win+ctrl+a").unwrap();
        assert_eq!(sc.to_canonical_string().unwrap(), "ctrl+win+alt+shift+a");
    }

    #[test]
    fn differently_ordered_input_canonicalizes_identically() {
        let a = Shortcut::parse("win+ctrl+left").unwrap();
        let b = Shortcut::parse("ctrl+win+left").unwrap();
        assert_eq!(a.to_canonical_string(), b.to_canonical_string());
        assert_eq!(a.to_canonical_string().unwrap(), "ctrl+win+left");
    }

    #[test]
    fn canonical_form_round_trips_through_parse() {
        for input in [
            "win+backtick",
            "alt+backtick",
            "ctrl+win+left",
            "ctrl+win+right",
            "ctrl+win+enter",
            "ctrl+win+down",
            "f5",
            "a",
            "7",
        ] {
            let parsed = Shortcut::parse(input).unwrap();
            let canonical = parsed.to_canonical_string().unwrap();
            let reparsed = Shortcut::parse(&canonical).unwrap();
            assert_eq!(parsed, reparsed, "round-trip lost information for {input}");
        }
    }

    #[test]
    fn every_frozen_default_has_a_canonical_form() {
        // The frozen defaults must all be representable, or
        // Settings could not display what the daemon is actually using.
        for d in [
            "win+backtick",
            "alt+backtick",
            "ctrl+win+left",
            "ctrl+win+right",
            "ctrl+win+enter",
            "ctrl+win+down",
        ] {
            let sc = Shortcut::parse(d).unwrap();
            assert_eq!(sc.to_canonical_string().unwrap(), d, "default {d} drifted");
        }
    }

    #[test]
    fn name_from_vk_inverts_vk_from_name() {
        for name in [
            "backtick", "tab", "enter", "space", "escape", "left", "up", "right", "down", "a", "z",
            "0", "9", "f1", "f12",
        ] {
            let vk = vk_from_name(name).unwrap();
            assert_eq!(name_from_vk(vk).as_deref(), Some(name), "vk {vk:#x}");
        }
    }

    #[test]
    fn unrepresentable_vk_yields_no_canonical_form() {
        let broken = Shortcut {
            win: true,
            ctrl: false,
            alt: false,
            shift: false,
            vk: 0xFF,
        };
        assert_eq!(broken.to_canonical_string(), None);
    }
}
