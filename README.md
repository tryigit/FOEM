<div align="center">

  <h1>FOEM</h1>

  <p><b>Free Open Ecosystem for Mobile Devices</b></p>

  <a href="https://github.com/tryigit/FOEM/releases">
    <img src="https://img.shields.io/github/v/release/tryigit/FOEM?style=for-the-badge&label=Download&color=0d1117" alt="Download Latest Release">
  </a>
  <a href="https://github.com/tryigit/FOEM">
    <img src="https://img.shields.io/github/stars/tryigit/FOEM?style=for-the-badge&color=0d1117" alt="GitHub Stars">
  </a>
  <a href="https://github.com/tryigit/FOEM/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/tryigit/FOEM/build.yml?style=for-the-badge&label=Build&color=0d1117" alt="Build Status">
  </a>

  <br><br>

  <a href="https://t.me/cleverestech">
    <img src="https://img.shields.io/badge/Telegram_Channel-Cleveres_Tech-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram Channel">
  </a>
  <a href="https://t.me/Resul2105">
    <img src="https://img.shields.io/badge/Developer-@Resul2105-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Contact Resul2105">
  </a>
  <a href="https://t.me/tryigitx">
    <img src="https://img.shields.io/badge/Developer-@tryigitx-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Contact tryigitx">
  </a>

</div>

---

## Overview

**FOEM** is a free, cross-platform application designed to remove manufacturer-imposed restrictions on mobile devices such as Bootloader (BL) locks. Built for **freedom of software** and **academic research**, FOEM gives users complete control over their own hardware.

Once you purchase a device, you should have full authority over it. FOEM breaks down artificial barriers safely and efficiently.

## Our Mission

We will prevent phone repair shops from charging fees for software through dealerships. This is a completely free application. FOEM is designed to give you back control of your device without hidden costs or gatekeeping.

## Key Features

### Bootloader Management

- Bootloader status checking and OEM unlock verification
- Bootloader unlocking with manufacturer-specific methods
- Bootloader relocking
- Fastboot device variable inspection
- Samsung Odin/Download mode, Xiaomi Mi Unlock, Huawei HiSilicon, Motorola unlock codes, Sony developer unlock, and standard fastboot OEM unlock

### Device Repair

- **IMEI Management** -- Read, backup, and write IMEI numbers using AT commands, service calls, and manufacturer-specific diagnostic methods
- **GMS Repair** -- Diagnose, clear cache, and restore Google Mobile Services (GMS, GSF, Play Store, Setup Wizard)
- **EFS Backup and Restore** -- Protect critical EFS partition data that stores IMEI, calibration, and radio configuration
- **NV Data Management** -- Backup and restore non-volatile data partitions (modemst1, modemst2, fsg, fsc)
- **DRK Repair** -- Device Root Key repair for Samsung devices (cc.dat, prov_data, ridge.dat)
- **Baseband and Modem Repair** -- Check baseband version, RIL, modem board info, and clear modem cache
- **CSC Change** -- Change Consumer Software Customization on Samsung devices (region/carrier settings)
- **Knox Counter Check** -- Verify Samsung Knox warranty trip counter status
- **Build.prop Reader** -- View critical system properties (model, manufacturer, Android version, security patch, fingerprint, hardware, bootloader)

### Network and Security

- **FRP Bypass** -- Factory Reset Protection bypass with multiple methods: ADB bypass, Setup Wizard skip, Account Manager removal, Content Provider reset
- **Google Account Removal** -- Remove Google account databases and clear GMS/GSF data for FRP preparation
- **Carrier and SIM Unlock** -- Check operator info, SIM state, network type, and apply NCK unlock codes
- **MDM Removal** -- Detect and remove Mobile Device Management profiles (Device Owner, Profile Owner, device_policies.xml)
- **Knox Bypass** -- Samsung Knox enrollment bypass by disabling Knox-related packages for the current user

### Flashing Tools

