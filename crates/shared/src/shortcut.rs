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

/// Category of Windows OS hotkey reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reservation {
    /// Immutable kernel / secure desktop chords (e.g. Win+L, Ctrl+Alt+Del).
    Immutable,
    /// Windows Shell / Explorer chords (e.g. Win+1..9, Win+D, Win+E).
    ShellOwned,
}

/// Information about a reserved Windows system hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservedInfo {
    pub kind: Reservation,
    pub owner: &'static str,
}

/// Check if a parsed shortcut is a reserved Windows system hotkey.
/// Reservation is evaluated per-chord, never per-key.
pub fn reservation(sc: &Shortcut) -> Option<ReservedInfo> {
    // 1. Immutable System Shortcuts
    if sc.win && !sc.ctrl && !sc.alt && !sc.shift && sc.vk == 0x4C {
        // Win + L
        return Some(ReservedInfo {
            kind: Reservation::Immutable,
            owner: "lock your PC",
        });
    }

    // 2. Escape Hatches (Product Invariant: never take the user's escape hatch)
    if sc.alt && !sc.win && !sc.ctrl && !sc.shift {
        if sc.vk == 0x09 {
            // Alt + Tab
            return Some(ReservedInfo {
                kind: Reservation::Immutable,
                owner: "the Windows window switcher",
            });
        }
        if sc.vk == 0x73 {
            // Alt + F4
            return Some(ReservedInfo {
                kind: Reservation::Immutable,
                owner: "close the active window",
            });
        }
    }

    if sc.alt && sc.shift && !sc.win && !sc.ctrl && sc.vk == 0x09 {
        // Alt + Shift + Tab
        return Some(ReservedInfo {
            kind: Reservation::Immutable,
            owner: "the Windows reverse window switcher",
        });
    }

    // Ctrl + Shift + Escape (Task Manager). Unmeasured on this product's target
    // Windows builds (`OQ-16`), so it defaults to the kind Windows keeps
    // regardless rather than the kind Wira Desk merely declines to take.
    if sc.ctrl && sc.shift && !sc.win && !sc.alt && sc.vk == 0x1B {
        return Some(ReservedInfo {
            kind: Reservation::Immutable,
            owner: "Task Manager",
        });
    }

    // Ctrl + Win + Enter is still absent from this catalogue, and the reason has changed.
    // It launches Narrator on stock Windows, and it used to be this product's own shipped
    // `snap_maximize` default, so adding it would have made that default fail its own
    // validation. `DEC-008` moved the family to `Ctrl+Alt`, so that premise no longer holds
    // and the chord *could* now be catalogued. It is left out because `DEC-008`'s scope was
    // the two virtual-desktop arrows, and adding a third entry is a decision nobody has
    // taken — `OQ-16` still carries it as unmeasured.

    // 3. Shell-Owned Shortcuts (Win + Key)
    if sc.win && !sc.ctrl && !sc.alt && !sc.shift {
        match sc.vk {
            0x31 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the first app pinned to your taskbar",
                })
            }
            0x32 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the second app pinned to your taskbar",
                })
            }
            0x33 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the third app pinned to your taskbar",
                })
            }
            0x34 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the fourth app pinned to your taskbar",
                })
            }
            0x35 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the fifth app pinned to your taskbar",
                })
            }
            0x36 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the sixth app pinned to your taskbar",
                })
            }
            0x37 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the seventh app pinned to your taskbar",
                })
            }
            0x38 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the eighth app pinned to your taskbar",
                })
            }
            0x39 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the ninth app pinned to your taskbar",
                })
            }
            0x30 => {
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the tenth app pinned to your taskbar",
                })
            }
            0x44 => {
                // Win + D
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "show and hide your desktop",
                });
            }
            0x45 => {
                // Win + E
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "File Explorer",
                });
            }
            0x52 => {
                // Win + R
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the Run dialog",
                });
            }
            0x56 => {
                // Win + V
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "clipboard history",
                });
            }
            0x5A => {
                // Win + Z
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Snap Layouts",
                });
            }
            0x09 => {
                // Win + Tab
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Task View",
                });
            }
            0x41 => {
                // Win + A
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Quick Settings",
                });
            }
            0x4E => {
                // Win + N
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the notification center",
                });
            }
            0x53 | 0x51 => {
                // Win + S / Win + Q
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Windows Search",
                });
            }
            0x49 => {
                // Win + I
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Windows Settings",
                });
            }
            0x58 => {
                // Win + X
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "the Quick Link menu",
                });
            }
            0x25..=0x28 => {
                // Win + Arrows
                return Some(ReservedInfo {
                    kind: Reservation::ShellOwned,
                    owner: "Windows window snapping",
                });
            }
            _ => {}
        }
    }

    // Win + Shift + S (Snipping tool)
    if sc.win && sc.shift && !sc.ctrl && !sc.alt && sc.vk == 0x53 {
        return Some(ReservedInfo {
            kind: Reservation::ShellOwned,
            owner: "the Snipping Tool screenshot capture",
        });
    }

    // Win + Ctrl + (D / F4) -> Virtual desktops creation and deletion
    if sc.win && sc.ctrl && !sc.alt && !sc.shift && matches!(sc.vk, 0x44 | 0x73) {
        return Some(ReservedInfo {
            kind: Reservation::ShellOwned,
            owner: "Windows virtual desktops",
        });
    }

    // Win + Ctrl + Left / Right -> navigate between virtual desktops.
    //
    // The gap this closes: the catalogue already listed `Win+Ctrl+D` and `Win+Ctrl+F4`,
    // which *create* and *close* a virtual desktop, and skipped the two arrows that
    // *navigate* between them. Three quarters of one shell feature was catalogued and the
    // quarter this product's own default sat on was not — so `ctrl+win+left` shipped as the
    // snap default and silently took a Windows function, which is precisely what `DEC-003`
    // forbids. Adding these two was only possible once `DEC-008` moved the family off them:
    // while `ctrl+win+left` was a shipped default, cataloguing it would have made that
    // default fail its own validation.
    //
    // `ShellOwned` rather than `Immutable`: the low-level hook demonstrably can swallow
    // these — that is how the old default worked at all — so the refusal can honestly offer
    // an alternative, which is the distinction `DEC-003` draws between the two kinds.
    if sc.win && sc.ctrl && !sc.alt && !sc.shift && matches!(sc.vk, 0x25 | 0x27) {
        return Some(ReservedInfo {
            kind: Reservation::ShellOwned,
            owner: "switch between your virtual desktops",
        });
    }

    None
}

