/// Device detection and diagnostic utilities via ADB and Fastboot.
use std::collections::BTreeMap;

use crate::exec::{self, COMMAND_TIMEOUT};

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

    /// Run a command and return its stdout, with a short timeout to avoid UI hangs.
    fn run_cmd(program: &str, args: &[&str]) -> Result<String, String> {
        exec::run_with_timeout(program, args, "Diagnostics command failed", COMMAND_TIMEOUT)
    }

    /// Check whether ADB is reachable.
    pub fn is_adb_available() -> bool {
        Self::run_cmd("adb", &["version"]).is_ok()
    }

    /// Check whether Fastboot is reachable.
    pub fn is_fastboot_available() -> bool {
        Self::run_cmd("fastboot", &["--version"]).is_ok()
    }

    /// Detect a connected device via `adb devices`.
    pub fn detect_device(&mut self) -> Result<Option<String>, String> {
        let output = Self::run_cmd("adb", &["devices"])?;
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                let serial = parts[0].to_string();
                self.device_serial = Some(serial.clone());
                return Ok(Some(serial));
            }
        }
        self.device_serial = None;
        Ok(None)
    }

    /// Retrieve basic device properties via ADB getprop, batched to avoid N+1 shell calls.
    pub fn get_device_info(&self) -> BTreeMap<String, String> {
        let mut info = BTreeMap::new();
        let serial = match self.device_serial.as_deref() {
            Some(s) => s,
            None => {
                info.insert(
                    "error".to_string(),
                    "No device detected. Please connect and authorize USB debugging.".to_string(),
                );
                return info;
            }
        };

        let props: &[(&str, &str)] = &[
            ("manufacturer", "ro.product.manufacturer"),
            ("model", "ro.product.model"),
            ("android_version", "ro.build.version.release"),
            ("sdk_version", "ro.build.version.sdk"),
            ("serial", "ro.serialno"),
            ("build_fingerprint", "ro.build.fingerprint"),
        ];

        let mut script = String::new();
        for (_, prop) in props {
            script.push_str(&format!("getprop {} 2>&1; echo B_MARKER_$?;\n", prop));
        }

        match Self::run_cmd("adb", &["-s", serial, "shell", "sh", "-c", &script]) {
            Ok(output) => {
                let mut parts = output.split("B_MARKER_");
                let mut prev = parts.next().unwrap_or("").to_string();
                for (idx, part) in parts.enumerate() {
                    if idx >= props.len() {
                        break;
                    }
                    let (code_str, rest) = match part.find('\n') {
                        Some(pos) => (&part[..pos], part[pos + 1..].to_string()),
                        None => (part, String::new()),
                    };
                    let val = prev.trim();
                    let key = props[idx].0;
                    let code: i32 = code_str.trim().parse().unwrap_or(-1);
                    if code == 0 && !val.is_empty() {
                        info.insert(key.to_string(), val.to_string());
                    } else if code != 0 {
                        info.insert(format!("error_{key}"), format!("getprop failed: {}", val));
                    }
                    prev = rest;
                }
            }
            Err(e) => {
                info.insert(
                    "error".to_string(),
                    format!("Unable to query properties: {}", e),
                );
            }
        };

        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let diagnostics = DeviceDiagnostics::new();
        assert_eq!(diagnostics.device_serial, None);
        assert_eq!(diagnostics.connected_device(), None);
    }

    #[test]
    fn test_get_device_info_no_device() {
        let diagnostics = DeviceDiagnostics::new();
        let info = diagnostics.get_device_info();
        assert_eq!(
            info.get("error").unwrap(),
            "No device detected. Please connect and authorize USB debugging."
        );
    }
}
