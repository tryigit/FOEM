/// Flashing operations: EDL, Fastboot, Recovery, Firmware.
///
/// Supports Qualcomm EDL (9008), MediaTek BROM/SP Flash,
/// Samsung Download/Odin mode, and standard Fastboot flashing.
use super::{adb, adb_shell, fastboot, Manufacturer};
use crate::exec::normalize_local_path;

// -- EDL (Emergency Download) Mode --

/// Reboot device into EDL mode (Qualcomm 9008).
pub fn enter_edl_mode(serial: &str) -> String {
    match crate::exec::run_with_serial("adb", serial, &["reboot", "edl"], "Failed to reboot to EDL")
    {
        Ok(res) => format!("EDL reboot initiated:\n{}", res),
        Err(e) => e,
    }
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
    "boot",
    "recovery",
    "system",
    "vendor",
    "dtbo",
    "vbmeta",
    "super",
    "userdata",
    "cache",
    "modem",
    "radio",
    "aboot",
    "sbl1",
    "rpm",
    "tz",
    "hyp",
    "keymaster",
    "cmnlib",
    "cmnlib64",
    "devcfg",
    "dsp",
    "mdtp",
];

/// Flash an image to a specific partition via fastboot.
pub fn flash_partition(serial: &str, partition: &str, image_path: &str) -> String {
    let path = normalize_local_path(image_path);
    if path.is_empty() {
        return format!("Flash {}: Image file path is required.", partition);
    }
    match fastboot(serial, &["flash", partition, &path]) {
        Ok(out) => format!("Flash {} result:\n{}", partition, out),
        Err(e) => format!(
            "Flash {} failed: {}\nEnsure device is in fastboot mode.",
            partition, e
        ),
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
    let path = normalize_local_path(image_path);
    if path.is_empty() {
        return "vbmeta path required. Use stock vbmeta.img.".to_string();
    }
    match fastboot(
        serial,
        &[
            "--disable-verity",
            "--disable-verification",
            "flash",
            "vbmeta",
            &path,
        ],
    ) {
        Ok(out) => format!("vbmeta flash (verification disabled):\n{}", out),
        Err(e) => format!("vbmeta flash failed: {}", e),
    }
}

// -- Recovery --

/// Flash a custom recovery image.
pub fn flash_recovery(serial: &str, recovery_path: &str) -> String {
    let path = normalize_local_path(recovery_path);
    if path.is_empty() {
        return "Recovery image path required (e.g., twrp.img, orangefox.img).".to_string();
    }
    match fastboot(serial, &["flash", "recovery", &path]) {
        Ok(out) => format!("Recovery flash result:\n{}", out),
        Err(e) => format!(
            "Recovery flash failed: {}\nSome A/B devices use: fastboot flash boot <recovery.img>",
            e
        ),
    }
}

/// Temporarily boot a recovery image without flashing.
pub fn boot_recovery_temp(serial: &str, recovery_path: &str) -> String {
    let path = normalize_local_path(recovery_path);
    if path.is_empty() {
        return "Recovery image path required.".to_string();
    }
    match fastboot(serial, &["boot", &path]) {
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
        Ok(out) => format!(
            "Reboot to '{}': {}",
            mode,
            if out.is_empty() { "OK" } else { &out }
        ),
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
        Ok(_) => "MediaTek BROM Mode:\n\
             Device rebooting to preloader/BROM.\n\
             For manual entry: Power off, hold Vol Up + connect USB.\n\
             Device should appear as MediaTek USB Port."
            .to_string(),
        Err(_) => "MediaTek BROM Mode:\n\
             Could not reboot via ADB.\n\
             Manual method: Power off, hold Vol Up + Vol Down + connect USB."
            .to_string(),
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

// -- Root Installation (Magisk / KernelSU) --

/// Install Magisk via APK, ZIP, or patched boot image.
pub fn install_magisk(serial: &str, path: &str) -> String {
    let path = normalize_local_path(path);
    if path.is_empty() {
        return "Magisk file path required. Provide .apk (Manager), .zip (Sideload), or .img (Patched Boot).".to_string();
    }
    if path.ends_with(".apk") {
        match adb(serial, &["install", "-r", "-d", &path]) {
            Ok(out) => format!("Magisk Manager APK install:\n{}", out),
            Err(e) => format!("Magisk APK install failed: {}", e),
        }
    } else if path.ends_with(".zip") {
        match adb(serial, &["sideload", &path]) {
            Ok(out) => format!("Magisk ZIP sideload:\n{}", out),
            Err(e) => format!(
                "Magisk sideload failed (ensure device is in ADB sideload mode): {}",
                e
            ),
        }
    } else if path.ends_with(".img") {
        match fastboot(serial, &["flash", "boot", &path]) {
            Ok(out) => format!("Magisk patched boot flash:\n{}", out),
            Err(e) => format!("Magisk boot flash failed: {}", e),
        }
    } else {
        "Unsupported file type. Use .apk, .zip, or .img.".to_string()
    }
}

/// Install KernelSU via APK, ZIP, or patched boot image.
pub fn install_kernelsu(serial: &str, path: &str) -> String {
    let path = normalize_local_path(path);
    if path.is_empty() {
        return "KernelSU file path required. Provide .apk (Manager), .zip (Sideload), or .img (Patched Boot).".to_string();
    }
    if path.ends_with(".apk") {
        match adb(serial, &["install", "-r", "-d", &path]) {
            Ok(out) => format!("KernelSU Manager APK install:\n{}", out),
            Err(e) => format!("KernelSU APK install failed: {}", e),
        }
    } else if path.ends_with(".zip") {
        match adb(serial, &["sideload", &path]) {
            Ok(out) => format!("KernelSU AnyKernel3 ZIP sideload:\n{}", out),
            Err(e) => format!(
                "KernelSU sideload failed (ensure device is in ADB sideload mode): {}",
                e
            ),
        }
    } else if path.ends_with(".img") {
        match fastboot(serial, &["flash", "boot", &path]) {
            Ok(out) => format!("KernelSU patched boot flash:\n{}", out),
            Err(e) => format!("KernelSU boot flash failed: {}", e),
        }
    } else {
        "Unsupported file type. Use .apk, .zip, or .img.".to_string()
    }
}
#[cfg(test)]
mod tests {
    use crate::exec::MOCK_RUN_IMPL;
    use crate::features::flash::enter_edl_mode;

    #[test]
    fn test_enter_edl_mode_success() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(|program, args, error_prefix| {
                assert_eq!(program, "adb");
                assert_eq!(args, &["-s", "SERIAL123", "reboot", "edl"]);
                assert_eq!(error_prefix, "Failed to reboot to EDL");
                Ok("Device rebooting to EDL...".to_string())
            }));
        });

        let result = enter_edl_mode("SERIAL123");
        assert_eq!(result, "EDL reboot initiated:\nDevice rebooting to EDL...");

        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_enter_edl_mode_failure() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(|_, _, _| {
                Err("Failed to reboot to EDL: error".to_string())
            }));
        });

        let result = enter_edl_mode("SERIAL123");
        assert_eq!(result, "Failed to reboot to EDL: error");

        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }
}