/// Check if a parsed shortcut is hardcoded/reserved by the Windows operating system.
pub fn is_reserved_system_shortcut(sc: &Shortcut) -> bool {
    reservation(sc).is_some()
}

/// Suggest an available alternative modifier combination for a rejected chord.
/// Tests candidate modifier layers deterministically against the reserved catalog
/// and current draft fields.
pub fn suggest_alternative<F>(sc: &Shortcut, is_field_conflict: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    // Try deterministic modifier ladder
    let ladder: [(bool, bool, bool, bool); 5] = [
        (true, sc.win, sc.alt, sc.shift),  // + Ctrl
        (sc.ctrl, sc.win, true, sc.shift), // + Alt
        (true, sc.win, true, sc.shift),    // + Ctrl + Alt
        (true, sc.win, sc.alt, true),      // + Ctrl + Shift
        (true, sc.win, true, true),        // + Ctrl + Alt + Shift
    ];

    for (ctrl, win, alt, shift) in ladder {
        let candidate = Shortcut {
            win,
            ctrl,
            alt,
            shift,
            vk: sc.vk,
        };
        // 1. Must not be in the reserved catalog
        if reservation(&candidate).is_some() {
            continue;
        }
        // 2. Must produce a canonical string
        if let Some(canonical) = candidate.to_canonical_string() {
            // 3. Must not collide with another field in draft
            if !is_field_conflict(&canonical) {
                return Some(canonical);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_ctrl_arrow_is_shell_owned() {
        for chord in ["win+ctrl+left", "win+ctrl+right"] {
            let sc = Shortcut::parse(chord).expect("parses");
            let info = reservation(&sc).unwrap_or_else(|| panic!("{chord} must be reserved"));
            assert_eq!(info.kind, Reservation::ShellOwned);
            assert_eq!(info.owner, "switch between your virtual desktops");
        }
    }

    #[test]
    fn win_ctrl_up_and_down_are_not_claimed() {
        // Only the two arrows Windows actually navigates desktops with. Reserving the whole
        // arrow cluster would refuse chords nothing owns, which is its own kind of wrong.
        for chord in ["win+ctrl+up", "win+ctrl+down"] {
            let sc = Shortcut::parse(chord).expect("parses");
            assert!(reservation(&sc).is_none(), "{chord} must stay available");
        }
    }

    #[test]
    fn ctrl_alt_arrow_is_not_reserved() {
        // The family the shipped defaults moved to. Not in the catalogue, because the
        // catalogue lists chords WINDOWS owns — a graphics driver binding screen rotation to
        // these is a real cost (OQ-20) that no list of Windows chords can express, and
        // DEC-002 forbids probing for it.
        for chord in [
            "ctrl+alt+left",
            "ctrl+alt+right",
            "ctrl+alt+up",
            "ctrl+alt+down",
            "ctrl+alt+enter",
            "ctrl+alt+shift+down",
            "ctrl+alt+shift+enter",
        ] {
            let sc = Shortcut::parse(chord).expect("parses");
            assert!(reservation(&sc).is_none(), "{chord} must be configurable");
        }
    }

    #[test]
    fn adding_ctrl_leaves_the_escape_hatches_alone() {
        // The escape-hatch guards are conditioned on `alt && !ctrl`, so the Ctrl+Alt family
        // cannot collide with Alt+Tab, Alt+F4, or Alt+Shift+Tab by construction rather than
        // by luck. Asserted because that is the load-bearing reason the family was safe.
        for chord in ["ctrl+alt+tab", "ctrl+alt+f4"] {
            let sc = Shortcut::parse(chord).expect("parses");
            assert!(reservation(&sc).is_none(), "{chord} must stay available");
        }
        for chord in ["alt+tab", "alt+f4"] {
            let sc = Shortcut::parse(chord).expect("parses");
            assert!(reservation(&sc).is_some(), "{chord} must stay refused");
        }
    }

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
    fn ctrl_shift_escape_is_reserved_for_task_manager() {
        let sc = Shortcut::parse("ctrl+shift+escape").unwrap();
        let info = reservation(&sc).expect("Ctrl+Shift+Esc must be reserved");
        assert_eq!(info.kind, Reservation::Immutable);
    }

    #[test]
    fn shipped_snap_maximize_default_is_never_reserved() {
        // Ctrl+Win+Enter launches Narrator on stock Windows and would
        // otherwise default to Immutable per OQ-16, but it is also this
        // product's own shipped `snap_maximize` default — reserving it would
        // make the shipped default fail its own validation, exactly the
        // carve-out DEC-003 already makes for Alt+Backtick.
        let sc = Shortcut::parse("ctrl+win+enter").unwrap();
        assert_eq!(reservation(&sc), None);
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
