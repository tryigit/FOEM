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
#[cfg(not(test))]
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
use std::cell::RefCell;

#[cfg(test)]
thread_local! {
    pub static MOCK_ADB_ARGS: RefCell<Option<(String, Vec<String>)>> = RefCell::new(None);
    pub static MOCK_ADB_RESULT: RefCell<Option<Result<String, String>>> = RefCell::new(None);
}

#[cfg(test)]
pub fn adb(serial: &str, args: &[&str]) -> Result<String, String> {
    MOCK_ADB_ARGS.with(|m| {
        *m.borrow_mut() = Some((serial.to_string(), args.iter().map(|s| s.to_string()).collect()));
    });

    // By default, if no mock result is configured, we fall back to a failure string
    // that matches the standard error so we don't unexpectedly succeed in unrelated tests.
    // However, if a test actively configures it, we return that result.
    MOCK_ADB_RESULT.with(|m| {
        m.borrow_mut().take().unwrap_or_else(|| Err("Failed to execute ADB: Mock not configured".to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_shell_prepends_shell_and_delegates() {
        // Configure the mock result for this specific test thread
        MOCK_ADB_RESULT.with(|m| {
            *m.borrow_mut() = Some(Ok("mocked_shell_output".to_string()));
        });

        let serial = "test_device_123";
        let args = &["ls", "-l", "/sdcard"];

        // Execute the function under test
        let result = adb_shell(serial, args);

        // Verify output matches the mock
        assert_eq!(result.unwrap(), "mocked_shell_output");

        // Verify arguments passed to adb
        MOCK_ADB_ARGS.with(|m| {
            let captured = m.borrow();
            let captured = captured.as_ref().expect("adb was not called");
            assert_eq!(captured.0, "test_device_123");
            assert_eq!(captured.1, vec!["shell", "ls", "-l", "/sdcard"]);
        });
    }

    #[test]
    fn test_adb_shell_propagates_errors() {
        // Configure the mock result to fail
        MOCK_ADB_RESULT.with(|m| {
            *m.borrow_mut() = Some(Err("Failed to execute ADB: device offline".to_string()));
        });

        let result = adb_shell("offline_device", &["echo"]);

        assert_eq!(result.unwrap_err(), "Failed to execute ADB: device offline");

        MOCK_ADB_ARGS.with(|m| {
            let captured = m.borrow();
            let captured = captured.as_ref().expect("adb was not called");
            assert_eq!(captured.0, "offline_device");
            assert_eq!(captured.1, vec!["shell", "echo"]);
        });
    }
}
