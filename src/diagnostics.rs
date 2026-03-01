/// Device detection and diagnostic utilities via ADB and Fastboot.
use std::collections::BTreeMap;
use std::process::Command;

pub struct DeviceDiagnostics {
    device_serial: Option<String>,
}

impl DeviceDiagnostics {
    pub fn new() -> Self {
        Self {
            device_serial: None,
        }
    }

    pub fn connected_device(&self) -> Option<&str> {
        self.device_serial.as_deref()
    }

    /// Run a command and return its stdout.
    fn run_cmd(args: &[&str]) -> Option<String> {
        Command::new(args[0])
            .args(&args[1..])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Check whether ADB is reachable.
    pub fn is_adb_available() -> bool {
        Self::run_cmd(&["adb", "version"]).is_some()
    }

    /// Check whether Fastboot is reachable.
    pub fn is_fastboot_available() -> bool {
        Self::run_cmd(&["fastboot", "--version"]).is_some()
    }

    /// Detect a connected device via `adb devices`.
    pub fn detect_device(&mut self) -> Option<String> {
        let output = Self::run_cmd(&["adb", "devices"])?;
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                let serial = parts[0].to_string();
                self.device_serial = Some(serial.clone());
                return Some(serial);
            }
        }
        self.device_serial = None;
        None
    }

    /// Retrieve basic device properties via ADB getprop.
    pub fn get_device_info(&self) -> BTreeMap<String, String> {
        let mut info = BTreeMap::new();
        let serial = match &self.device_serial {
            Some(s) => s.clone(),
            None => return info,
        };

        let props = [
            ("manufacturer", "ro.product.manufacturer"),
            ("model", "ro.product.model"),
            ("android_version", "ro.build.version.release"),
            ("sdk_version", "ro.build.version.sdk"),
            ("serial", "ro.serialno"),
            ("build_fingerprint", "ro.build.fingerprint"),
        ];

        for (key, prop) in &props {
            if let Some(val) =
                Self::run_cmd(&["adb", "-s", &serial, "shell", "getprop", prop])
            {
                if !val.is_empty() {
                    info.insert(key.to_string(), val);
                }
            }
        }

        info
    }
}
