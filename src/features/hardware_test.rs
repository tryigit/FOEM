/// Hardware diagnostic tests via ADB.
///
/// Battery, screen, sensors, camera, audio, connectivity,
/// biometrics, USB, vibration, and general hardware tests.
use super::adb_shell;
use std::fmt::Write;

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
        let _ = writeln!(script, "{}; echo B_MARKER_FOEM_$?", cmd);
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

            append_battery_report(&mut output, &parts[0], &parts[1]);
            append_sensor_report(&mut output, &parts[2]);
            append_display_report(&mut output, &parts[3], &parts[4], &parts[5], &parts[6]);
            append_audio_report(&mut output, &parts[7], &parts[8]);
            append_connectivity_report(&mut output, &parts[9], &parts[10], &parts[11], &parts[12]);
            append_camera_report(&mut output, &parts[13]);
            append_biometrics_report(&mut output, &parts[14], &parts[15]);
            append_storage_report(&mut output, &parts[16], &parts[17]);
            append_usb_report(&mut output, &parts[18], &parts[19]);
            append_telephony_report(&mut output, &parts[20..26]);
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

fn append_battery_report(output: &mut String, status: &(String, i32), stats: &(String, i32)) {
    output.push_str("\nBattery Status:\n");
    if status.1 == 0 {
        output.push_str(&format!("{}\n", status.0));
    } else {
        output.push_str(&format!("Battery check failed: exit status {}\n", status.1));
    }

    if stats.1 == 0 {
        let val = &stats.0;
        let summary = if val.len() > 500 { &val[..500] } else { val };
        output.push_str(&format!("Battery Statistics (summary):\n{}\n", summary));
    } else {
        output.push_str(&format!("Battery stats failed: exit status {}\n", stats.1));
    }
}

fn append_sensor_report(output: &mut String, sensors: &(String, i32)) {
    output.push_str("\nSensor Report:\n");
    if sensors.1 == 0 {
        let mut count = 0;
        for line in sensors.0.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') || trimmed.contains("name=") || trimmed.contains("vendor=")
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
    } else {
        output.push_str(&format!(
            "\nSensor test failed: exit status {}\n",
            sensors.1
        ));
    }
}

fn append_display_report(
    output: &mut String,
    res: &(String, i32),
    density: &(String, i32),
    disp: &(String, i32),
    touch: &(String, i32),
) {
    output.push_str("\nDisplay Test:\n");
    if res.1 == 0 {
        output.push_str(&format!("  Resolution: {}\n", res.0));
    } else {
        output.push_str("  Resolution: unknown\n");
    }

    if density.1 == 0 {
        output.push_str(&format!("  Density: {}\n", density.0));
    } else {
        output.push_str("  Density: unknown\n");
    }

    if disp.1 == 0 {
        for line in disp.0.lines() {
            let trimmed = line.trim();
            if trimmed.contains("mPhysicalDisplayInfo")
                || trimmed.contains("mBaseDisplayInfo")
                || trimmed.contains("fps")
            {
                output.push_str(&format!("  {}\n", trimmed));
            }
        }
    } else {
        output.push_str("  Display info not available.\n");
    }

    if touch.1 == 0 {
        let touch_count = touch.0.matches("ABS_MT_POSITION").count();
        output.push_str(&format!(
            "  Touch input devices: {} axes found\n",
            touch_count
        ));
    } else {
        output.push_str("  Touch info not available.\n");
    }
}

fn append_audio_report(output: &mut String, audio: &(String, i32), vol: &(String, i32)) {
    output.push_str("\nAudio Test:\n");
    if audio.1 == 0 {
        for line in audio.0.lines().take(30) {
            let trimmed = line.trim();
            if trimmed.contains("Stream")
                || trimmed.contains("speaker")
                || trimmed.contains("SPEAKER")
                || trimmed.contains("volume")
            {
                output.push_str(&format!("  {}\n", trimmed));
            }
        }
    } else {
        output.push_str("  Audio dump not available.\n");
    }

    if vol.1 == 0 {
        output.push_str("  Volume UI triggered.\n");
    }
}

fn append_connectivity_report(
    output: &mut String,
    wifi: &(String, i32),
    bt: &(String, i32),
    gps: &(String, i32),
    nfc: &(String, i32),
) {
    output.push_str("\nConnectivity Test:\n");
    if wifi.1 == 0 {
        let enabled = wifi.0.contains("Wi-Fi is enabled");
        output.push_str(&format!(
            "  WiFi: {}\n",
            if enabled {
                "enabled"
            } else {
                "disabled/unknown"
            }
        ));
    } else {
        output.push_str("  WiFi: check failed\n");
    }

    if bt.1 == 0 {
        let enabled = bt.0.contains("enabled: true");
        output.push_str(&format!(
            "  Bluetooth: {}\n",
            if enabled {
                "enabled"
            } else {
                "disabled/unknown"
            }
        ));
    } else {
        output.push_str("  Bluetooth: check failed\n");
    }

    if gps.1 == 0 {
        let has_gps = gps.0.contains("gps") || gps.0.contains("GPS");
        output.push_str(&format!(
            "  GPS: {}\n",
            if has_gps { "available" } else { "not detected" }
        ));
    } else {
        output.push_str("  GPS: check failed\n");
    }

    if nfc.1 == 0 {
        let has_nfc = nfc.0.contains("mState=") || nfc.0.contains("NFC");
        output.push_str(&format!(
            "  NFC: {}\n",
            if has_nfc { "available" } else { "not detected" }
        ));
    } else {
        output.push_str("  NFC: not available\n");
    }
}

