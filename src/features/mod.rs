/// Feature modules for FOEM.
pub mod bootloader;
pub mod flash;
pub mod hardware_test;
pub mod network;
pub mod repair;
pub mod tools;
pub mod ai_assistant;

use crate::exec;

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
    Zte,
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
        Self::Zte,
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
            Self::Zte => "ZTE",
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
            Self::Zte | Self::Meizu => "Qualcomm / MediaTek",
            Self::Infinix | Self::Tecno => "MediaTek",
            Self::Nothing => "Qualcomm",
        }
    }
}

/// Shared helper: run an ADB command and return its output.
pub fn adb(serial: &str, args: &[&str]) -> Result<String, String> {
    exec::run_with_serial("adb", serial, args, "Failed to execute ADB")
}

/// Shared helper: run a Fastboot command and return its output.
pub fn fastboot(serial: &str, args: &[&str]) -> Result<String, String> {
    exec::run_with_serial("fastboot", serial, args, "Failed to execute Fastboot")
}

/// Shared helper: run an ADB shell command.
pub fn adb_shell(serial: &str, args: &[&str]) -> Result<String, String> {
    let mut full_args = vec!["shell"];
    full_args.extend_from_slice(args);
    adb(serial, &full_args)
}
