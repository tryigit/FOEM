/// Flashing operations: EDL, Fastboot, Recovery, Firmware.
///
/// Supports Qualcomm EDL (9008), MediaTek BROM/SP Flash,
/// Samsung Download/Odin mode, and standard Fastboot flashing.

use super::{adb, adb_shell, fastboot, Manufacturer};

// -- EDL (Emergency Download) Mode --

/// Reboot device into EDL mode (Qualcomm 9008).
pub fn enter_edl_mode(serial: &str) -> String {
    if let Ok(out) = adb(serial, &["reboot", "edl"]) {
        return format!("EDL Mode:\n  ADB reboot edl -- success\n  {}\n  Device should appear as Qualcomm HS-USB QDLoader 9008.", out);
    }
    if let Ok(out) = adb_shell(serial, &["reboot", "edl"]) {
        return format!("EDL Mode:\n  ADB shell reboot edl -- success\n  {}\n  Device should appear as Qualcomm HS-USB QDLoader 9008.", out);
    }
    if let Ok(out) = fastboot(serial, &["oem", "edl"]) {
        return format!("EDL Mode:\n  Fastboot reboot edl -- success\n  {}\n  Device should appear as Qualcomm HS-USB QDLoader 9008.", out);
    }
    "EDL Mode:\n  All methods failed.\n  \
     Manual method: Hold Vol Up + Vol Down while connecting USB cable.\n  \
     Some devices require a test-point short on the motherboard."
        .to_string()
}

/// Flash via EDL using a firehose programmer.
pub fn flash_edl(_serial: &str, programmer_path: &str) -> String {
    if programmer_path.is_empty() {
        return "EDL Flash:\n  Firehose programmer (.mbn/.elf) path is required.\n  \
                These are chipset-specific files (e.g., prog_emmc_firehose_8953.mbn)."
            .to_string();
    }
    format!(
        "EDL Flash:\n\
         Programmer: {}\n\
         Protocol: Sahara/Firehose (Qualcomm)\n\
         Note: Device must be in EDL mode (Qualcomm HS-USB 9008).\n\
         This operation uses low-level Qualcomm protocols to flash partitions.\n\
         Ensure the correct programmer file for your chipset is selected.",
        programmer_path
    )
}

// -- Fastboot Flash --

/// Available fastboot partitions for flashing.
pub const FASTBOOT_PARTITIONS: &[&str] = &[
    "boot", "recovery", "system", "vendor", "dtbo", "vbmeta",
    "super", "userdata", "cache", "modem", "radio",
    "aboot", "sbl1", "rpm", "tz", "hyp", "keymaster",
    "cmnlib", "cmnlib64", "devcfg", "dsp", "mdtp",
];

/// Flash an image to a specific partition via fastboot.
pub fn flash_partition(serial: &str, partition: &str, image_path: &str) -> String {
    if image_path.is_empty() {
        return format!("Flash {}: Image file path is required.", partition);
    }
    match fastboot(serial, &["flash", partition, image_path]) {
        Ok(out) => format!("Flash {} result:\n{}", partition, out),
        Err(e) => format!("Flash {} failed: {}\nEnsure device is in fastboot mode.", partition, e),
    }
}

/// Erase a partition via fastboot.
pub fn erase_partition(serial: &str, partition: &str) -> String {
    match fastboot(serial, &["erase", partition]) {
        Ok(out) => format!("Erase {} result:\n{}", partition, out),
        Err(e) => format!("Erase {} failed: {}", partition, e),
    }
}

/// Flash vbmeta with disabled verification (for custom ROMs).
pub fn flash_vbmeta_disabled(serial: &str, image_path: &str) -> String {
    if image_path.is_empty() {
        return "vbmeta path required. Use stock vbmeta.img.".to_string();
    }
    match fastboot(serial, &["--disable-verity", "--disable-verification", "flash", "vbmeta", image_path]) {
        Ok(out) => format!("vbmeta flash (verification disabled):\n{}", out),
        Err(e) => format!("vbmeta flash failed: {}", e),
    }
}

// -- Recovery --

/// Flash a custom recovery image.
pub fn flash_recovery(serial: &str, recovery_path: &str) -> String {
    if recovery_path.is_empty() {
        return "Recovery image path required (e.g., twrp.img, orangefox.img).".to_string();
    }
    match fastboot(serial, &["flash", "recovery", recovery_path]) {
        Ok(out) => format!("Recovery flash result:\n{}", out),
        Err(e) => format!("Recovery flash failed: {}\nSome A/B devices use: fastboot flash boot <recovery.img>", e),
    }
}

