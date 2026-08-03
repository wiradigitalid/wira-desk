//! `u8` command enum flowing through the Hook → Worker ring buffer.
//! The ring buffer carries only raw `u8` values — no heap allocation. Both sides
//! convert via `Command::from_u8` and `Command as u8`.

/// Command decoded by the Hook Thread from key combinations, then executed
/// by the Worker Thread. Discrete `u8` values for safe transfer via ring buffer.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// No command (empty slot).
    Nop = 0,
    /// Rotate focus among same-app windows (Win+Backtick).
    Cycle = 1,
    /// Snap the active window to the left half of the screen (Ctrl+Win+Left).
    SnapLeft = 2,
    /// Snap the active window to the right half of the screen (Ctrl+Win+Right).
    SnapRight = 3,
    /// Maximize the active window (Ctrl+Win+Enter).
    SnapMaximize = 4,
    /// Rearrange windows with overlapping stack layout (Ctrl+Win+Down).
    OverlappingStack = 5,
}

impl Command {
    /// Convert from raw `u8` (read from ring buffer). Unknown values → `Nop`.
    #[inline]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Command::Cycle,
            2 => Command::SnapLeft,
            3 => Command::SnapRight,
            4 => Command::SnapMaximize,
            5 => Command::OverlappingStack,
            _ => Command::Nop,
        }
    }

    /// Convert to raw `u8` for writing to the ring buffer.
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_commands() {
        for cmd in [
            Command::Nop,
            Command::Cycle,
            Command::SnapLeft,
            Command::SnapRight,
            Command::SnapMaximize,
            Command::OverlappingStack,
        ] {
            assert_eq!(Command::from_u8(cmd.as_u8()), cmd);
        }
    }

    #[test]
    fn unknown_values_map_to_nop() {
        assert_eq!(Command::from_u8(6), Command::Nop);
        assert_eq!(Command::from_u8(255), Command::Nop);
    }

    /// frozen extension contract: these `u8` values travel through
    /// the ring buffer and are consumed by Epics 3, 4, and 5 as sibling lanes.
    /// Renumbering any of them silently changes what a queued command means,
    /// so the wire values are pinned rather than merely derived.
    #[test]
    fn frozen_command_wire_values() {
        assert_eq!(Command::Nop.as_u8(), 0);
        assert_eq!(Command::Cycle.as_u8(), 1);
        assert_eq!(Command::SnapLeft.as_u8(), 2);
        assert_eq!(Command::SnapRight.as_u8(), 3);
        assert_eq!(Command::SnapMaximize.as_u8(), 4);
        assert_eq!(Command::OverlappingStack.as_u8(), 5);
    }
}
