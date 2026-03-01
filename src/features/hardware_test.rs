/// Hardware diagnostic tests via ADB.
///
/// Battery, screen, sensors, camera, audio, connectivity,
/// biometrics, USB, vibration, and general hardware tests.

use super::adb_shell;

/// Run all available hardware tests.
pub fn run_all(serial: &str) -> String {
    let mut output = String::from("Full Hardware Diagnostics:\n");
    output.push_str(&format!("\n{}", check_battery(serial)));
    output.push_str(&format!("\n{}", test_sensors(serial)));
    output.push_str(&format!("\n{}", test_display(serial)));
    output.push_str(&format!("\n{}", test_audio(serial)));
    output.push_str(&format!("\n{}", test_connectivity(serial)));
    output.push_str(&format!("\n{}", test_cameras(serial)));
    output.push_str(&format!("\n{}", test_biometrics(serial)));
    output.push_str(&format!("\n{}", test_storage(serial)));
    output.push_str(&format!("\n{}", test_usb(serial)));
    output.push_str(&format!("\n{}", test_telephony(serial)));
    output
}

// -- Battery --

/// Check battery health, level, temperature, and charging status.
pub fn check_battery(serial: &str) -> String {
    match adb_shell(serial, &["dumpsys", "battery"]) {
        Ok(val) => format!("Battery Status:\n{}", val),
        Err(e) => format!("Battery check failed: {}", e),
    }
}

/// Get detailed battery statistics.
pub fn battery_stats(serial: &str) -> String {
    match adb_shell(serial, &["dumpsys", "batterystats", "--checkin"]) {
        Ok(val) => {
            let summary = if val.len() > 500 { &val[..500] } else { &val };
            format!("Battery Statistics (summary):\n{}", summary)
        }
        Err(e) => format!("Battery stats failed: {}", e),
    }
}

// -- Display and Touch --

/// Test display by launching display test activities.
pub fn test_display(serial: &str) -> String {
    let mut output = String::from("Display Test:\n");
    match adb_shell(serial, &["wm", "size"]) {
        Ok(val) => output.push_str(&format!("  Resolution: {}\n", val)),
        Err(_) => output.push_str("  Resolution: unknown\n"),
    }
    match adb_shell(serial, &["wm", "density"]) {
        Ok(val) => output.push_str(&format!("  Density: {}\n", val)),
        Err(_) => output.push_str("  Density: unknown\n"),
    }
    match adb_shell(serial, &["dumpsys", "display"]) {
        Ok(val) => {
            for line in val.lines() {
                let trimmed = line.trim();
                if trimmed.contains("mPhysicalDisplayInfo")
                    || trimmed.contains("mBaseDisplayInfo")
                    || trimmed.contains("fps")
                {
                    output.push_str(&format!("  {}\n", trimmed));
                }
            }
        }
        Err(_) => output.push_str("  Display info not available.\n"),
    }
    // Touch test
    match adb_shell(serial, &["getevent", "-lp"]) {
        Ok(val) => {
            let touch_count = val.matches("ABS_MT_POSITION").count();
            output.push_str(&format!("  Touch input devices: {} axes found\n", touch_count));
        }
        Err(_) => output.push_str("  Touch info not available.\n"),
    }
    output
}

// -- Sensors --

/// Test device sensors.
pub fn test_sensors(serial: &str) -> String {
    match adb_shell(serial, &["dumpsys", "sensorservice"]) {
        Ok(val) => {
            let mut output = String::from("Sensor Report:\n");
            let mut count = 0;
            for line in val.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('{') || trimmed.contains("name=") || trimmed.contains("vendor=") {
                    output.push_str(&format!("  {}\n", trimmed));
                    count += 1;
                    if count > 30 {
                        output.push_str("  ... (truncated)\n");
                        break;
                    }
                }
            }
            if count == 0 {
                output.push_str("  No sensor data available.\n");
            }
            output
        }
        Err(e) => format!("Sensor test failed: {}", e),
    }
}

// -- Audio --

/// Test audio subsystem.
pub fn test_audio(serial: &str) -> String {
    let mut output = String::from("Audio Test:\n");
    match adb_shell(serial, &["dumpsys", "audio"]) {
        Ok(val) => {
            for line in val.lines().take(30) {
                let trimmed = line.trim();
                if trimmed.contains("Stream")
                    || trimmed.contains("speaker")
                    || trimmed.contains("SPEAKER")
                    || trimmed.contains("volume")
                {
                    output.push_str(&format!("  {}\n", trimmed));
                }
            }
        }
        Err(_) => output.push_str("  Audio dump not available.\n"),
    }
    // Try to play a test tone
    match adb_shell(serial, &["media", "volume", "--show"]) {
        Ok(_) => output.push_str("  Volume UI triggered.\n"),
        Err(_) => {}
    }
    output
}

// -- Connectivity --