fn append_camera_report(output: &mut String, camera: &(String, i32)) {
    if camera.1 == 0 {
        output.push_str("\nCamera Report:\n");
        let cam_count = camera.0.matches("Camera ID").count();
        output.push_str(&format!("  Cameras detected: {}\n", cam_count));
        for line in camera.0.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Camera ID") || trimmed.contains("facing") {
                output.push_str(&format!("  {}\n", trimmed));
            }
        }
    } else {
        output.push_str(&format!("\nCamera test failed: exit status {}\n", camera.1));
    }
}

fn append_biometrics_report(output: &mut String, fp: &(String, i32), face: &(String, i32)) {
    output.push_str("\nBiometrics Test:\n");
    if fp.1 == 0 {
        let has_fp = fp.0.contains("HAL") || fp.0.contains("fingerprint");
        output.push_str(&format!(
            "  Fingerprint: {}\n",
            if has_fp {
                "sensor detected"
            } else {
                "not available"
            }
        ));
    } else {
        output.push_str("  Fingerprint: check failed\n");
    }

    if face.1 == 0 {
        let has_face = !face.0.is_empty() && !face.0.contains("not found");
        output.push_str(&format!(
            "  Face Unlock: {}\n",
            if has_face {
                "available"
            } else {
                "not available"
            }
        ));
    } else {
        output.push_str("  Face Unlock: not available\n");
    }
}

fn append_storage_report(output: &mut String, df: &(String, i32), sm: &(String, i32)) {
    output.push_str("\nStorage Report:\n");
    if df.1 == 0 {
        for line in df.0.lines().take(10) {
            output.push_str(&format!("  {}\n", line));
        }
    } else {
        output.push_str("  Storage info not available.\n");
    }

    if sm.1 == 0 {
        output.push_str(&format!("  Primary storage UUID: {}\n", sm.0));
    }
}

fn append_usb_report(output: &mut String, state: &(String, i32), controller: &(String, i32)) {
    output.push_str("\nUSB Status:\n");
    if state.1 == 0 {
        output.push_str(&format!("  USB mode: {}\n", state.0));
    } else {
        output.push_str("  USB mode: unknown\n");
    }

    if controller.1 == 0 && !controller.0.is_empty() {
        output.push_str(&format!("  Controller: {}\n", controller.0));
    }
}

