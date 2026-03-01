/// Feature modules for FOEM.
///
/// Each module provides functions that execute ADB/Fastboot commands
/// for a specific category of device operations.
/// Manufacturer-specific logic is handled via the Manufacturer enum.

pub mod bootloader;
pub mod repair;
pub mod network;
pub mod flash;
pub mod hardware_test;
pub mod tools;

use std::process::Command;

/// Supported device manufacturers.
/// Used to select manufacturer-specific methods and protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manufacturer {
    Samsung,
    Xiaomi,
    Huawei,
    Google,
    OnePlus,
    Motorola,
    Sony,
    LG,
    Nokia,
    Oppo,
    Vivo,
    Realme,
    Asus,
    ZTE,
    Meizu,
    Lenovo,
    Honor,
    Infinix,
    Nothing,
    Tecno,
}

impl Manufacturer {
    pub const ALL: &[Manufacturer] = &[
        Self::Samsung,
        Self::Xiaomi,
        Self::Huawei,
        Self::Google,
        Self::OnePlus,
        Self::Motorola,
        Self::Sony,
        Self::LG,
        Self::Nokia,
        Self::Oppo,
        Self::Vivo,
        Self::Realme,
        Self::Asus,
        Self::ZTE,
        Self::Meizu,
        Self::Lenovo,
        Self::Honor,
        Self::Infinix,
        Self::Nothing,
        Self::Tecno,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Samsung => "Samsung",
            Self::Xiaomi => "Xiaomi",
            Self::Huawei => "Huawei",
            Self::Google => "Google (Pixel)",
            Self::OnePlus => "OnePlus",
            Self::Motorola => "Motorola",
            Self::Sony => "Sony",
            Self::LG => "LG",
            Self::Nokia => "Nokia",
            Self::Oppo => "Oppo",
            Self::Vivo => "Vivo",
            Self::Realme => "Realme",
            Self::Asus => "Asus",
            Self::ZTE => "ZTE",
            Self::Meizu => "Meizu",
            Self::Lenovo => "Lenovo",
            Self::Honor => "Honor",
            Self::Infinix => "Infinix",
            Self::Nothing => "Nothing",
            Self::Tecno => "Tecno",
        }
    }

    /// Chipset platform typically used by this manufacturer.
    pub fn platform_hint(&self) -> &'static str {
        match self {
            Self::Samsung => "Exynos / Qualcomm",
            Self::Xiaomi => "Qualcomm / MediaTek",
            Self::Huawei | Self::Honor => "HiSilicon Kirin / Qualcomm",
            Self::Google => "Google Tensor / Qualcomm",
            Self::OnePlus | Self::Oppo | Self::Realme | Self::Vivo => "Qualcomm / MediaTek",
            Self::Motorola | Self::Lenovo => "Qualcomm / MediaTek",
            Self::Sony => "Qualcomm",
            Self::LG => "Qualcomm / MediaTek",
            Self::Nokia => "Qualcomm / MediaTek",
            Self::Asus => "Qualcomm",
            Self::ZTE | Self::Meizu => "Qualcomm / MediaTek",
            Self::Infinix | Self::Tecno => "MediaTek",
            Self::Nothing => "Qualcomm",
        }
    }
}

/// Shared helper: run an ADB command and return its output.
pub fn adb(serial: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("adb");
    cmd.args(["-s", serial]);
    cmd.args(args);
    match cmd.output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if err.is_empty() {
                "Command failed with no error output.".to_string()
            } else {
                err
            })
        }
        Err(e) => Err(format!("Failed to execute ADB: {}", e)),
    }
}

/// Shared helper: run a Fastboot command and return its output.
pub fn fastboot(serial: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("fastboot");
    cmd.args(["-s", serial]);
    cmd.args(args);
    match cmd.output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if err.is_empty() {
                "Command failed with no error output.".to_string()
            } else {
                err
            })
        }
        Err(e) => Err(format!("Failed to execute Fastboot: {}", e)),
    }
}

/// Shared helper: run an ADB shell command.
pub fn adb_shell(serial: &str, args: &[&str]) -> Result<String, String> {
    let mut full_args = vec!["shell"];
    full_args.extend_from_slice(args);
    adb(serial, &full_args)
}