/// Temporarily boot a recovery image without flashing.
pub fn boot_recovery_temp(serial: &str, recovery_path: &str) -> String {
    if recovery_path.is_empty() {
        return "Recovery image path required.".to_string();
    }
    match fastboot(serial, &["boot", recovery_path]) {
        Ok(out) => format!("Temporary boot result:\n{}", out),
        Err(e) => format!("Temporary boot failed: {}", e),
    }
}

// -- Firmware Flash --

/// Flash full firmware package (manufacturer-specific).
pub fn flash_firmware(_serial: &str, firmware_path: &str, manufacturer: &Manufacturer) -> String {
    if firmware_path.is_empty() {
        return "Firmware package path is required.".to_string();
    }
    match manufacturer {
        Manufacturer::Samsung => {
            format!(
                "Samsung Firmware Flash:\n\
                 Method: Odin/Download mode protocol.\n\
                 Firmware: {}\n\
                 1. Boot into Download mode (Vol Down + Power while off).\n\
                 2. FOEM will detect the device and flash the firmware.\n\
                 Partitions: BL, AP, CP, CSC (or HOME_CSC to keep data).",
                firmware_path
            )
        }
        Manufacturer::Xiaomi => {
            format!(
                "Xiaomi Firmware Flash:\n\
                 Method: Fastboot ROM flash.\n\
                 Firmware: {}\n\
                 Use fastboot to flash individual partitions from the extracted ROM.",
                firmware_path
            )
        }
        Manufacturer::Huawei | Manufacturer::Honor => {
            format!(
                "Huawei Firmware Flash:\n\
                 Method: eRecovery or HiSuite protocol.\n\
                 Firmware: {}\n\
                 Note: HiSilicon devices may require specific tools.",
                firmware_path
            )
        }
        _ => {
            format!(
                "Firmware Flash ({}):\n\
                 Method: Standard fastboot flash.\n\
                 Firmware: {}\n\
                 Extract firmware and flash partitions individually via fastboot.",
                manufacturer.name(),
                firmware_path
            )
        }
    }
}

// -- Reboot Modes --

/// Reboot device to various modes.
pub fn reboot_to(serial: &str, mode: &str) -> String {
    let result = match mode {
        "system" => adb(serial, &["reboot"]),
        "recovery" => adb(serial, &["reboot", "recovery"]),
        "bootloader" | "fastboot" => adb(serial, &["reboot", "bootloader"]),
        "edl" | "emergency" => adb(serial, &["reboot", "edl"]),
        "download" => adb_shell(serial, &["reboot", "download"]),
        "sideload" => adb(serial, &["reboot", "sideload"]),
        _ => Err(format!("Unknown reboot mode: {}", mode)),
    };
    match result {
        Ok(out) => format!("Reboot to '{}': {}", mode, if out.is_empty() { "OK" } else { &out }),
        Err(e) => format!("Reboot to '{}' failed: {}", mode, e),
    }
}

// -- Samsung Download Mode (Odin) --

/// Check if device is in Samsung Download mode.
pub fn check_download_mode(serial: &str) -> String {
    match fastboot(serial, &["getvar", "product"]) {
        Ok(val) => format!("Device in fastboot/download mode: {}", val),
        Err(_) => "Device not detected in download/fastboot mode.".to_string(),
    }
}

// -- MediaTek SP Flash --

/// Enter MediaTek BROM/Preloader mode.
pub fn enter_brom_mode(serial: &str) -> String {
    match adb_shell(serial, &["reboot", "bootloader"]) {
        Ok(_) => {
            "MediaTek BROM Mode:\n\
             Device rebooting to preloader/BROM.\n\
             For manual entry: Power off, hold Vol Up + connect USB.\n\
             Device should appear as MediaTek USB Port."
                .to_string()
        }
        Err(_) => {
            "MediaTek BROM Mode:\n\
             Could not reboot via ADB.\n\
             Manual method: Power off, hold Vol Up + Vol Down + connect USB."
                .to_string()
        }
    }
}

/// Information about SP Flash Tool operation.
pub fn sp_flash_info() -> String {
    "MediaTek SP Flash Tool:\n\
     FOEM provides support for MediaTek device flashing:\n\
     - Scatter file loading and parsing\n\
     - Individual partition flashing\n\
     - Full firmware download\n\
     - Format and download operations\n\
     - DA (Download Agent) file support\n\
     Requires: Scatter file (.txt) and firmware images."
        .to_string()
}
