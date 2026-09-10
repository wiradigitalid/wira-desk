# Digest: Landscape & Peripheral Ecosystem (Dimension 1)

## Findings & Extracted Claims

1. **Vendor Suite Footprint & Process Sprawl:**
   - Vendor companion suites (e.g. Logitech Options+, Razer Synapse) modernly rely on web/Chromium runtimes (Electron/CEF) and multi-process architectures.
   - Standard installations spawn 3–5 persistent background executables (`logioptionsplus_agent.exe`, `logioptionsplus_updater.exe`, `logioptionsplus_appbroker.exe`, OSD/overlay daemons).
   - Combined background RAM consumption typically ranges from 150 MB to over 500 MB, with installation footprints on disk frequently exceeding 500 MB to 1.5 GB.
   - Sources: Community benchmarks & technical user reports on [Reddit r/logitech Options+ Resource Overhead](https://www.reddit.com/r/logitech/comments/164n7o6/logi_options_high_ram_and_disk_usage/) and [Logitech Community Discussions](https://support.logi.com/).

2. **Hardware Constraints of Productivity Mice:**
   - Office/productivity mice (Logitech M-series like M590, MX Master series, Microsoft Bluetooth/Ergonomic mice, Dell Premier) consistently omit hardware on-board memory (EEPROM for keymap storage) to optimize production cost and rely on host-side OS software.
   - Gaming mice often provide on-board profiles, but productivity mice report extra physical buttons (Button 4/5, horizontal tilt wheel) strictly as standard USB HID events, requiring active OS-level software intervention to bind them to desktop workflows.
   - Class: `landscape/hardware`. Confidence: High.

3. **Enterprise & Workstation Policy Conflict:**
   - High-overhead vendor companion software with continuous background telemetries and automatic updaters creates friction in managed enterprise workstations, developer environments, and low-latency systems where strict CPU/RAM budgets and offline zero-telemetry rules apply.