- **EDL Mode (Qualcomm 9008)** -- Enter Emergency Download mode via ADB, shell, or fastboot, with manual test-point instructions
- **EDL Flash** -- Flash via Qualcomm Sahara/Firehose protocol with chipset-specific programmer files
- **Fastboot Flash** -- Flash individual partitions (boot, recovery, system, vendor, dtbo, vbmeta, super, modem, radio, and more)
- **Partition Erase** -- Erase specific partitions via fastboot
- **vbmeta Flash** -- Flash vbmeta with disabled verification for custom ROM installations
- **Recovery Flash** -- Install custom recovery (TWRP, OrangeFox) via fastboot flash or temporary boot
- **Firmware Flash** -- Flash full stock firmware with manufacturer-specific protocols (Samsung Odin, Xiaomi fastboot, Huawei eRecovery)
- **MediaTek SP Flash** -- Enter BROM/Preloader mode, scatter file support, DA file loading
- **Reboot Modes** -- Reboot to system, recovery, bootloader, EDL, download mode, or sideload mode

### Hardware Diagnostics

- **Battery** -- Health, level, temperature, voltage, charging status, and detailed battery statistics
- **Display** -- Resolution, density, refresh rate, physical display info, and touch input axis detection
- **Sensors** -- Full sensor service dump: accelerometer, gyroscope, proximity, light, magnetometer, barometer, and more
- **Audio** -- Speaker, microphone, earpiece detection, volume streams, and audio subsystem dump
- **Cameras** -- Camera count detection, camera IDs, facing direction (front/rear)
- **Connectivity** -- WiFi, Bluetooth, GPS, and NFC availability and status
- **Biometrics** -- Fingerprint sensor HAL detection, face unlock availability
- **Storage** -- Disk usage, partition layout, primary storage UUID
- **USB** -- USB mode (MTP, ADB, PTP), controller info
- **Telephony** -- SIM state, operator, network type, phone type, data state

### Utility Tools

- **ADB Shell** -- Interactive command execution on the connected device
- **Logcat Viewer** -- Capture and display device logs, clear logcat buffer
- **File Manager** -- Pull files from device, push files to device, list directory contents
- **APK Installer** -- Sideload APK files via ADB with reinstall and downgrade support
- **Package Manager** -- List all packages, user-installed packages, or system packages with optional filtering
- **Bloatware Removal** -- Disable system apps for current user (no root required) and re-enable previously disabled apps
- **Full Backup and Restore** -- Complete device backup (apps, data, shared storage) and restore via ADB backup
- **Screenshot and Screen Recording** -- Capture device screen and pull to local machine
- **Device Reboot** -- Reboot to system, recovery, bootloader, with quick-access buttons
- **Developer Options** -- Launch device info settings for build number tap activation
- **System Info** -- Device uptime, memory info, CPU info, running process list

## Design Philosophy

The user interface and experience of FOEM are inspired by the clean, modern aesthetics of **iOS**, **NothingOS**, and similar design systems. The goal is a minimal, intuitive, and visually polished application.

## Supported Manufacturers

FOEM is committed to universal compatibility. This program supports all major smartphone manufacturers with manufacturer-specific methods and protocols:

| Manufacturer | Platform | Manufacturer | Platform |
| :--- | :--- | :--- | :--- |
| Samsung | Exynos / Qualcomm | Xiaomi / POCO / Redmi | Qualcomm / MediaTek |
| Google (Pixel) | Tensor / Qualcomm | OnePlus | Qualcomm |
| Motorola | Qualcomm / MediaTek | Sony | Qualcomm |
| LG | Qualcomm / MediaTek | Nokia | Qualcomm / MediaTek |
| Huawei | HiSilicon / Qualcomm | Honor | HiSilicon / Qualcomm |
| Oppo | Qualcomm / MediaTek | Vivo | Qualcomm / MediaTek |
| Realme | Qualcomm / MediaTek | Asus | Qualcomm |
| ZTE | Qualcomm / MediaTek | Meizu | Qualcomm / MediaTek |
| Lenovo | Qualcomm / MediaTek | Infinix | MediaTek |
| Nothing | Qualcomm | Tecno | MediaTek |

Each manufacturer has specific unlock procedures, flashing protocols, and diagnostic methods. FOEM selects the appropriate method based on the detected device and the selected manufacturer profile.

## Updates and Releases

The application includes a built-in update checker that queries the latest release from the GitHub repository. You can also download releases manually:

<p>
  <a href="https://github.com/tryigit/FOEM/releases">
    <img src="https://img.shields.io/badge/Download-Latest_Release-blue?style=for-the-badge&logo=github&logoColor=white" alt="Latest Release">
  </a>
</p>

## Getting Started

### Requirements

- Rust (stable toolchain, edition 2021)
- ADB and Fastboot installed on your system
- USB debugging enabled on your device
- Linux: libxcb, libxkbcommon, libGL, libgtk-3 (for the GUI)