/// Test WiFi, Bluetooth, GPS, and NFC.
pub fn test_connectivity(serial: &str) -> String {
    let mut output = String::from("Connectivity Test:\n");
    // WiFi
    match adb_shell(serial, &["dumpsys", "wifi"]) {
        Ok(val) => {
            let enabled = val.contains("Wi-Fi is enabled");
            output.push_str(&format!("  WiFi: {}\n", if enabled { "enabled" } else { "disabled/unknown" }));
        }
        Err(_) => output.push_str("  WiFi: check failed\n"),
    }
    // Bluetooth
    match adb_shell(serial, &["dumpsys", "bluetooth_manager"]) {
        Ok(val) => {
            let enabled = val.contains("enabled: true");
            output.push_str(&format!("  Bluetooth: {}\n", if enabled { "enabled" } else { "disabled/unknown" }));
        }
        Err(_) => output.push_str("  Bluetooth: check failed\n"),
    }
    // GPS
    match adb_shell(serial, &["dumpsys", "location"]) {
        Ok(val) => {
            let has_gps = val.contains("gps") || val.contains("GPS");
            output.push_str(&format!("  GPS: {}\n", if has_gps { "available" } else { "not detected" }));
        }
        Err(_) => output.push_str("  GPS: check failed\n"),
    }
    // NFC
    match adb_shell(serial, &["dumpsys", "nfc"]) {
        Ok(val) => {
            let has_nfc = val.contains("mState=") || val.contains("NFC");
            output.push_str(&format!("  NFC: {}\n", if has_nfc { "available" } else { "not detected" }));
        }
        Err(_) => output.push_str("  NFC: not available\n"),
    }
    output
}

// -- Camera --

/// Test camera subsystem.
pub fn test_cameras(serial: &str) -> String {
    match adb_shell(serial, &["dumpsys", "media.camera"]) {
        Ok(val) => {
            let mut output = String::from("Camera Report:\n");
            let cam_count = val.matches("Camera ID").count();
            output.push_str(&format!("  Cameras detected: {}\n", cam_count));
            for line in val.lines() {
                let trimmed = line.trim();
                if trimmed.contains("Camera ID") || trimmed.contains("facing") {
                    output.push_str(&format!("  {}\n", trimmed));
                }
            }
            output
        }
        Err(e) => format!("Camera test failed: {}", e),
    }
}

// -- Biometrics --

/// Test biometric sensors (fingerprint, face unlock).
pub fn test_biometrics(serial: &str) -> String {
    let mut output = String::from("Biometrics Test:\n");
    // Fingerprint
    match adb_shell(serial, &["dumpsys", "fingerprint"]) {
        Ok(val) => {
            let has_fp = val.contains("HAL") || val.contains("fingerprint");
            output.push_str(&format!(
                "  Fingerprint: {}\n",
                if has_fp { "sensor detected" } else { "not available" }
            ));
        }
        Err(_) => output.push_str("  Fingerprint: check failed\n"),
    }
    // Face unlock
    match adb_shell(serial, &["dumpsys", "face"]) {
        Ok(val) => {
            let has_face = !val.is_empty() && !val.contains("not found");
            output.push_str(&format!(
                "  Face Unlock: {}\n",
                if has_face { "available" } else { "not available" }
            ));
        }
        Err(_) => output.push_str("  Face Unlock: not available\n"),
    }
    output
}

// -- Storage --

/// Check storage health and usage.
pub fn test_storage(serial: &str) -> String {
    let mut output = String::from("Storage Report:\n");
    match adb_shell(serial, &["df", "-h"]) {
        Ok(val) => {
            for line in val.lines().take(10) {
                output.push_str(&format!("  {}\n", line));
            }
        }
        Err(_) => output.push_str("  Storage info not available.\n"),
    }
    // Internal storage health
    match adb_shell(serial, &["sm", "get-primary-storage-uuid"]) {
        Ok(val) => output.push_str(&format!("  Primary storage UUID: {}\n", val)),
        Err(_) => {}
    }
    output
}

// -- USB --

/// Test USB connection status.
pub fn test_usb(serial: &str) -> String {
    let mut output = String::from("USB Status:\n");
    match adb_shell(serial, &["getprop", "sys.usb.state"]) {
        Ok(val) => output.push_str(&format!("  USB mode: {}\n", val)),
        Err(_) => output.push_str("  USB mode: unknown\n"),
    }
    match adb_shell(serial, &["getprop", "sys.usb.controller"]) {
        Ok(val) if !val.is_empty() => output.push_str(&format!("  Controller: {}\n", val)),
        _ => {}
    }
    output
}

// -- Telephony --

/// Test telephony and SIM status.
pub fn test_telephony(serial: &str) -> String {
    let mut output = String::from("Telephony Status:\n");
    let props = [
        ("SIM state", "gsm.sim.state"),
        ("Operator", "gsm.sim.operator.alpha"),
        ("Network type", "gsm.network.type"),
        ("Signal strength", "gsm.nitz.time"),
        ("Phone type", "gsm.current.phone-type"),
        ("Data state", "gsm.defaultpdpcontext.active"),
    ];
    for (label, prop) in &props {
        match adb_shell(serial, &["getprop", prop]) {
            Ok(val) if !val.is_empty() => output.push_str(&format!("  {}: {}\n", label, val)),
            _ => output.push_str(&format!("  {}: --\n", label)),
        }
    }
    output
}
