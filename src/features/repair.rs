/// Device repair operations: IMEI, GMS, EFS, NV data, DRK, baseband, CSC.
///
/// These operations interact with critical device partitions and data.
/// Manufacturer-specific methods are used where applicable.
use super::{adb, adb_shell, Manufacturer};
use crate::adaptive_engine::{autodetect_diag_port, execute_goal, fingerprint, FuzzGoal};

use std::io::{Read, Write};
use std::time::Duration;

// -- IMEI Management --

/// Execute a batch of adb shell commands in one `sh -c` to avoid N+1 overhead.
fn batch_shell(
    serial: &str,
    labeled_cmds: &[(&str, &str)],
) -> Vec<(String, Result<String, String>)> {
    if labeled_cmds.is_empty() {
        return Vec::new();
    }
    let mut script = String::new();
    for (_label, cmd) in labeled_cmds {
        script.push_str(cmd);
        script.push_str("; echo B_MARKER_$?;\n");
    }
    let mut results = Vec::with_capacity(labeled_cmds.len());
    match adb_shell(serial, &["sh", "-c", &script]) {
        Ok(output) => {
            let mut parts = output.split("B_MARKER_");
            let mut prev = parts.next().unwrap_or("").to_string();
            for (idx, part) in parts.enumerate() {
                if idx >= labeled_cmds.len() {
                    break;
                }
                let (code_str, rest) = match part.find('\n') {
                    Some(pos) => (&part[..pos], part[pos + 1..].to_string()),
                    None => (part, String::new()),
                };
                let val = prev.trim().to_string();
                let code: i32 = code_str.trim().parse().unwrap_or(-1);
                if code == 0 {
                    results.push((labeled_cmds[idx].0.to_string(), Ok(val)));
                } else {
                    let err = if val.is_empty() {
                        format!("exit code {}", code)
                    } else {
                        val
                    };
                    results.push((labeled_cmds[idx].0.to_string(), Err(err)));
                }
                prev = rest;
            }
        }
        Err(e) => {
            for (label, _) in labeled_cmds {
                results.push((label.to_string(), Err(e.clone())));
            }
        }
    }
    results
}

/// Read current IMEI(s) from the device using batched shell commands to avoid N+1 overhead.
pub fn read_imei(serial: &str) -> String {
    let methods: &[(&str, &str)] = &[
        ("service call", "service call iphonesubinfo 1"),
        ("getprop", "getprop persist.radio.imei"),
        ("dumpsys", "dumpsys iphonesubinfo"),
    ];
    let mut output = String::from("IMEI Information:\n");
    for (label, res) in batch_shell(serial, methods) {
        match res {
            Ok(val) if !val.is_empty() => output.push_str(&format!("  {} -- {}\n", label, val)),
            Ok(_) => output.push_str(&format!("  {} -- empty response\n", label)),
            Err(e) => output.push_str(&format!("  {} -- error: {}\n", label, e)),
        }
    }
    // Try AT command via dialer
    if adb_shell(
        serial,
        &[
            "am",
            "start",
            "-a",
            "android.intent.action.DIAL",
            "-d",
            "tel:%2A%2306%23",
        ],
    )
    .is_ok()
    {
        output.push_str("  Dialer IMEI check launched (*#06#)\n");
    }
    // Report available diagnostic serial ports for AT command access
    output.push_str("\nDiagnostic Serial Ports:\n");
    match serialport::available_ports() {
        Ok(ports) => {
            let usb_ports: Vec<_> = ports
                .iter()
                .filter(|p| matches!(p.port_type, serialport::SerialPortType::UsbPort(_)))
                .collect();
            if usb_ports.is_empty() {
                output.push_str("  No USB diagnostic ports detected.\n");
            } else {
                for p in &usb_ports {
                    let desc = match &p.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            info.product.as_deref().unwrap_or("Unknown device")
                        }
                        _ => "Unknown",
                    };
                    output.push_str(&format!("  {} ({})\n", p.port_name, desc));
                }
                output
                    .push_str("  Use 'Read IMEI (Diag)' with a port name for AT command access.\n");
            }
        }
        Err(e) => {
            output.push_str(&format!("  Port enumeration unavailable: {}\n", e));
        }
    }
    output
}

