# WinTick Project Conventions

This companion document outlines the structural conventions for configuration files and build automation as part of the WinTick contract.

## 1. Local Configuration (TOML)

WinTick loads settings from a local configuration file named `config.toml` situated in a safe, user-writable directory (e.g., `%APPDATA%\WinTick`) to avoid permission issues when running elevated.

### Configuration Schema Example

```toml
[general]
# Auto-start WinTick on Windows boot (Task Scheduler highest privileges)
auto_start = true

[switcher]
# Main same-app switcher shortcut
# Supported modifiers: win, alt, ctrl, shift
# Supported keys: backtick, tab, etc.
shortcut = "win+backtick"
fallback_shortcut = "alt+backtick"

[snapping]
# Snap layout shortcuts
snap_half_left = "ctrl+win+left"
snap_half_right = "ctrl+win+right"
snap_maximize = "ctrl+win+enter"

[layout]
# Enable overlapping stack layout for small screens (P2 feature)
enable_overlapping_stack = false
stack_width_percent = 50

[vm_bypass]
# Processes whose active window causes WinTick to forward input unmodified.
# Add custom VM/RDP process names here.
bypass_processes = [
  "mstsc.exe",
  "vmconnect.exe",
  "vmware.exe",
  "VirtualBoxVM.exe",
  "MobaXterm.exe",
]
```

## 2. Local Build Automation (PowerShell)

To enable closed-source local builds, a PowerShell script named `build.ps1` must be provided in the project root.

### Build Script Requirements

The `build.ps1` script must support the following functionality:
- **Development Build**:
  - Command: `.\build.ps1 -Mode dev`
  - Purpose: Compiles a debug version with local logging enabled.
- **Production Build**:
  - Command: `.\build.ps1 -Mode prod`
  - Purpose: Compiles a release version using `windows-sys` crate with aggressive profile (`lto=true`, `opt-level="z"`, `strip=true`, `panic="abort"`), targeting binary size <500KB and memory footprint <2MB RAM. Runs headless without spawning a console window.
- **Dependency Check**: Verifies that the Rust toolchain (Cargo) is installed and available before compiling.