### Build from Source

```
git clone https://github.com/tryigit/FOEM.git
cd FOEM
cargo build --release
```

The compiled binary will be located at `target/release/foem` (Linux) or `target\release\foem.exe` (Windows).

### Run

```
cargo run --release
```

Pre-built binaries for Linux and Windows are available on the releases page.

## Project Structure

```
FOEM/
  src/
    main.rs                  -- Application entry point
    app.rs                   -- Sidebar navigation, all feature panels, license and donate views
    theme.rs                 -- Black/white iOS/NothingOS theme constants
    license_text.rs          -- Embeds LICENSE into the binary
    diagnostics.rs           -- ADB/Fastboot device detection and health checks
    update_manager.rs        -- GitHub release update checker
    features/
      mod.rs                 -- Manufacturer enum, shared ADB/Fastboot helpers
      bootloader.rs          -- BL unlock with manufacturer-specific methods
      repair.rs              -- IMEI, GMS, EFS, NV data, DRK, baseband, CSC repair
      network.rs             -- FRP bypass, carrier unlock, MDM removal, Knox bypass
      flash.rs               -- EDL, fastboot, recovery, firmware, SP Flash, reboot modes
      hardware_test.rs       -- Battery, display, sensors, camera, audio, connectivity, biometrics
      tools.rs               -- ADB shell, logcat, file manager, APK install, backup, bloatware
  Cargo.toml                 -- Rust dependencies and build configuration
  LICENSE                    -- Non-Commercial EULA
  SECURITY.md                -- Security policy and vulnerability reporting
  .github/
    workflows/
      build.yml              -- CI build for Windows and Linux with artifact upload
  README.md
```

## Legal and Ethical Disclaimer

FOEM is intended strictly for academic purposes, independent security research, and right-to-repair initiatives. The software interacts with low-level device hardware including bootloaders, EDL modes, IMEI data, EFS partitions, and baseband processors.

THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND. The developers assume NO responsibility for any device bricking, hardware damage, data loss, voided warranties, or legal consequences resulting from the use of this software. The user assumes 100% of the risk. Certain features such as IMEI modification, FRP bypass, and bootloader unlocking may be regulated in your jurisdiction. Ensure you understand and comply with all applicable laws.

See the LICENSE file for the complete legal terms.

## Contributing

Contributions are welcome. If you have improvements, bug fixes, or feature ideas, open a pull request or submit an issue.

<p>
  <a href="https://github.com/tryigit/FOEM/issues">
    <img src="https://img.shields.io/badge/Report_Issue-GitHub-red?style=for-the-badge&logo=github&logoColor=white" alt="Report Issue">
  </a>
  <a href="https://github.com/tryigit/FOEM/pulls">
    <img src="https://img.shields.io/badge/Pull_Requests-GitHub-green?style=for-the-badge&logo=github&logoColor=white" alt="Pull Requests">
  </a>
</p>

## Support the Development

If you find this project helpful, consider supporting the development. Your contributions help maintain the project and keep it free for everyone.

### Crypto Addresses

| Asset | Network | Address |
| :--- | :--- | :--- |
| **USDT** | **TRC20** | `TQGTsbqawRHhv35UMxjHo14mieUGWXyQzk` |
| **XMR** | **Monero** | `85m61iuWiwp24g8NRXoMKdW25ayVWFzYf5BoAqvgGpLACLuMsXbzGbWR9mC8asnCSfcyHN3dZgEX8KZh2pTc9AzWGXtrEUv` |
| **USDT / USDC** | **ERC20 / BEP20** | `0x1a4b9e55e268e6969492a70515a5fd9fd4e6ea8b` |

### Platforms

<p>
  <a href="https://www.paypal.me/tryigitx">
    <img src="https://img.shields.io/badge/PayPal-Donate-003087?style=for-the-badge&logo=paypal&logoColor=white" alt="PayPal">
  </a>
  <a href="https://buymeacoffee.com/yigitx">
    <img src="https://img.shields.io/badge/Buy_Me_A_Coffee-Support-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black" alt="Buy Me a Coffee">
  </a>
</p>

- **Binance User ID:** `114574830`

## License

This project is licensed under the FOEM Non-Commercial Software License. See the [LICENSE](LICENSE) file for details. Commercial use is strictly prohibited without written permission from the Author. The Author reserves the right to issue separate commercial licenses (dual-licensing model).
