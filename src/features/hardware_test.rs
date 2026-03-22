/// Hardware diagnostic tests via ADB.
///
/// Battery, screen, sensors, camera, audio, connectivity,
/// biometrics, USB, vibration, and general hardware tests.
use super::adb_shell;

/// Run all available hardware tests.
pub fn run_all(serial: &str) -> String {
    let mut output = String::from(
        "Full Hardware Diagnostics:
",
    );

    // Batch all commands
    let commands = [
        // Battery
        "dumpsys battery 2>/dev/null",
        "dumpsys batterystats --checkin 2>/dev/null",
        // Sensors
        "dumpsys sensorservice 2>/dev/null",
        // Display
        "wm size 2>/dev/null",
        "wm density 2>/dev/null",
        "dumpsys display 2>/dev/null",
        "getevent -lp 2>/dev/null",
        // Audio
        "dumpsys audio 2>/dev/null",
        "media volume --show 2>/dev/null",
        // Connectivity
        "dumpsys wifi 2>/dev/null",
        "dumpsys bluetooth_manager 2>/dev/null",
        "dumpsys location 2>/dev/null",
        "dumpsys nfc 2>/dev/null",
        // Cameras
        "dumpsys media.camera 2>/dev/null",
        // Biometrics
        "dumpsys fingerprint 2>/dev/null",
        "dumpsys face 2>/dev/null",
        // Storage
        "df -h 2>/dev/null",
        "sm get-primary-storage-uuid 2>/dev/null",
        // USB
        "getprop sys.usb.state 2>/dev/null",
        "getprop sys.usb.controller 2>/dev/null",
        // Telephony
        "getprop gsm.sim.state 2>/dev/null",
        "getprop gsm.sim.operator.alpha 2>/dev/null",
        "getprop gsm.network.type 2>/dev/null",
        "getprop gsm.nitz.time 2>/dev/null",
        "getprop gsm.current.phone-type 2>/dev/null",
        "getprop gsm.defaultpdpcontext.active 2>/dev/null",
    ];

    let mut script = String::new();
    for cmd in &commands {
        script.push_str(&format!(
            "{}; echo B_MARKER_FOEM_$?;
",
            cmd
        ));
    }

    match adb_shell(serial, &["sh", "-c", &script]) {
        Ok(res) => {
            let mut parts = Vec::new();
            let mut current_part = String::new();
            let mut current_status = 0;

            for line in res.lines() {
                if let Some(stripped) = line.strip_prefix("B_MARKER_FOEM_") {
                    if let Ok(status) = stripped.parse::<i32>() {
                        parts.push((current_part.trim_end().to_string(), status));
                        current_part.clear();
                        current_status = status;
                        continue;
                    }
                }
                current_part.push_str(line);
                current_part.push('\n');
            }

            // If the output ended without a marker (shouldn't happen), add the last part
            if !current_part.is_empty() {
                parts.push((current_part.trim_end().to_string(), current_status));
            }

            // Ensure we have enough parts
            while parts.len() < commands.len() {
                parts.push((String::new(), 1)); // Default to failure
            }

            // -- Battery --
            output.push_str(
                "
Battery Status:
",
            );
            if parts[0].1 == 0 {
                output.push_str(&format!(
                    "{}
",
                    parts[0].0
                ));
            } else {
                output.push_str(&format!(
                    "Battery check failed: exit status {}
",
                    parts[0].1
                ));
            }

            if parts[1].1 == 0 {
                let val = &parts[1].0;
                let summary = if val.len() > 500 { &val[..500] } else { val };
                output.push_str(&format!(
                    "Battery Statistics (summary):
{}
",
                    summary
                ));
            } else {
                output.push_str(&format!(
                    "Battery stats failed: exit status {}
",
                    parts[1].1
                ));
            }

            // -- Sensors --
            if parts[2].1 == 0 {
                output.push_str(
                    "
Sensor Report:
",
                );
                let mut count = 0;
                for line in parts[2].0.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('{')
                        || trimmed.contains("name=")
                        || trimmed.contains("vendor=")
                    {
                        output.push_str(&format!(
                            "  {}
",
                            trimmed
                        ));
                        count += 1;
                        if count > 30 {
                            output.push_str(
                                "  ... (truncated)
",
                            );
                            break;
                        }
                    }
                }
                if count == 0 {
                    output.push_str(
                        "  No sensor data available.
",
                    );
                }
            } else {
                output.push_str(&format!(
                    "
Sensor test failed: exit status {}
",
                    parts[2].1
                ));
            }

            // -- Display --
            output.push_str(
                "
Display Test:
",
            );
            if parts[3].1 == 0 {
                output.push_str(&format!(
                    "  Resolution: {}
",
                    parts[3].0
                ));
            } else {
                output.push_str(
                    "  Resolution: unknown
",
                );
            }

            if parts[4].1 == 0 {
                output.push_str(&format!(
                    "  Density: {}
",
                    parts[4].0
                ));
            } else {
                output.push_str(
                    "  Density: unknown
",
                );
            }

            if parts[5].1 == 0 {
                for line in parts[5].0.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("mPhysicalDisplayInfo")
                        || trimmed.contains("mBaseDisplayInfo")
                        || trimmed.contains("fps")
                    {
                        output.push_str(&format!(
                            "  {}
",
                            trimmed
                        ));
                    }
                }
            } else {
                output.push_str(
                    "  Display info not available.
",
                );
            }

            if parts[6].1 == 0 {
                let touch_count = parts[6].0.matches("ABS_MT_POSITION").count();
                output.push_str(&format!(
                    "  Touch input devices: {} axes found
",
                    touch_count
                ));
            } else {
                output.push_str(
                    "  Touch info not available.
",
                );
            }

            // -- Audio --
            output.push_str(
                "
Audio Test:
",
            );
            if parts[7].1 == 0 {
                for line in parts[7].0.lines().take(30) {
                    let trimmed = line.trim();
                    if trimmed.contains("Stream")
                        || trimmed.contains("speaker")
                        || trimmed.contains("SPEAKER")
                        || trimmed.contains("volume")
                    {
                        output.push_str(&format!(
                            "  {}
",
                            trimmed
                        ));
                    }
                }
            } else {
                output.push_str(
                    "  Audio dump not available.
",
                );
            }

            if parts[8].1 == 0 {
                output.push_str(
                    "  Volume UI triggered.
",
                );
            }

            // -- Connectivity --
            output.push_str(
                "
Connectivity Test:
",
            );
            if parts[9].1 == 0 {
                let enabled = parts[9].0.contains("Wi-Fi is enabled");
                output.push_str(&format!(
                    "  WiFi: {}
",
                    if enabled {
                        "enabled"
                    } else {
                        "disabled/unknown"
                    }
                ));
            } else {
                output.push_str(
                    "  WiFi: check failed
",
                );
            }

            if parts[10].1 == 0 {
                let enabled = parts[10].0.contains("enabled: true");
                output.push_str(&format!(
                    "  Bluetooth: {}
",
                    if enabled {
                        "enabled"
                    } else {
                        "disabled/unknown"
                    }
                ));
            } else {
                output.push_str(
                    "  Bluetooth: check failed
",
                );
            }

            if parts[11].1 == 0 {
                let has_gps = parts[11].0.contains("gps") || parts[11].0.contains("GPS");
                output.push_str(&format!(
                    "  GPS: {}
",
                    if has_gps { "available" } else { "not detected" }
                ));
            } else {
                output.push_str(
                    "  GPS: check failed
",
                );
            }

            if parts[12].1 == 0 {
                let has_nfc = parts[12].0.contains("mState=") || parts[12].0.contains("NFC");
                output.push_str(&format!(
                    "  NFC: {}
",
                    if has_nfc { "available" } else { "not detected" }
                ));
            } else {
                output.push_str(
                    "  NFC: not available
",
                );
            }

            // -- Cameras --
            if parts[13].1 == 0 {
                output.push_str(
                    "
Camera Report:
",
                );
                let cam_count = parts[13].0.matches("Camera ID").count();
                output.push_str(&format!(
                    "  Cameras detected: {}
",
                    cam_count
                ));
                for line in parts[13].0.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("Camera ID") || trimmed.contains("facing") {
                        output.push_str(&format!(
                            "  {}
",
                            trimmed
                        ));
                    }
                }
            } else {
                output.push_str(&format!(
                    "
Camera test failed: exit status {}
",
                    parts[13].1
                ));
            }

            // -- Biometrics --
            output.push_str(
                "
Biometrics Test:
",
            );
            if parts[14].1 == 0 {
                let has_fp = parts[14].0.contains("HAL") || parts[14].0.contains("fingerprint");
                output.push_str(&format!(
                    "  Fingerprint: {}
",
                    if has_fp {
                        "sensor detected"
                    } else {
                        "not available"
                    }
                ));
            } else {
                output.push_str(
                    "  Fingerprint: check failed
",
                );
            }

            if parts[15].1 == 0 {
                let has_face = !parts[15].0.is_empty() && !parts[15].0.contains("not found");
                output.push_str(&format!(
                    "  Face Unlock: {}
",
                    if has_face {
                        "available"
                    } else {
                        "not available"
                    }
                ));
            } else {
                output.push_str(
                    "  Face Unlock: not available
",
                );
            }

            // -- Storage --
            output.push_str(
                "
Storage Report:
",
            );
            if parts[16].1 == 0 {
                for line in parts[16].0.lines().take(10) {
                    output.push_str(&format!(
                        "  {}
",
                        line
                    ));
                }
            } else {
                output.push_str(
                    "  Storage info not available.
",
                );
            }

            if parts[17].1 == 0 {
                output.push_str(&format!(
                    "  Primary storage UUID: {}
",
                    parts[17].0
                ));
            }

            // -- USB --
            output.push_str(
                "
USB Status:
",
            );
            if parts[18].1 == 0 {
                output.push_str(&format!(
                    "  USB mode: {}
",
                    parts[18].0
                ));
            } else {
                output.push_str(
                    "  USB mode: unknown
",
                );
            }

            if parts[19].1 == 0 && !parts[19].0.is_empty() {
                output.push_str(&format!(
                    "  Controller: {}
",
                    parts[19].0
                ));
            }

            // -- Telephony --
            output.push_str(
                "
Telephony Status:
",
            );
            let labels = [
                "SIM state",
                "Operator",
                "Network type",
                "Signal strength",
                "Phone type",
                "Data state",
            ];

            for (i, label) in labels.iter().enumerate() {
                let p_idx = 20 + i;
                if parts[p_idx].1 == 0 && !parts[p_idx].0.trim().is_empty() {
                    output.push_str(&format!(
                        "  {}: {}
",
                        label,
                        parts[p_idx].0.trim()
                    ));
                } else {
                    output.push_str(&format!(
                        "  {}: --
",
                        label
                    ));
                }
            }
        }
        Err(e) => {
            output.push_str(&format!(
                "Hardware diagnostics failed to execute: {}
",
                e
            ));
        }
    }

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
            output.push_str(&format!(
                "  Touch input devices: {} axes found\n",
                touch_count
            ));
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
                if trimmed.starts_with('{')
                    || trimmed.contains("name=")
                    || trimmed.contains("vendor=")
                {
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
    if adb_shell(serial, &["media", "volume", "--show"]).is_ok() {
        output.push_str("  Volume UI triggered.\n");
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
            output.push_str(&format!(
                "  WiFi: {}\n",
                if enabled {
                    "enabled"
                } else {
                    "disabled/unknown"
                }
            ));
        }
        Err(_) => output.push_str("  WiFi: check failed\n"),
    }
    // Bluetooth
    match adb_shell(serial, &["dumpsys", "bluetooth_manager"]) {
        Ok(val) => {
            let enabled = val.contains("enabled: true");
            output.push_str(&format!(
                "  Bluetooth: {}\n",
                if enabled {
                    "enabled"
                } else {
                    "disabled/unknown"
                }
            ));
        }
        Err(_) => output.push_str("  Bluetooth: check failed\n"),
    }
    // GPS
    match adb_shell(serial, &["dumpsys", "location"]) {
        Ok(val) => {
            let has_gps = val.contains("gps") || val.contains("GPS");
            output.push_str(&format!(
                "  GPS: {}\n",
                if has_gps { "available" } else { "not detected" }
            ));
        }
        Err(_) => output.push_str("  GPS: check failed\n"),
    }
    // NFC
    match adb_shell(serial, &["dumpsys", "nfc"]) {
        Ok(val) => {
            let has_nfc = val.contains("mState=") || val.contains("NFC");
            output.push_str(&format!(
                "  NFC: {}\n",
                if has_nfc { "available" } else { "not detected" }
            ));
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
                if has_fp {
                    "sensor detected"
                } else {
                    "not available"
                }
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
                if has_face {
                    "available"
                } else {
                    "not available"
                }
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
    if let Ok(val) = adb_shell(serial, &["sm", "get-primary-storage-uuid"]) {
        output.push_str(&format!("  Primary storage UUID: {}\n", val));
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

    // Batch all getprop commands into a single shell execution
    let mut script = String::new();
    for (_, prop) in &props {
        script.push_str(&format!("getprop {}; ", prop));
    }

    match adb_shell(serial, &["sh", "-c", &script]) {
        Ok(res) => {
            let lines: Vec<&str> = res.lines().collect();
            for (i, (label, _)) in props.iter().enumerate() {
                if i < lines.len() && !lines[i].trim().is_empty() {
                    output.push_str(&format!("  {}: {}\n", label, lines[i].trim()));
                } else {
                    output.push_str(&format!("  {}: --\n", label));
                }
            }
        }
        Err(_) => {
            for (label, _) in &props {
                output.push_str(&format!("  {}: --\n", label));
            }
        }
    }

    output
}
