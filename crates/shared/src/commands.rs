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
    /// Rearrange windows with overlapping stack layout.
    OverlappingStack = 5,
    /// Snap the active window to the top half of the screen.
    SnapTop = 6,
    /// Snap the active window to the bottom half of the screen.
    SnapBottom = 7,
    /// Move the active window to the next physical monitor, keeping its share
    /// of the work area.
    MoveToNextMonitor = 8,
    /// Snap the active window against the left edge at a configured percentage.
    SnapPercentLeft = 9,
    /// Snap the active window against the right edge at a configured percentage.
    SnapPercentRight = 10,
    /// Snap the active window against the top edge at a configured percentage.
    SnapPercentTop = 11,
    /// Snap the active window against the bottom edge at a configured percentage.
    SnapPercentBottom = 12,
    /// Snap the active window to the left third of the screen.
    SnapThirdLeft = 13,
    /// Snap the active window to the middle third of the screen.
    SnapThirdMiddle = 14,
    /// Snap the active window to the right third of the screen.
    SnapThirdRight = 15,
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
            6 => Command::SnapTop,
            7 => Command::SnapBottom,
            8 => Command::MoveToNextMonitor,
            9 => Command::SnapPercentLeft,
            10 => Command::SnapPercentRight,
            11 => Command::SnapPercentTop,
            12 => Command::SnapPercentBottom,
            13 => Command::SnapThirdLeft,
            14 => Command::SnapThirdMiddle,
            15 => Command::SnapThirdRight,
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
            Command::SnapTop,
            Command::SnapBottom,
            Command::MoveToNextMonitor,
            Command::SnapPercentLeft,
            Command::SnapPercentRight,
            Command::SnapPercentTop,
            Command::SnapPercentBottom,
            Command::SnapThirdLeft,
            Command::SnapThirdMiddle,
            Command::SnapThirdRight,
        ] {
            assert_eq!(Command::from_u8(cmd.as_u8()), cmd);
        }
    }

    #[test]
    fn unknown_values_map_to_nop() {
        assert_eq!(Command::from_u8(16), Command::Nop);
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
        assert_eq!(Command::SnapTop.as_u8(), 6);
        assert_eq!(Command::SnapBottom.as_u8(), 7);
        assert_eq!(Command::MoveToNextMonitor.as_u8(), 8);
        assert_eq!(Command::SnapPercentLeft.as_u8(), 9);
        assert_eq!(Command::SnapPercentRight.as_u8(), 10);
        assert_eq!(Command::SnapPercentTop.as_u8(), 11);
        assert_eq!(Command::SnapPercentBottom.as_u8(), 12);
        assert_eq!(Command::SnapThirdLeft.as_u8(), 13);
        assert_eq!(Command::SnapThirdMiddle.as_u8(), 14);
        assert_eq!(Command::SnapThirdRight.as_u8(), 15);
    }
}