/// Open Xiaomi Modem Test Board (MTB) menu using secret code broadcast
pub fn open_xiaomi_mtb(serial: &str) -> String {
    let mut output = String::from(
        "Opening Xiaomi MTB Menu (*#*#663368378#*#*)...
",
    );

    // Method 1: Broadcast secret code (Best method for Android 8+)
    let _ = adb_shell(
        serial,
        &[
            "am",
            "broadcast",
            "-a",
            "android.provider.Telephony.SECRET_CODE",
            "-d",
            "android_secret_code://663368378",
        ],
    );

    // Method 2: Launch com.xiaomi.mtb activity directly
    let _ = adb_shell(
        serial,
        &[
            "am",
            "start",
            "-n",
            "com.xiaomi.mtb/com.xiaomi.mtb.MainActivity",
        ],
    );

    // Method 3: Call via dialer just in case
    let _ = adb_shell(
        serial,
        &[
            "am",
            "start",
            "-a",
            "android.intent.action.DIAL",
            "-d",
            "tel:%2A%23%2A%23663368378%23%2A%23%2A",
        ],
    );

    output.push_str(
        "Executed launch commands.
Check the device screen for the MTB/Modem Test interface.
",
    );

    output
}

/// Backup IMEI data (EFS-based) to device storage.
pub fn backup_imei(serial: &str) -> String {
    let backup_path = "/sdcard/FOEM/imei_backup";
    let _ = adb_shell(serial, &["mkdir", "-p", backup_path]);
    let partitions = ["efs", "modemst1", "modemst2", "fsg", "fsc"];
    let mut output = String::from("IMEI/EFS Backup:\n");
    for part in &partitions {
        let src = format!("/dev/block/bootdevice/by-name/{}", part);
        let dst = format!("{}/{}.img", backup_path, part);
        match adb_shell(
            serial,
            &["dd", &format!("if={}", src), &format!("of={}", dst)],
        ) {
            Ok(_) => output.push_str(&format!("  {} -- backed up\n", part)),
            Err(_) => output.push_str(&format!("  {} -- not found or access denied\n", part)),
        }
    }
    output
}

/// Write IMEI via AT command over a diagnostic port when available.
pub fn write_imei(_serial: &str, imei: &str, manufacturer: &Manufacturer) -> String {
    let imeis = match parse_imei_input(imei) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let diag_port = autodetect_diag_port();
    let port_name = match diag_port {
        Some(p) => p,
        None => {
            return "No diagnostic port detected. Enable diag/AT mode and try again.".to_string();
        }
    };

    let mut port = match open_diag_port(&port_name) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let commands = build_imei_write_commands(manufacturer, &imeis);
    let mut output = format!("IMEI write command(s) sent on {}:\n", port_name);
    for (slot, at_cmd) in commands.iter().enumerate() {
        match send_at_command(&mut port, at_cmd) {
            Ok(resp) => {
                output.push_str(&format!(
                    "  Slot {} command: {}\n  Response:\n{}\n",
                    slot + 1,
                    at_cmd,
                    resp
                ));
            }
            Err(e) => {
                output.push_str(&format!(
                    "  Slot {} command failed: {}\n  Error: {}\n",
                    slot + 1,
                    at_cmd,
                    e
                ));
            }
        }
    }
    output
}

