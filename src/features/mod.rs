pub mod ai_assistant;
/// Feature modules for FOEM.
pub mod bootloader;
pub mod flash;
pub mod hardware_test;
pub mod network;
pub mod repair;
pub mod tools;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturer_name() {
        assert_eq!(Manufacturer::Samsung.name(), "Samsung");
        assert_eq!(Manufacturer::Xiaomi.name(), "Xiaomi");

        for manufacturer in Manufacturer::ALL {
            let name = manufacturer.name();
            assert!(!name.is_empty(), "Manufacturer name should not be empty");
        }
    }


    #[test]
    fn test_adb_fastboot_wrappers() {
        // We test that the wrappers correctly format errors when the underlying command fails.
        // This ensures the wrappers are properly passing arguments and the error prefix down.
        let serial = "dummy_serial_that_does_not_exist";
        let args = &["dummy_arg"];

        let adb_res = adb(serial, args);
        let e = adb_res.unwrap_err();
        assert!(!e.is_empty(), "Error message should not be empty");

        let shell_res = adb_shell(serial, args);
        let e = shell_res.unwrap_err();
        assert!(!e.is_empty(), "Error message should not be empty");

        let fastboot_res = fastboot(serial, args);
        let e = fastboot_res.unwrap_err();
        assert!(!e.is_empty(), "Error message should not be empty");
    }

    #[test]
    fn test_manufacturer_platform_hint() {
        assert_eq!(Manufacturer::Samsung.platform_hint(), "Exynos / Qualcomm");
        assert_eq!(
            Manufacturer::Google.platform_hint(),
            "Google Tensor / Qualcomm"
        );

        for manufacturer in Manufacturer::ALL {
            let hint = manufacturer.platform_hint();
            assert!(
                !hint.is_empty(),
                "Manufacturer platform hint should not be empty"
            );
        }
    }
}
