# FOEM Architecture and Security

This document provides a detailed technical breakdown of the FOEM application architecture, its multi-threading model, USB communication abstractions, and the safety protocols designed to prevent data loss and device bricking.

---

## Table of Contents

1. [Architectural Overview](#architectural-overview)
2. [Technology Stack](#technology-stack)
3. [Module Structure](#module-structure)
4. [Multi-Threading Model](#multi-threading-model)
5. [USB Communication Abstractions](#usb-communication-abstractions)
6. [Auto-Backup Protocol](#auto-backup-protocol)
7. [Brick Prevention and Data Integrity](#brick-prevention-and-data-integrity)
8. [Localization Architecture](#localization-architecture)
9. [Security Considerations](#security-considerations)

---

## Architectural Overview

FOEM follows a modular, layered architecture designed to separate concerns between the user interface, business logic, device communication, and safety enforcement layers.

```
+-------------------------------------------------------------+
|                      User Interface Layer                    |
|          (eframe / egui -- immediate mode GUI)               |
+-------------------------------------------------------------+
|                     Application Logic Layer                  |
|  (Feature Modules: bootloader, repair, network, flash, etc) |
+-------------------------------------------------------------+
|                   Safety Enforcement Layer                   |
|     (Auto-Backup, CRC Validation, Timeout Management)       |
+-------------------------------------------------------------+
|                  Device Communication Layer                  |
|        (ADB, Fastboot, EDL/Sahara/Firehose, BROM,           |
|         Serial/Diag Ports, libusb abstractions)              |
+-------------------------------------------------------------+
|                     Operating System / USB                   |
+-------------------------------------------------------------+
```

Each layer communicates strictly downward. The UI layer never issues raw USB commands directly. All device I/O passes through the communication layer, which is wrapped by the safety enforcement layer to guarantee backup and validation policies.

---

## Technology Stack

| Component              | Technology                                            |
| :--------------------- | :---------------------------------------------------- |
| Language               | Rust (2021 edition, stable toolchain)                 |
| GUI Framework          | eframe 0.29 / egui (immediate mode rendering)         |
| Serialization          | serde + serde_json                                    |
| HTTP Client            | ureq (for update checking)                            |
| Device Communication   | ADB / Fastboot CLI wrappers, serial port access       |
| USB Protocols          | Qualcomm EDL (Sahara/Firehose), MediaTek BROM/Preloader, Samsung Odin/Download |
| Build System           | Cargo (with release optimizations: LTO, size-opt, strip) |
| CI/CD                  | GitHub Actions (Windows and Linux artifact builds)     |
| Localization           | JSON-based i18n with runtime locale loading           |

---

## Module Structure

The source code is organized into a flat feature-module layout under `src/`:

```
src/
  main.rs                -- Application entry point and eframe initialization.
  app.rs                 -- Central application state, sidebar navigation, panel routing,
                            license and donation views. Owns the main update/render loop.
  theme.rs               -- Visual theme constants (colors, spacing, font sizes) following
                            a black/white aesthetic inspired by iOS and NothingOS.
  license_text.rs        -- Embeds the LICENSE file contents into the binary at compile
                            time for display in the About panel.
  diagnostics.rs         -- ADB and Fastboot device detection, connection health checks,
                            and device state enumeration (ADB, Fastboot, EDL, BROM).
  update_manager.rs      -- Queries the GitHub Releases API for new versions and presents
                            update notifications in the UI.
  features/
    mod.rs               -- Manufacturer enum definitions, shared ADB/Fastboot command
                            helpers, and common utility functions used across all modules.
    bootloader.rs        -- Bootloader status checking, OEM unlock verification, unlock
                            and relock operations with manufacturer-specific method dispatch.
    repair.rs            -- IMEI read/write/backup, GMS diagnostics, EFS backup/restore/wipe,
                            NV data and QCN management, DRK repair, baseband/modem repair,
                            CSC changes, Knox counter checks, build.prop reading.
    network.rs           -- FRP bypass (multiple methods), Google account removal, carrier
                            and SIM unlock, MDM detection/removal, Knox bypass.
    flash.rs             -- EDL mode entry and flashing (Sahara/Firehose), Fastboot partition
                            flash/erase, vbmeta flashing, recovery installation, full firmware
                            flash with manufacturer protocols, MediaTek SP Flash (BROM/scatter),
                            reboot mode selection, partition manager, root maker (Magisk).
    hardware_test.rs     -- Battery, display, sensor, audio, camera, connectivity, biometric,
                            storage, USB, and telephony diagnostics via ADB dumpsys.
    tools.rs             -- ADB shell, logcat viewer, file manager (push/pull), APK installer,
                            package manager, bloatware removal, full backup/restore, screenshot
                            and screen recording, developer options, system info.
```

### Module Communication Pattern

Each feature module exposes a struct containing its internal state and a public `ui()` method that receives a mutable reference to `egui::Ui`. The central `app.rs` module routes to the appropriate feature panel based on the active sidebar selection.

Feature modules that perform device I/O invoke shared helper functions defined in `features/mod.rs`, which wrap `std::process::Command` calls to ADB and Fastboot binaries. This indirection ensures consistent error handling, timeout enforcement, and logging across all modules.

---

## Multi-Threading Model

Responsiveness is a non-negotiable requirement. All USB I/O and device communication must execute on background threads. The GUI thread must never block on a device operation.

### Thread Architecture

```
+------------------+       +------------------------+
|   GUI Thread     | <---> |   Shared State (Arc)   |
| (egui render     |       |   - operation_status   |
|  loop, 60fps)    |       |   - result_buffer      |
+------------------+       |   - progress_value     |
                           |   - cancel_flag        |
                           +------------------------+
                                      ^
                                      |
                           +------------------------+
                           | Worker Thread(s)       |
                           | - ADB/Fastboot calls   |
                           | - EDL protocol I/O     |
                           | - BROM communication   |
                           | - File read/write      |
                           +------------------------+
```

### Implementation Pattern

1. **Thread Spawning.** When the user initiates a device operation (flash, erase, backup, etc.), the feature module spawns a new thread using `std::thread::spawn`. The spawned thread receives a clone of an `Arc<Mutex<SharedState>>` struct that holds the operation status, result buffer, and a cancellation flag.

2. **UI Polling.** On each frame, the GUI thread locks the shared state briefly (non-blocking try_lock preferred) to read the current operation status and display progress. This ensures the render loop is never stalled by a long-running device operation.

3. **Result Delivery.** When the background thread completes, it writes the result (success, failure, or partial output) into the shared state buffer and sets the status to `Completed` or `Failed`. The GUI thread picks this up on the next frame.

4. **Cancellation.** The shared state includes an `AtomicBool` cancellation flag. The GUI thread can set this flag to `true` when the user requests cancellation. The worker thread checks this flag at safe checkpoints (between partition writes, between protocol stages) and aborts gracefully if set.

### Thread Safety Guarantees

- All shared state is accessed through `Arc<Mutex<T>>` or `Arc<AtomicBool>`.
- Worker threads do not directly interact with `egui` or any GUI types.
- Long-running operations are divided into stages with explicit yield points for cancellation checks.
- Thread panics are caught at the boundary to prevent application crashes.

---

## USB Communication Abstractions

FOEM supports multiple device communication protocols, each abstracted behind a common interface pattern.

### Protocol Handlers

| Protocol                 | Transport        | Use Case                                    |
| :----------------------- | :--------------- | :------------------------------------------ |
| ADB                      | USB / TCP        | General device management, file operations   |
| Fastboot                 | USB              | Bootloader operations, partition flashing     |
| EDL (Sahara + Firehose)  | USB (Qualcomm 9008) | Qualcomm emergency download mode          |
| BROM / Preloader         | USB (Serial)     | MediaTek low-level flash mode                |
| Odin / Download          | USB              | Samsung firmware flash mode                  |
| Diag (QCDM)             | Serial / USB     | Qualcomm diagnostic port for NV/EFS/QCN      |

### ADB and Fastboot Wrapper

The primary communication layer wraps the `adb` and `fastboot` command-line binaries using `std::process::Command`. This approach provides broad device compatibility without requiring custom USB drivers in user space.

```
Command Execution Flow:
  1. Validate binary exists on PATH or in configured location.
  2. Build argument list for the specific operation.
  3. Set timeout duration based on operation type.
  4. Spawn process on worker thread.
  5. Read stdout/stderr with timeout.
  6. Parse output and return structured result.
```

Timeout values are calibrated per operation type:

| Operation Category        | Default Timeout | Rationale                                    |
| :------------------------ | :-------------- | :------------------------------------------- |
| Device detection          | 5 seconds       | Fast probe; should not delay UI startup       |
| Property read             | 10 seconds      | Single getprop or dumpsys call                |
| Partition flash           | 300 seconds     | Large images (system, super) take time        |
| Full firmware flash       | 600 seconds     | Multi-partition sequential write              |
| EFS/NV backup             | 60 seconds      | Small partitions but include dd + pull        |
| EDL Firehose session      | 120 seconds     | Protocol handshake and negotiation phase      |
| BROM handshake            | 30 seconds      | Initial serial handshake with MediaTek ROM    |

### EDL Protocol Handler

For Qualcomm devices in EDL mode (USB VID:PID 05C6:9008), the communication follows a two-stage protocol:

1. **Sahara Stage.** The host sends a programmer binary (e.g., `prog_firehose_*.mbn`) to the device. The Sahara protocol negotiates the transfer using a series of hello/command/data packets.

2. **Firehose Stage.** Once the programmer is loaded and executing on the device, the host sends XML-formatted commands over the USB bulk pipe to read/write/erase partitions. The Firehose protocol parses `rawprogram.xml` and `patch0.xml` to determine partition layout and flash sequences.

### BROM / Preloader Handler

For MediaTek devices, BROM (Boot ROM) mode communication uses serial-over-USB:

1. **Handshake.** The host sends a sequence of `0xA0` bytes to synchronize with the device BROM.
2. **Download Agent (DA).** A Download Agent binary is uploaded to device RAM and executed.
3. **Scatter File Parsing.** The scatter file defines partition layout, load addresses, and region types. The handler parses this to build the flash plan.
4. **Flash Execution.** Data is written partition-by-partition per the scatter file specification.

---

## Auto-Backup Protocol

The auto-backup system is the first line of defense against data loss. It is enforced at the safety layer and cannot be bypassed through the UI.

### Policy

Before any operation classified as destructive, the system automatically creates a backup of the affected data. Destructive operations include:

| Operation               | Data Backed Up                              |
| :---------------------- | :------------------------------------------ |
| EFS Wipe                | Full EFS partition image                     |
| NV Data Write/Restore   | Current modemst1, modemst2, fsg, fsc images  |
| QCN Write               | Current QCN file dump                        |
| Partition Erase         | Partition image (if readable)                |
| Partition Flash         | Current partition image (if readable)        |
| Bootloader Unlock       | EFS and NV data (as unlock may trigger wipe) |
| Full Firmware Flash     | EFS, NV data, and current boot image         |
| DRK Repair              | Current DRK files (cc.dat, prov_data, ridge.dat) |
| Root Maker (boot patch) | Original unpatched boot.img                  |

### Backup Workflow

```
User Initiates Destructive Operation
  |
  v
Safety Layer Intercept
  |
  v
Check: Is auto-backup enabled? (Default: YES, always recommended)
  |
  +-- YES --> Create timestamped backup directory
  |              <backup_root>/<device_serial>/<YYYY-MM-DD_HH-MM-SS>/
  |           Write affected partition/data images
  |           Verify backup integrity (file size > 0, optional CRC)
  |           Log backup path to operation log
  |              |
  |              v
  |           Proceed with destructive operation
  |
  +-- NO  --> Display strong warning dialog
              "Auto-backup is disabled. Data loss may be irreversible."
              Require explicit user confirmation (type "I UNDERSTAND")
                 |
                 v
              Proceed with destructive operation
```

### Backup Storage

Backups are stored in a user-configurable directory (default: `<user_home>/FOEM_Backups/`). The directory structure uses device serial numbers and timestamps to prevent collisions and maintain traceability:

```
FOEM_Backups/
  <device_serial>/
    2026-03-15_18-30-00/
      efs.img
      modemst1.img
      modemst2.img
      fsg.img
      backup_manifest.json
    2026-03-15_19-45-00/
      boot.img
      backup_manifest.json
```

The `backup_manifest.json` file records the operation that triggered the backup, the files included, their sizes, CRC32 checksums, and the device information at the time of backup.

---

## Brick Prevention and Data Integrity

Beyond auto-backups, FOEM implements several active measures to prevent device bricking during critical operations.

### CRC Validation

All data transfers to and from the device are validated using CRC32 checksums where the protocol supports it:

- **EDL Firehose:** CRC validation is built into the Firehose XML protocol. Each data segment includes a CRC field that the device validates before committing to flash.
- **BROM/DA:** The Download Agent protocol includes checksum verification at the block level during scatter-based flashing.
- **Fastboot:** For partition flashing, the source image CRC is computed before transfer and logged. Post-flash verification reads back a hash from the device (where supported) for comparison.
- **EFS/NV Backup:** Backup files are CRC-verified immediately after creation. If the CRC check fails, the backup is marked as invalid and the destructive operation is blocked.

### Timeout Management

Aggressive timeout handling prevents hung operations from leaving the device in an inconsistent state:

1. **Per-Command Timeouts.** Every ADB/Fastboot/serial command has a timeout appropriate to its expected duration (see the timeout table in the USB Communication section).

2. **Stage Timeouts.** Multi-stage operations (EDL Sahara-to-Firehose transition, BROM handshake-to-DA-load) have per-stage timeouts. If any stage exceeds its timeout, the entire operation is aborted cleanly.

3. **Watchdog Timer.** Long-running flash operations use a watchdog pattern: the worker thread must report progress at regular intervals. If no progress is reported within the watchdog window (configurable, default 60 seconds), the operation is flagged as potentially stalled and the user is alerted.

4. **Graceful Abort.** On timeout, the system attempts a graceful abort sequence appropriate to the active protocol:
   - ADB/Fastboot: Kill the subprocess and reboot the device to a safe state if possible.
   - EDL: Send a Firehose reset command before disconnecting.
   - BROM: Send a DA reset command and release the serial port.

### Flash Sequence Safety

For multi-partition firmware flashing operations, additional protections are in place:

1. **Dry Run Validation.** Before beginning the flash, the system parses all firmware manifests (`rawprogram.xml`, scatter files) and validates that all referenced files exist and are non-zero in size.

2. **Sequential Ordering.** Partitions are flashed in a safe order. Critical partitions (bootloader, modem) are flashed last to minimize the risk of a partial flash leaving the device unbootable.

3. **Power Loss Awareness.** The UI displays persistent warnings during flash operations reminding the user not to disconnect the device or interrupt the process.

4. **Rollback Information.** The auto-backup taken before the flash operation provides the data needed for manual rollback if the flash fails partway through.

---

## Localization Architecture

The localization system uses JSON locale files stored in the `locales/` directory. The architecture is designed for simplicity, runtime flexibility, and community contribution.

### Runtime Loading

1. On startup, the application reads the user's language preference from the settings configuration.
2. The corresponding locale file is loaded from `locales/<code>.json`.
3. If the file does not exist or cannot be parsed, the system falls back to `locales/en.json`.
4. Locale data is deserialized into a nested `HashMap<String, serde_json::Value>` structure.
5. String lookups use dot-notation keys (e.g., `repair.efs.backup`) that are resolved by walking the nested map.

### Fallback Chain

```
Requested Key --> Active Locale File
                      |
                      +-- Found --> Return translated string
                      |
                      +-- Not Found --> English Locale File (en.json)
                                            |
                                            +-- Found --> Return English string
                                            |
                                            +-- Not Found --> Return key path as-is
                                                             (indicates a bug)
```

### Adding New Translatable Strings

When adding a new feature that requires user-facing text:

1. Add the English string to `locales/en.json` under the appropriate section.
2. Use the localization lookup function in the UI code instead of hardcoded strings.
3. Document the new keys in the PR so translators can update their locale files.

See `TRANSLATION_GUIDE.md` for the complete contributor guide.

---

## Security Considerations

### Local-Only Communication

All device communication occurs locally over USB. FOEM does not transmit device data, IMEI information, EFS contents, or any other device-specific data to any remote server. The only network request the application makes is the optional GitHub Releases API query for update checking, which can be disabled in settings.

### No Telemetry

FOEM contains no telemetry, analytics, crash reporting, or tracking of any kind. The application does not collect or store personal information.

### Dependency Hygiene

All third-party dependencies are:
- Sourced from crates.io (the official Rust package registry).
- Checked against the GitHub Advisory Database before inclusion.
- Kept to a minimal set to reduce attack surface.

Current dependency count is intentionally low (eframe, serde, serde_json, ureq, open) to minimize supply chain risk.

### Binary Distribution

Release builds are compiled with:
- `opt-level = "s"` (size optimization).
- `lto = true` (link-time optimization, also reduces binary surface).
- `strip = true` (debug symbols removed).

Builds are produced via GitHub Actions CI with full build logs available for audit.

---

For questions about the architecture, security model, or implementation details, open an issue on the GitHub repository or refer to the inline documentation in the source files.