fn parse_imei_input(input: &str) -> Result<Vec<String>, String> {
    let imeis: Vec<String> = input
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter_map(|piece| {
            let trimmed = piece.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();
    if imeis.is_empty() {
        return Err("Invalid IMEI. Enter one or two IMEI values.".to_string());
    }
    if imeis.len() > 2 {
        return Err("Invalid IMEI input. Enter at most two IMEI values.".to_string());
    }
    if imeis
        .iter()
        .any(|v| v.len() != 15 || !v.chars().all(|c| c.is_ascii_digit()))
    {
        return Err("Invalid IMEI. Each IMEI must be exactly 15 digits.".to_string());
    }
    Ok(imeis)
}

fn build_imei_write_commands(manufacturer: &Manufacturer, imeis: &[String]) -> Vec<String> {
    if imeis.len() == 2 {
        return match manufacturer {
            Manufacturer::Google | Manufacturer::OnePlus | Manufacturer::Motorola => vec![
                format!("AT+CGSN={}", imeis[0]),
                format!("AT+CGSN=1,{}", imeis[1]),
            ],
            _ => vec![
                format!("AT+EGMR=1,7,\"{}\"", imeis[0]),
                format!("AT+EGMR=1,10,\"{}\"", imeis[1]),
            ],
        };
    }

    let primary = match manufacturer {
        Manufacturer::Samsung => format!("AT+EGMR=1,7,\"{}\"", imeis[0]),
        Manufacturer::Xiaomi
        | Manufacturer::Oppo
        | Manufacturer::Realme
        | Manufacturer::Vivo
        | Manufacturer::Huawei
        | Manufacturer::Honor => format!("AT+EGMR=1,10,\"{}\"", imeis[0]),
        Manufacturer::Google | Manufacturer::OnePlus | Manufacturer::Motorola => {
            format!("AT+CGSN={}", imeis[0])
        }
        _ => format!("AT+EGMR=1,7,\"{}\"", imeis[0]),
    };
    vec![primary]
}

// -- Diagnostic Serial Port Communication --

/// Send an AT command to a serial port and read the response.
pub fn send_at_command(
    port: &mut Box<dyn serialport::SerialPort>,
    command: &str,
) -> Result<String, String> {
    // Clear any stale data from the serial buffer before sending the command
    let mut discard = [0u8; 1024];
    let _ = port.read(&mut discard);

    let cmd = format!("{}\r\n", command);
    port.write_all(cmd.as_bytes())
        .map_err(|e| format!("Failed to write to port: {}", e))?;
    port.flush()
        .map_err(|e| format!("Failed to flush port: {}", e))?;

    // Allow device time to process the command and prepare the response
    std::thread::sleep(Duration::from_millis(200));

    let mut response = Vec::new();
    let mut buf = [0u8; 256];
    let deadline = std::time::Instant::now() + Duration::from_secs(3);

    loop {
        match port.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&response);
                if text.contains("OK") || text.contains("ERROR") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if !response.is_empty() {
                    break;
                }
            }
            Err(e) => return Err(format!("Read error: {}", e)),
        }
        if std::time::Instant::now() >= deadline {
            break;
        }
    }

    if response.is_empty() {
        return Err("No response from device. Port may not be a diagnostic port.".to_string());
    }

    Ok(String::from_utf8_lossy(&response).to_string())
}

/// Parse the meaningful value from an AT command response.
fn parse_at_value(response: &str, command_echo: &str) -> String {
    let mut value_lines = Vec::new();
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "OK" || trimmed == "ERROR" {
            continue;
        }
        if trimmed.starts_with(command_echo) {
            continue;
        }
        if let Some((_prefix, val)) = trimmed.split_once(':') {
            value_lines.push(val.trim().to_string());
        } else {
            value_lines.push(trimmed.to_string());
        }
    }
    value_lines.join("\n")
}

/// List available serial/diagnostic ports on the system.
pub fn list_diag_ports() -> String {
    match serialport::available_ports() {
        Ok(ports) if ports.is_empty() => "No serial/diagnostic ports detected.\n\
             Ensure the device is connected and in diagnostic (Diag) mode.\n\
             Qualcomm devices: Look for Qualcomm HS-USB Diagnostics 900E.\n\
             MediaTek devices: Look for MediaTek USB Port."
            .to_string(),
        Ok(ports) => {
            let mut output = String::from("Available Serial Ports:\n");
            for port in &ports {
                let type_info = match &port.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        format!(
                            "USB -- VID:{:04X} PID:{:04X}{}",
                            info.vid,
                            info.pid,
                            info.product
                                .as_deref()
                                .map(|p| format!(" ({})", p))
                                .unwrap_or_default()
                        )
                    }
                    serialport::SerialPortType::PciPort => "PCI".to_string(),
                    serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                    serialport::SerialPortType::Unknown => "Unknown".to_string(),
                };
                output.push_str(&format!("  {} -- {}\n", port.port_name, type_info));
            }
            output
        }
        Err(e) => format!("Failed to enumerate serial ports: {}", e),
    }
}

