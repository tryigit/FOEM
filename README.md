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

- **Bootloader (BL) Unlocking** -- Circumvent BL locks to allow custom firmware installation, root access, and low-level system modifications.
- **ADB and Fastboot Integration** -- Full device management, debugging, and flashing capabilities through ADB and Fastboot.
- **EDL (Emergency Download) Support** -- Low-level device flashing and unbricking through emergency protocols such as Qualcomm EDL mode.
- **GMS Repair** -- Diagnose and restore Google Mobile Services on devices where GMS is broken or missing.
- **Device Diagnostics** -- Deep-level insights into device partitions, boot states, and OEM parameters.
- **Repair Shop Utilities** -- Essential tools commonly used by professional phone repair shops to diagnose, service, and restore devices.
- **Built-in Update Checker** -- Automatic update checking against the latest GitHub release.
- **100% Free** -- No premium tiers, no hidden fees. Completely free and open.

## Design Philosophy

The user interface and experience of FOEM are inspired by the clean, modern aesthetics of **iOS**, **NothingOS**, and similar design systems. The goal is a minimal, intuitive, and visually polished application.

## Supported Manufacturers

FOEM is committed to universal compatibility. This program will support all major smartphone manufacturers:

| Manufacturer | Manufacturer | Manufacturer |
| :--- | :--- | :--- |
| Samsung | Xiaomi / POCO / Redmi | Google (Pixel) |
| OnePlus | Motorola | Sony |
| LG | Nokia | Huawei |
| Infinix | Oppo | Vivo |
| Realme | Asus | ZTE |
| Meizu | Lenovo | Honor |

All listed manufacturers and their sub-brands will be fully supported.

## Updates and Releases

The application includes a built-in update checker that queries the latest release from the GitHub repository. You can also download releases manually:

<p>
  <a href="https://github.com/tryigit/FOEM/releases">
    <img src="https://img.shields.io/badge/Download-Latest_Release-blue?style=for-the-badge&logo=github&logoColor=white" alt="Latest Release">
  </a>
</p>

## Getting Started

### Requirements

- Python 3.10 or later
- ADB and Fastboot installed on your system
- USB debugging enabled on your device

### Installation

```
git clone https://github.com/tryigit/FOEM.git
cd FOEM
pip install -r requirements.txt
python -m src.main
```

Detailed setup instructions and platform-specific guides will be added as the project matures.

## Project Structure

```
FOEM/
  src/
    __init__.py
    main.py           -- Application entry point
    diagnostics.py    -- Device detection and health checks
    gms_repair.py     -- GMS repair logic
    update_manager.py -- Update checking against GitHub releases
    ui/
      __init__.py
      app.py          -- Main application window
  .github/
    workflows/
      build.yml       -- CI build for Windows and Linux
  requirements.txt
  README.md
```

## Legal and Ethical Disclaimer

FOEM is intended strictly for academic purposes, independent research, and right-to-repair initiatives. The developers assume no responsibility for any damage caused to your device, including soft/hard bricking, loss of data, or voiding of warranties. Proceed with caution and ensure you understand the risks associated with modifying bootloaders and low-level firmware.

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

This project is licensed under the MIT License. See the LICENSE file for details.
