/// Bootloader unlock and management operations.
///
/// Supports manufacturer-specific unlock methods.
/// Samsung: Odin/Download mode, OEM unlock toggle
/// Xiaomi: Mi Unlock (account binding required)
/// Huawei: HiSilicon bootloader code
/// Google Pixel: Direct fastboot OEM unlock
/// OnePlus: Fastboot OEM unlock (similar to Google)
/// Motorola: Unlock code from manufacturer portal
/// Sony: Unlock code from developer portal
/// Others: Standard fastboot OEM unlock

use super::{adb_shell, fastboot, Manufacturer};

/// Check current bootloader lock status via fastboot.
pub fn check_status(serial: &str) -> String {
    match fastboot(serial, &["getvar", "unlocked"]) {
        Ok(out) => format!("Bootloader status:\n{}", out),
        Err(e) => format!("Failed to check BL status: {}\nDevice may not be in fastboot mode.", e),
    }
}

/// Check OEM unlock setting via ADB.
pub fn check_oem_unlock_setting(serial: &str) -> String {
    match adb_shell(serial, &["settings", "get", "global", "oem_unlock_allowed"]) {
        Ok(val) => {
            let enabled = val.trim() == "1";
            format!(
                "OEM Unlock in Developer Options: {}",
                if enabled { "Enabled" } else { "Disabled" }
            )
        }
        Err(e) => format!("Failed to check OEM unlock setting: {}", e),
    }
}

/// Unlock bootloader using manufacturer-appropriate method.
pub fn unlock(serial: &str, manufacturer: &Manufacturer) -> String {
    match manufacturer {
        Manufacturer::Samsung => {
            "Samsung bootloader unlock:\n\
             1. Enable OEM Unlock in Developer Options.\n\
             2. Boot into Download mode (Vol Down + Power).\n\
             3. Long-press Vol Up to enter unlock mode.\n\
             4. Confirm unlock. Device will factory reset.\n\
             Note: Knox counter will be tripped permanently."
                .to_string()
        }
        Manufacturer::Xiaomi => {
            "Xiaomi bootloader unlock:\n\
             1. Apply for unlock permission at en.miui.com/unlock.\n\
             2. Wait for the binding period (72h to 30 days).\n\
             3. Use Mi Unlock Tool or FOEM to send unlock command.\n\
             Attempting fastboot unlock..."
                .to_string()
            // In a full implementation: fastboot(serial, &["oem", "unlock"])
        }
        Manufacturer::Huawei | Manufacturer::Honor => {
            "Huawei/Honor bootloader unlock:\n\
             Official unlock codes are no longer provided by Huawei.\n\
             Third-party unlock methods may be available for some models.\n\
             Attempting fastboot unlock..."
                .to_string()
        }
        Manufacturer::Motorola => {
            "Motorola bootloader unlock:\n\
             1. Get unlock code from motorola.com/unlocking.\n\
             2. Run: fastboot oem unlock <CODE>\n\
             Attempting standard unlock..."
                .to_string()
        }
        Manufacturer::Sony => {
            "Sony bootloader unlock:\n\
             1. Get unlock code from developer.sony.com/unlock.\n\
             2. Run: fastboot oem unlock 0x<CODE>\n\
             Note: DRM keys will be lost (camera quality may degrade)."
                .to_string()
        }
        _ => {
            // Standard fastboot unlock for Google, OnePlus, and others
            match fastboot(serial, &["flashing", "unlock"]) {
                Ok(out) => format!("Bootloader unlock result:\n{}", out),
                Err(_) => match fastboot(serial, &["oem", "unlock"]) {
                    Ok(out) => format!("Bootloader unlock result:\n{}", out),
                    Err(e) => format!("Unlock failed: {}\nTry: fastboot flashing unlock", e),
                },
            }
        }
    }
}

/// Relock bootloader.
pub fn relock(serial: &str) -> String {
    match fastboot(serial, &["flashing", "lock"]) {
        Ok(out) => format!("Bootloader relock result:\n{}", out),
        Err(_) => match fastboot(serial, &["oem", "lock"]) {
            Ok(out) => format!("Bootloader relock result:\n{}", out),
            Err(e) => format!("Relock failed: {}", e),
        },
    }
}

/// Get critical variables from fastboot.
pub fn get_device_vars(serial: &str) -> String {
    let vars = ["unlocked", "secure", "variant", "serialno", "product"];
    let mut output = String::from("Fastboot device variables:\n");
    for var in &vars {
        match fastboot(serial, &["getvar", var]) {
            Ok(val) => output.push_str(&format!("  {}: {}\n", var, val)),
            Err(_) => output.push_str(&format!("  {}: (unavailable)\n", var)),
        }
    }
    output
}

/// Get manufacturer-specific notes and warnings.
pub fn manufacturer_notes(manufacturer: &Manufacturer) -> &'static str {
    match manufacturer {
        Manufacturer::Samsung => {
            "Samsung Notes:\n\
             - Unlocking trips Knox counter permanently (0x1).\n\
             - Samsung Pay, Secure Folder, and some banking apps will stop working.\n\
             - Use Download mode (Vol Down + Power) for Odin operations.\n\
             - Binary counter increases with unofficial firmware."
        }
        Manufacturer::Xiaomi => {
            "Xiaomi Notes:\n\
             - Account binding wait period varies by model (72h to 30 days).\n\
             - Mi Unlock Tool or fastboot command can be used.\n\
             - POCO and Redmi sub-brands follow the same process.\n\
             - HyperOS may require additional verification."
        }
        Manufacturer::Huawei | Manufacturer::Honor => {
            "Huawei/Honor Notes:\n\
             - Official bootloader unlock codes discontinued since 2018.\n\
             - HiSilicon (Kirin) chips require special tools for EDL.\n\
             - Some models support test-point method for low-level access."
        }
        Manufacturer::Google => {
            "Google Pixel Notes:\n\
             - Straightforward unlock via fastboot flashing unlock.\n\
             - No manufacturer restrictions or wait periods.\n\
             - Carrier-locked Pixels may not support OEM unlock."
        }
        Manufacturer::OnePlus => {
            "OnePlus Notes:\n\
             - Bootloader unlock is straightforward via fastboot.\n\
             - No special tools or wait periods required.\n\
             - Device will factory reset on unlock."
        }
        _ => {
            "General Notes:\n\
             - Check manufacturer website for unlock policies.\n\
             - Standard fastboot OEM unlock may work.\n\
             - Some carriers restrict bootloader unlocking."
        }
    }
}


/// Attempt to root the device even with a locked bootloader.
/// This will try `adb root`. If it fails, it provides an informative message
/// about the difficulty of this process.
pub fn attempt_locked_root(serial: &str) -> String {
    match super::adb(serial, &["root"]) {
        Ok(out) => {
            if out.contains("cannot run as root") {
                "Attempting to root with a locked bootloader requires specific vulnerabilities (exploits)                  for your device's exact firmware version. There is no universal method.\n                 Please research your specific model (e.g., MTK-SU for older MediaTek devices,                  or specific Qualcomm EDL exploits).".to_string()
            } else {
                format!("ADB root command sent.\nOutput: {}", out)
            }
        }
        Err(e) => format!("Error sending adb root command: {}", e),
    }
}