pub fn open_diag_port(port_name: &str) -> Result<Box<dyn serialport::SerialPort>, String> {
    let port_result = serialport::new(port_name, 115200)
        .timeout(Duration::from_secs(3))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .open();

    match port_result {
        Ok(p) => Ok(p),
        Err(e) => {
            let detail = match e.kind {
                serialport::ErrorKind::NoDevice => format!(
                    "Port {} not found.\n\
                     The device may have been disconnected or the port name is incorrect.",
                    port_name
                ),
                serialport::ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => format!(
                    "Permission denied on port {}.\n\
                     The port may be locked by another application or require elevated privileges.\n\
                     Linux: Check udev rules or run with sudo.\n\
                     Windows: Check Device Manager for port conflicts.",
                    port_name
                ),
                _ => format!(
                    "Cannot open port {}: {}\n\
                     Ensure the device is connected and in diagnostic mode.",
                    port_name, e
                ),
            };
            Err(format!("Diag Port Error:\n  {}", detail))
        }
    }
}

/// Send raw diagnostic bytes over an open port. Returns error on write/flush failure.
pub fn send_diag_bytes(
    port: &mut Box<dyn serialport::SerialPort>,
    bytes: &[u8],
) -> Result<(), String> {
    port.write_all(bytes)
        .map_err(|e| format!("Diag write failed: {}", e))?;
    port.flush()
        .map_err(|e| format!("Diag flush failed: {}", e))?;
    Ok(())
}

