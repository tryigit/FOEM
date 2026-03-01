/// Device repair operations: IMEI, GMS, EFS, NV data, DRK, baseband, CSC.
///
/// These operations interact with critical device partitions and data.
/// Manufacturer-specific methods are used where applicable.

use super::{adb, adb_shell, Manufacturer};

// -- IMEI Management --

/// Read current IMEI(s) from the device.
pub fn read_imei(serial: &str) -> String {
    let methods = [
        ("service call", &["service", "call", "iphonesubinfo", "1"][..]),
        ("getprop", &["getprop", "persist.radio.imei"][..]),
        ("dumpsys", &["dumpsys", "iphonesubinfo"][..]),
    ];
    let mut output = String::from("IMEI Information:\n");
    for (name, args) in &methods {
        match adb_shell(serial, args) {
            Ok(val) if !val.is_empty() => {
                output.push_str(&format!("  {} -- {}\n", name, val));
            }
            _ => {
                output.push_str(&format!("  {} -- not available\n", name));
            }
        }
    }
    // Try AT command via dialer
    match adb_shell(serial, &["am", "start", "-a", "android.intent.action.DIAL", "-d", "tel:%2A%2306%23"]) {
        Ok(_) => output.push_str("  Dialer IMEI check launched (*#06#)\n"),
        Err(_) => {}
    }
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
        match adb_shell(serial, &["dd", &format!("if={}", src), &format!("of={}", dst)]) {
            Ok(_) => output.push_str(&format!("  {} -- backed up\n", part)),
            Err(_) => output.push_str(&format!("  {} -- not found or access denied\n", part)),
        }
    }
    output
}

/// Write IMEI via AT command (requires root or diag mode on some devices).
pub fn write_imei(_serial: &str, imei: &str, manufacturer: &Manufacturer) -> String {
    if imei.len() != 15 || !imei.chars().all(|c| c.is_ascii_digit()) {
        return "Invalid IMEI. Must be exactly 15 digits.".to_string();
    }
    match manufacturer {
        Manufacturer::Samsung => {
            format!(
                "Samsung IMEI write:\n\
                 Method: AT command via diagnostic port.\n\
                 AT+EGMR=1,7,\"{}\"\n\
                 Note: Requires UART/diagnostic mode access.",
                imei
            )
        }
        Manufacturer::Xiaomi | Manufacturer::Oppo | Manufacturer::Realme | Manufacturer::Vivo => {
            format!(
                "Qualcomm/MediaTek IMEI write:\n\
                 Method: Engineering mode or QPST/QFIL.\n\
                 IMEI: {}\n\
                 Note: Requires diagnostic mode (diag port).",
                imei
            )
        }
        _ => {
            format!(
                "IMEI write for {}:\n\
                 IMEI: {}\n\
                 Method varies by chipset. Check platform-specific tools.",
                manufacturer.name(),
                imei
            )
        }
    }
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
    for pkg in GMS_PACKAGES {
        let installed = adb_shell(serial, &["pm", "list", "packages", pkg])
            .map(|out| out.contains(pkg))
            .unwrap_or(false);
        output.push_str(&format!(
            "  {} -- {}\n",
            pkg,
            if installed { "installed" } else { "MISSING" }
        ));
    }
    output
}

/// Clear GMS caches and force restart.
pub fn repair_gms(serial: &str) -> String {
    let mut output = String::from("GMS Repair:\n");
    for pkg in GMS_PACKAGES {
        let _ = adb_shell(serial, &["pm", "clear", pkg]);
        output.push_str(&format!("  Cleared cache: {}\n", pkg));
    }
    let _ = adb_shell(serial, &["am", "force-stop", "com.google.android.gms"]);
    let _ = adb_shell(serial, &["am", "broadcast", "-a", "android.intent.action.BOOT_COMPLETED"]);
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
            let _ = adb_shell(serial, &[
                "tar", "-czf",
                &format!("{}/efs.tar.gz", backup_dir),
                "/efs/",
            ]);
            format!("EFS backup attempt:\n  Contents: {}\n  Saved to: {}/efs.tar.gz", listing, backup_dir)
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
            format!("EFS restore attempted from {}.\nReboot required.", backup_path)
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
    for part in &partitions {
        let src = format!("/dev/block/bootdevice/by-name/{}", part);
        let dst = format!("{}/{}.img", backup_dir, part);
        match adb_shell(serial, &["dd", &format!("if={}", src), &format!("of={}", dst)]) {
            Ok(_) => output.push_str(&format!("  {} -- saved\n", part)),
            Err(_) => output.push_str(&format!("  {} -- failed (root required)\n", part)),
        }
    }
    output
}

/// Restore NV data partitions from backup.
pub fn restore_nv_data(serial: &str) -> String {
    let backup_dir = "/sdcard/FOEM/nv_backup";
    let partitions = ["modemst1", "modemst2", "fsg", "fsc"];
    let mut output = String::from("NV Data Restore:\n");
    for part in &partitions {
        let src = format!("{}/{}.img", backup_dir, part);
        let dst = format!("/dev/block/bootdevice/by-name/{}", part);
        match adb_shell(serial, &["dd", &format!("if={}", src), &format!("of={}", dst)]) {
            Ok(_) => output.push_str(&format!("  {} -- restored\n", part)),
            Err(_) => output.push_str(&format!("  {} -- failed\n", part)),
        }
    }
    output.push_str("  Reboot required.\n");
    output
}

// -- Samsung-Specific Repair --

/// DRK (Device Root Key) repair for Samsung devices.
pub fn repair_drk(serial: &str) -> String {
    let cmds = [
        ("Removing DRK flag", &["rm", "-f", "/efs/prov/cc.dat"][..]),
        ("Clearing DRK data", &["rm", "-rf", "/efs/prov_data/"][..]),
        ("Removing warranty void", &["rm", "-f", "/efs/prov/ridge.dat"][..]),
    ];
    let mut output = String::from("DRK Repair (Samsung):\n");
    for (desc, args) in &cmds {
        match adb_shell(serial, args) {
            Ok(_) => output.push_str(&format!("  {} -- done\n", desc)),
            Err(_) => output.push_str(&format!("  {} -- failed (root required)\n", desc)),
        }
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
    let mut output = String::from("Baseband/Modem Info:\n");
    for (label, prop) in &props {
        match adb_shell(serial, &["getprop", prop]) {
            Ok(val) if !val.is_empty() => output.push_str(&format!("  {}: {}\n", label, val)),
            _ => output.push_str(&format!("  {}: not available\n", label)),
        }
    }
    output
}

/// Attempt baseband repair by clearing modem cache.
pub fn repair_baseband(serial: &str) -> String {
    let mut output = String::from("Baseband Repair:\n");
    let _ = adb_shell(serial, &["setprop", "persist.sys.modem.diag", ",default"]);
    match adb_shell(serial, &["rm", "-rf", "/cache/modem_*"]) {
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
    for (label, prop) in &props {
        match adb_shell(serial, &["getprop", prop]) {
            Ok(val) if !val.is_empty() => output.push_str(&format!("  {}: {}\n", label, val)),
            _ => output.push_str(&format!("  {}: --\n", label)),
        }
    }
    output
}