fn append_telephony_report(output: &mut String, parts: &[(String, i32)]) {
    output.push_str("\nTelephony Status:\n");
    let labels = [
        "SIM state",
        "Operator",
        "Network type",
        "Signal strength",
        "Phone type",
        "Data state",
    ];

    for (i, label) in labels.iter().enumerate() {
        if i < parts.len() && parts[i].1 == 0 && !parts[i].0.trim().is_empty() {
            output.push_str(&format!("  {}: {}\n", label, parts[i].0.trim()));
        } else {
            output.push_str(&format!("  {}: --\n", label));
        }
    }
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
    let script = "wm size 2>&1; echo B_MARKER_$?; \
                  wm density 2>&1; echo B_MARKER_$?; \
                  dumpsys display 2>&1; echo B_MARKER_$?; \
                  getevent -lp 2>&1; echo B_MARKER_$?;";
    match adb_shell(serial, &["sh", "-c", script]) {
        Ok(res) => {
            let mut results = Vec::new();
            let mut remaining = res.as_str();
            for _ in 0..4 {
                if let Some(pos) = remaining.find("B_MARKER_") {
                    let out = remaining[..pos].trim_end().to_string();
                    remaining = &remaining[pos + "B_MARKER_".len()..];
                    let status: String = remaining
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    remaining = &remaining[status.len()..];
                    if remaining.starts_with("\r\n") {
                        remaining = &remaining[2..];
                    } else if remaining.starts_with('\n') {
                        remaining = &remaining[1..];
                    }
                    if status == "0" {
                        results.push(Ok(out));
                    } else {
                        results.push(Err(()));
                    }
                } else {
                    results.push(Err(()));
                }
            }

            // 1. wm size
            match results.first().unwrap_or(&Err(())) {
                Ok(val) => output.push_str(&format!("  Resolution: {}\n", val)),
                Err(_) => output.push_str("  Resolution: unknown\n"),
            }

            // 2. wm density
            match results.get(1).unwrap_or(&Err(())) {
                Ok(val) => output.push_str(&format!("  Density: {}\n", val)),
                Err(_) => output.push_str("  Density: unknown\n"),
            }

            // 3. dumpsys display
            match results.get(2).unwrap_or(&Err(())) {
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

            // 4. getevent -lp
            match results.get(3).unwrap_or(&Err(())) {
                Ok(val) => {
                    let touch_count = val.matches("ABS_MT_POSITION").count();
                    output.push_str(&format!(
                        "  Touch input devices: {} axes found\n",
                        touch_count
                    ));
                }
                Err(_) => output.push_str("  Touch info not available.\n"),
            }
        }
        Err(_) => {
            output.push_str("  Resolution: unknown\n");
            output.push_str("  Density: unknown\n");
            output.push_str("  Display info not available.\n");
            output.push_str("  Touch info not available.\n");
        }
    }
    output
}

// -- Sensors --

/// Test device sensors.
pub fn test_sensors(serial: &str) -> String {
    match adb_shell(serial, &["dumpsys", "sensorservice"]) {
        Ok(val) => parse_sensor_report(&val),
        Err(e) => format!("Sensor test failed: {}", e),
    }
}

fn parse_sensor_report(val: &str) -> String {
    use std::fmt::Write;
    let mut output = String::from("Sensor Report:\n");
    let mut count = 0;

    for line in val.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') || trimmed.contains("name=") || trimmed.contains("vendor=") {
            let _ = writeln!(output, "  {}", trimmed);
            count += 1;
            if count > 30 {
                let _ = writeln!(output, "  ... (truncated)");
                break;
            }
        }
    }

    if count == 0 {
        let _ = writeln!(output, "  No sensor data available.");
    }

    output
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
    let script = "getprop sys.usb.state; getprop sys.usb.controller";
    match adb_shell(serial, &["sh", "-c", script]) {
        Ok(res) => {
            let mut lines = res.lines();

            // First part: USB state
            if let Some(state) = lines.next() {
                let val = state.trim();
                if !val.is_empty() {
                    output.push_str(&format!("  USB mode: {}\n", val));
                } else {
                    output.push_str("  USB mode: unknown\n");
                }
            } else {
                output.push_str("  USB mode: unknown\n");
            }

            // Second part: USB controller
            if let Some(controller) = lines.next() {
                let val = controller.trim();
                if !val.is_empty() {
                    output.push_str(&format!("  Controller: {}\n", val));
                }
            }
        }
        Err(_) => {
            output.push_str("  USB mode: unknown\n");
        }
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
        let _ = write!(script, "getprop {}; ", prop);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MOCK_RUN_IMPL;

    #[test]
    fn test_connectivity_check() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(
                |cmd: &str, args: &[&str], _: &str| -> Result<String, String> {
                    if cmd != "adb" {
                        return Err("Expected adb".to_string());
                    }

                    // Matches `adb_shell(serial, &["dumpsys", <service>])`
                    // run_with_serial prepends: ["-s", <serial>]
                    // and adb_shell appends: ["shell", "dumpsys", <service>]
                    // Total args: ["-s", serial, "shell", "dumpsys", <service>]
                    if args.len() >= 5 && args[2] == "shell" && args[3] == "dumpsys" {
                        let dump_target = args[4];
                        match dump_target {
                            "wifi" => Ok("Wi-Fi is enabled".to_string()),
                            "bluetooth_manager" => Ok("enabled: true".to_string()),
                            "location" => Ok("Provider gps is enabled".to_string()),
                            "nfc" => Ok("mState=on".to_string()),
                            _ => Err("Unknown dumpsys target".to_string()),
                        }
                    } else {
                        Err("Invalid arguments for adb shell".to_string())
                    }
                },
            ));
        });

        let output = test_connectivity("DEVICE123");

        assert!(output.contains("WiFi: enabled"));
        assert!(output.contains("Bluetooth: enabled"));
        assert!(output.contains("GPS: available"));
        assert!(output.contains("NFC: available"));

        // Clean up mock
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_connectivity_disabled() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(
                |_cmd: &str, args: &[&str], _: &str| -> Result<String, String> {
                    if args.len() >= 5 && args[2] == "shell" && args[3] == "dumpsys" {
                        let dump_target = args[4];
                        match dump_target {
                            "wifi" => Ok("Wi-Fi is disabled".to_string()),
                            "bluetooth_manager" => Ok("enabled: false".to_string()),
                            "location" => Ok("Provider none".to_string()),
                            "nfc" => Ok("None".to_string()),
                            _ => Err("Unknown".to_string()),
                        }
                    } else {
                        Err("Invalid args".to_string())
                    }
                },
            ));
        });

        let output = test_connectivity("DEVICE123");

        assert!(output.contains("WiFi: disabled/unknown"));
        assert!(output.contains("Bluetooth: disabled/unknown"));
        assert!(output.contains("GPS: not detected"));
        assert!(output.contains("NFC: not detected"));

        // Clean up mock
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_connectivity_error() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(
                |_: &str, _: &[&str], _: &str| -> Result<String, String> {
                    Err("adb error".to_string())
                },
            ));
        });

        let output = test_connectivity("DEVICE123");

        assert!(output.contains("WiFi: check failed"));
        assert!(output.contains("Bluetooth: check failed"));
        assert!(output.contains("GPS: check failed"));
        assert!(output.contains("NFC: not available"));

        // Clean up mock
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }
}