fn query_device_identity(port: &mut Box<dyn serialport::SerialPort>, output: &mut String) {
    match send_at_command(port, "AT") {
        Ok(resp) => {
            if !resp.contains("OK") {
                output.push_str(
                    "  Port did not respond with OK to AT command.\n\
                     Device may not be in AT command / diagnostic mode.\n",
                );
                return;
            }
            output.push_str("  Port alive: OK\n");
        }
        Err(e) => {
            output.push_str(&format!(
                "  Port is not responding to AT commands: {}\n\
                 Device may not be in diagnostic mode.\n\
                 Qualcomm: Device must be in Diag/AT mode (not ADB or Fastboot).\n\
                 Try switching USB mode on the device.\n",
                e
            ));
            return;
        }
    }

    if let Ok(resp) = send_at_command(port, "AT+CGMI") {
        let val = parse_at_value(&resp, "AT+CGMI");
        if !val.is_empty() {
            output.push_str(&format!("  Manufacturer: {}\n", val));
        }
    }

    if let Ok(resp) = send_at_command(port, "AT+CGMM") {
        let val = parse_at_value(&resp, "AT+CGMM");
        if !val.is_empty() {
            output.push_str(&format!("  Model: {}\n", val));
        }
    }

    if let Ok(resp) = send_at_command(port, "AT+CGMR") {
        let val = parse_at_value(&resp, "AT+CGMR");
        if !val.is_empty() {
            output.push_str(&format!("  Revision: {}\n", val));
        }
    }

    match send_at_command(port, "AT+CGSN") {
        Ok(resp) => {
            if resp.contains("ERROR") {
                output.push_str(
                    "  IMEI (AT+CGSN): ERROR -- command not supported or device not ready.\n",
                );
            } else {
                let imei = parse_at_value(&resp, "AT+CGSN");
                if imei.is_empty() {
                    output.push_str("  IMEI (AT+CGSN): no response data.\n");
                } else {
                    let clean: String =
                        imei.trim().chars().filter(|c| c.is_ascii_digit()).collect();
                    if clean.len() == 15 {
                        output.push_str(&format!("  IMEI 1: {}\n", clean));
                    } else {
                        output.push_str(&format!("  IMEI (raw): {}\n", imei));
                    }
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("  IMEI read failed: {}\n", e));
        }
    }

    if let Ok(resp) = send_at_command(port, "AT+CGSN=1") {
        if !resp.contains("ERROR") {
            let imei2 = parse_at_value(&resp, "AT+CGSN");
            if !imei2.is_empty() {
                let clean: String = imei2
                    .trim()
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if clean.len() == 15 {
                    output.push_str(&format!("  IMEI 2: {}\n", clean));
                }
            }
        }
    }
}

/// Read IMEI from a diagnostic serial port using AT+CGSN.
///
/// Opens the specified port at 115200 baud, sends standard AT commands
/// to read device identity, and returns the results. Returns clear error
/// messages if the port is locked, inaccessible, or the device is not
/// in diagnostic mode.
pub fn read_imei_diag(port_name: &str) -> String {
    if port_name.is_empty() {
        return "Diagnostic port name is required.\n\
                Use 'List Ports' to find available diagnostic ports."
            .to_string();
    }

    let mut port = match open_diag_port(port_name) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let mut output = format!("Diag Port IMEI Read ({}):\n", port_name);
    query_device_identity(&mut port, &mut output);
    output
}

fn get_fingerprint(serial: &str) -> String {
    let script = "getprop ro.product.model; echo SEP; getprop ro.build.version.release; echo SEP; getprop ro.board.platform";
    let out = adb_shell(serial, &["sh", "-c", script]).unwrap_or_default();
    let mut parts = out.split("SEP");
    let model = parts.next().unwrap_or("").trim();
    let release = parts.next().unwrap_or("").trim();
    let platform = parts.next().unwrap_or("").trim();
    fingerprint(model, release, platform)
}

// -- GMS (Google Mobile Services) Repair --

const GMS_PACKAGES: &[&str] = &[
    "com.google.android.gms",
    "com.google.android.gsf",
    "com.android.vending",
    "com.google.android.apps.setup",
    "com.google.android.setupwizard",
    "com.google.android.apps.restore",
];

/// Check GMS package status.
pub fn check_gms(serial: &str) -> String {
    let mut output = String::from("GMS Package Status:\n");
    let mut cmd = String::new();
    for (i, pkg) in GMS_PACKAGES.iter().enumerate() {
        cmd.push_str(&format!("pm list packages {}", pkg));
        if i < GMS_PACKAGES.len() - 1 {
            cmd.push_str("; echo B_MARKER; ");
        }
    }

    match adb_shell(serial, &["sh", "-c", &cmd]) {
        Ok(res) => {
            let parts: Vec<&str> = res.split("B_MARKER").collect();
            for (i, pkg) in GMS_PACKAGES.iter().enumerate() {
                let out = parts.get(i).copied().unwrap_or("");
                let installed = out.contains(pkg);
                output.push_str(&format!(
                    "  {} -- {}\n",
                    pkg,
                    if installed { "installed" } else { "MISSING" }
                ));
            }
        }
        Err(_) => {
            for pkg in GMS_PACKAGES {
                output.push_str(&format!("  {} -- MISSING\n", pkg));
            }
        }
    }
    output
}

/// Clear GMS caches and force restart.
pub fn repair_gms(serial: &str) -> String {
    let mut output = String::from("GMS Repair:\n");
    let mut cmd = String::new();
    for (i, pkg) in GMS_PACKAGES.iter().enumerate() {
        cmd.push_str(&format!("pm clear {}", pkg));
        if i < GMS_PACKAGES.len() - 1 {
            cmd.push_str("; echo B_MARKER; ");
        }
    }

    // Batch force-stop and broadcast into the same shell command
    cmd.push_str("; am force-stop com.google.android.gms; am broadcast -a android.intent.action.BOOT_COMPLETED");

    let _ = adb_shell(serial, &["sh", "-c", &cmd]);
    for pkg in GMS_PACKAGES {
        output.push_str(&format!("  Cleared cache: {}\n", pkg));
    }

    output.push_str("  Force-stopped GMS and sent boot broadcast.\n");
    output.push_str("  Reboot recommended for full effect.\n");
    output
}

/// Install GMS from a package archive path on the device.
pub fn install_gms_package(serial: &str, apk_path: &str) -> String {
    match adb(serial, &["install", "-r", apk_path]) {
        Ok(out) => format!("GMS install result:\n{}", out),
        Err(e) => format!("GMS install failed: {}", e),
    }
}

// -- EFS Backup and Restore --

/// Backup EFS partition.
pub fn backup_efs(serial: &str) -> String {
    let backup_dir = "/sdcard/FOEM/efs_backup";
    let _ = adb_shell(serial, &["mkdir", "-p", backup_dir]);
    match adb_shell(serial, &["ls", "/efs/"]) {
        Ok(listing) => {
            let _ = adb_shell(
                serial,
                &[
                    "tar",
                    "-czf",
                    &format!("{}/efs.tar.gz", backup_dir),
                    "/efs/",
                ],
            );
            format!(
                "EFS backup attempt:\n  Contents: {}\n  Saved to: {}/efs.tar.gz",
                listing, backup_dir
            )
        }
        Err(_) => "EFS partition not accessible. Root may be required.".to_string(),
    }
}

/// Restore EFS partition from backup.
pub fn restore_efs(serial: &str) -> String {
    let backup_path = "/sdcard/FOEM/efs_backup/efs.tar.gz";
    match adb_shell(serial, &["ls", backup_path]) {
        Ok(_) => {
            let _ = adb_shell(serial, &["tar", "-xzf", backup_path, "-C", "/"]);
            format!(
                "EFS restore attempted from {}.\nReboot required.",
                backup_path
            )
        }
        Err(_) => "No EFS backup found. Run backup first.".to_string(),
    }
}

// -- NV Data (Non-Volatile) --

/// Backup NV data partitions (modemst1, modemst2, fsg).
pub fn backup_nv_data(serial: &str) -> String {
    let backup_dir = "/sdcard/FOEM/nv_backup";
    let _ = adb_shell(serial, &["mkdir", "-p", backup_dir]);
    let partitions = ["modemst1", "modemst2", "fsg", "fsc"];
    let mut output = String::from("NV Data Backup:\n");

    let mut cmd = String::new();
    for (i, part) in partitions.iter().enumerate() {
        let src = format!("/dev/block/bootdevice/by-name/{}", part);
        let dst = format!("{}/{}.img", backup_dir, part);
        cmd.push_str(&format!(
            "if dd if={} of={} 2>/dev/null; then echo OK; else echo FAIL; fi",
            src, dst
        ));
        if i < partitions.len() - 1 {
            cmd.push_str("; echo B_MARKER; ");
        }
    }

    match adb_shell(serial, &["sh", "-c", &cmd]) {
        Ok(res) => {
            let parts: Vec<&str> = res.split("B_MARKER").collect();
            for (i, part) in partitions.iter().enumerate() {
                let out = parts.get(i).copied().unwrap_or("").trim();
                if out.contains("OK") {
                    output.push_str(&format!("  {} -- saved\n", part));
                } else {
                    output.push_str(&format!("  {} -- failed (root required)\n", part));
                }
            }
        }
        Err(_) => {
            for part in &partitions {
                output.push_str(&format!("  {} -- failed (root required)\n", part));
            }
        }
    }
    output
}

/// Restore NV data partitions from backup.
pub fn restore_nv_data(serial: &str) -> String {
    let backup_dir = "/sdcard/FOEM/nv_backup";
    let partitions = ["modemst1", "modemst2", "fsg", "fsc"];
    let mut output = String::from("NV Data Restore:\n");

    let mut cmd = String::new();
    for (i, part) in partitions.iter().enumerate() {
        let src = format!("{}/{}.img", backup_dir, part);
        let dst = format!("/dev/block/bootdevice/by-name/{}", part);
        cmd.push_str(&format!(
            "if dd if={} of={} 2>/dev/null; then echo OK; else echo FAIL; fi",
            src, dst
        ));
        if i < partitions.len() - 1 {
            cmd.push_str("; echo B_MARKER; ");
        }
    }

    match adb_shell(serial, &["sh", "-c", &cmd]) {
        Ok(res) => {
            let parts: Vec<&str> = res.split("B_MARKER").collect();
            for (i, part) in partitions.iter().enumerate() {
                let out = parts.get(i).copied().unwrap_or("").trim();
                if out.contains("OK") {
                    output.push_str(&format!("  {} -- restored\n", part));
                } else {
                    output.push_str(&format!("  {} -- failed\n", part));
                }
            }
        }
        Err(_) => {
            for part in &partitions {
                output.push_str(&format!("  {} -- failed\n", part));
            }
        }
    }
    output.push_str("  Reboot required.\n");
    output
}

// -- Samsung-Specific Repair --

/// DRK (Device Root Key) repair for Samsung devices.
pub fn repair_drk(serial: &str) -> String {
    let script = "rm -f /efs/prov/cc.dat && rm -rf /efs/prov_data/ && rm -f /efs/prov/ridge.dat";
    let mut output = String::from("DRK Repair (Samsung):\n");
    match adb_shell(serial, &["sh", "-c", script]) {
        Ok(_) => {
            output.push_str("  Removing DRK flag -- done\n");
            output.push_str("  Clearing DRK data -- done\n");
            output.push_str("  Removing warranty void -- done\n");
        }
        Err(_) => output.push_str("  DRK Repair operations failed (root required)\n"),
    }
    output.push_str("  Reboot required. DRK will re-provision on next boot.\n");
    output
}

/// Check Samsung Knox counter status.
pub fn check_knox_counter(serial: &str) -> String {
    match adb_shell(serial, &["cat", "/sys/kernel/security/knox/knox_warranty"]) {
        Ok(val) => {
            let tripped = val.trim() == "1" || val.contains("0x1");
            format!(
                "Knox Warranty Counter: {} ({})",
                val.trim(),
                if tripped { "TRIPPED" } else { "OK" }
            )
        }
        Err(_) => "Knox counter not readable. Not a Samsung device or root required.".to_string(),
    }
}

/// Change CSC (Consumer Software Customization) on Samsung.
pub fn change_csc(_serial: &str, csc_code: &str) -> String {
    if csc_code.len() != 3 || !csc_code.chars().all(|c| c.is_ascii_uppercase()) {
        return "Invalid CSC code. Must be 3 uppercase letters (e.g., XEU, OXM, INS).".to_string();
    }
    format!(
        "CSC Change to {}:\n\
         Method: Write CSC code to sales_code.dat in EFS.\n\
         Path: /efs/imei/mps_code.dat\n\
         Note: Factory reset required after CSC change.\n\
         This operation requires root access.",
        csc_code
    )
}

// -- Baseband and Modem --

/// Check baseband/modem version.
pub fn check_baseband(serial: &str) -> String {
    let props = [
        ("Baseband", "gsm.version.baseband"),
        ("RIL Version", "gsm.version.ril-impl"),
        ("Modem Board", "ro.board.platform"),
        ("Radio", "gsm.current.phone-type"),
    ];

    let mut script = String::new();
    for (_, prop) in &props {
        script.push_str(&format!("getprop {}; echo B_MARKER; ", prop));
    }

    let mut output = String::from("Baseband/Modem Info:\n");
    match adb_shell(serial, &["sh", "-c", script.as_str()]) {
        Ok(res) => {
            let mut parts = res.split("B_MARKER");
            for (label, _) in &props {
                let val = parts.next().unwrap_or("").trim();
                if !val.is_empty() {
                    output.push_str(&format!("  {}: {}\n", label, val));
                } else {
                    output.push_str(&format!("  {}: not available\n", label));
                }
            }
        }
        Err(_) => {
            for (label, _) in &props {
                output.push_str(&format!("  {}: not available\n", label));
            }
        }
    }
    output
}

/// Attempt baseband repair by clearing modem cache.
pub fn repair_baseband(serial: &str) -> String {
    let mut output = String::from("Baseband Repair:\n");
    let script = "setprop persist.sys.modem.diag ,default; rm -rf /cache/modem_*";
    match adb_shell(serial, &["sh", "-c", script]) {
        Ok(_) => output.push_str("  Cleared modem cache.\n"),
        Err(_) => output.push_str("  Modem cache clear failed (root may be required).\n"),
    }
    output.push_str("  Reflashing modem partition may be required for severe issues.\n");
    output.push_str("  Reboot required.\n");
    output
}

// -- Build.prop Management --

/// Read key build.prop values.
pub fn read_build_props(serial: &str) -> String {
    let props = [
        ("Model", "ro.product.model"),
        ("Device", "ro.product.device"),
        ("Brand", "ro.product.brand"),
        ("Manufacturer", "ro.product.manufacturer"),
        ("Android Version", "ro.build.version.release"),
        ("SDK", "ro.build.version.sdk"),
        ("Security Patch", "ro.build.version.security_patch"),
        ("Build Number", "ro.build.display.id"),
        ("Fingerprint", "ro.build.fingerprint"),
        ("Hardware", "ro.hardware"),
        ("Bootloader", "ro.bootloader"),
        ("Board", "ro.board.platform"),
    ];
    let mut output = String::from("Build Properties:\n");

    let mut cmd = String::new();
    for (i, (_, prop)) in props.iter().enumerate() {
        cmd.push_str(&format!("getprop {}", prop));
        if i < props.len() - 1 {
            cmd.push_str("; echo B_MARKER; ");
        }
    }

    match adb_shell(serial, &["sh", "-c", &cmd]) {
        Ok(res) => {
            let parts: Vec<&str> = res.split("B_MARKER").map(|s| s.trim()).collect();
            for (i, (label, _)) in props.iter().enumerate() {
                let val = parts.get(i).copied().unwrap_or("").trim();
                let display_val = if val.is_empty() { "--" } else { val };
                output.push_str(&format!("  {}: {}\n", label, display_val));
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

// -- Adaptive Diag Enable via heuristic engine --

/// Enable diagnostic port using adaptive heuristic engine with self-learning.
pub fn enable_diag_port(serial: &str, manufacturer: &Manufacturer) -> String {
    let fp = get_fingerprint(serial);
    // Use manufacturer hint as part of fingerprint to increase specificity
    let fp = format!("{}|{}", fp, manufacturer.name());
    execute_goal(serial, FuzzGoal::EnableDiagPort, &fp, None)
}

#[cfg(test)]
mod tests {
    use super::{build_imei_write_commands, parse_imei_input};
    use crate::features::Manufacturer;

    #[test]
    fn parse_single_imei() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = parse_imei_input("123456789012345")?;
        assert_eq!(parsed, vec!["123456789012345".to_string()]);
        Ok(())
    }

    #[test]
    fn parse_dual_imei_with_comma() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = parse_imei_input("123456789012345,234567890123456")?;
        assert_eq!(
            parsed,
            vec!["123456789012345".to_string(), "234567890123456".to_string()]
        );
        Ok(())
    }

    #[test]
    fn parse_rejects_more_than_two_imeis() {
        let err = parse_imei_input("123456789012345,234567890123456,345678901234567")
            .expect_err("more than two imeis should fail");
        assert!(err.contains("at most two"));
    }

    #[test]
    fn build_commands_includes_second_slot_when_present() {
        let imeis = vec!["123456789012345".to_string(), "234567890123456".to_string()];
        let commands = build_imei_write_commands(&Manufacturer::Samsung, &imeis);
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("AT+EGMR=1,7"));
        assert!(commands[1].contains("AT+EGMR=1,10"));
        assert!(commands[0].contains("123456789012345"));
        assert!(commands[1].contains("234567890123456"));
    }

    #[test]
    fn build_commands_dual_imei_uses_two_slots_for_xiaomi() {
        let imeis = vec!["123456789012345".to_string(), "234567890123456".to_string()];
        let commands = build_imei_write_commands(&Manufacturer::Xiaomi, &imeis);
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("AT+EGMR=1,7"));
        assert!(commands[1].contains("AT+EGMR=1,10"));
    }
}
