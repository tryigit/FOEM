/// ADB utility tools: shell, logcat, file operations, reboot,
/// backup/restore, APK management, bloatware removal, screenshots.

use super::{adb, adb_shell};

// -- ADB Shell --

/// Execute an arbitrary ADB shell command.
pub fn execute_shell(serial: &str, command: &str) -> String {
    if command.is_empty() {
        return "No command entered.".to_string();
    }
    let args: Vec<&str> = command.split_whitespace().collect();
    match adb_shell(serial, &args) {
        Ok(out) => {
            if out.is_empty() {
                "(command returned no output)".to_string()
            } else {
                out
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

// -- Logcat --

/// Capture logcat output (limited to recent lines).
pub fn capture_logcat(serial: &str, lines: usize) -> String {
    let line_count = format!("{}", lines);
    match adb(serial, &["logcat", "-d", "-t", &line_count]) {
        Ok(out) => format!("Logcat (last {} lines):\n{}", lines, out),
        Err(e) => format!("Logcat failed: {}", e),
    }
}

/// Clear logcat buffer.
pub fn clear_logcat(serial: &str) -> String {
    match adb(serial, &["logcat", "-c"]) {
        Ok(_) => "Logcat buffer cleared.".to_string(),
        Err(e) => format!("Clear logcat failed: {}", e),
    }
}

// -- File Manager --

/// Pull a file from the device to the local machine.
pub fn pull_file(serial: &str, remote_path: &str, local_path: &str) -> String {
    if remote_path.is_empty() || local_path.is_empty() {
        return "Both remote and local paths are required.".to_string();
    }
    match adb(serial, &["pull", remote_path, local_path]) {
        Ok(out) => format!("Pull result:\n{}", out),
        Err(e) => format!("Pull failed: {}", e),
    }
}

/// Push a file from the local machine to the device.
pub fn push_file(serial: &str, local_path: &str, remote_path: &str) -> String {
    if local_path.is_empty() || remote_path.is_empty() {
        return "Both local and remote paths are required.".to_string();
    }
    match adb(serial, &["push", local_path, remote_path]) {
        Ok(out) => format!("Push result:\n{}", out),
        Err(e) => format!("Push failed: {}", e),
    }
}

/// List files in a directory on the device.
pub fn list_files(serial: &str, path: &str) -> String {
    let dir = if path.is_empty() { "/sdcard/" } else { path };
    match adb_shell(serial, &["ls", "-la", dir]) {
        Ok(out) => format!("{}:\n{}", dir, out),
        Err(e) => format!("List failed: {}", e),
    }
}

// -- APK Management --

/// Install an APK from the local machine.
pub fn install_apk(serial: &str, apk_path: &str) -> String {
    if apk_path.is_empty() {
        return "APK file path is required.".to_string();
    }
    match adb(serial, &["install", "-r", "-d", apk_path]) {
        Ok(out) => format!("Install result:\n{}", out),
        Err(e) => format!("Install failed: {}", e),
    }
}

/// List installed packages (optionally filtered).
pub fn list_packages(serial: &str, filter: &str) -> String {
    let mut args = vec!["pm", "list", "packages"];
    if !filter.is_empty() {
        args.push(filter);
    }
    match adb_shell(serial, &args) {
        Ok(out) => {
            let count = out.lines().count();
            format!("Packages ({}):\n{}", count, out)
        }
        Err(e) => format!("List packages failed: {}", e),
    }
}

/// List only third-party (user-installed) packages.
pub fn list_user_packages(serial: &str) -> String {
    match adb_shell(serial, &["pm", "list", "packages", "-3"]) {
        Ok(out) => {
            let count = out.lines().count();
            format!("User-installed packages ({}):\n{}", count, out)
        }
        Err(e) => format!("List failed: {}", e),
    }
}

/// List system packages.
pub fn list_system_packages(serial: &str) -> String {
    match adb_shell(serial, &["pm", "list", "packages", "-s"]) {
        Ok(out) => {
            let count = out.lines().count();
            format!("System packages ({}):\n{}", count, out)
        }
        Err(e) => format!("List failed: {}", e),
    }
}

// -- Bloatware Removal --

/// Disable a system app for the current user (no root required).
pub fn disable_package(serial: &str, package: &str) -> String {
    if package.is_empty() {
        return "Package name is required.".to_string();
    }
    match adb_shell(serial, &["pm", "uninstall", "-k", "--user", "0", package]) {
        Ok(out) => format!("Disable '{}': {}", package, out),
        Err(e) => format!("Disable '{}' failed: {}", package, e),
    }
}

/// Re-enable a previously disabled package.
pub fn enable_package(serial: &str, package: &str) -> String {
    if package.is_empty() {
        return "Package name is required.".to_string();
    }
    match adb_shell(serial, &["cmd", "package", "install-existing", package]) {
        Ok(out) => format!("Enable '{}': {}", package, out),
        Err(e) => format!("Enable '{}' failed: {}", package, e),
    }
}

// -- Backup and Restore --

/// Full device backup via ADB backup.
pub fn full_backup(serial: &str, backup_path: &str) -> String {
    let path = if backup_path.is_empty() { "foem_backup.ab" } else { backup_path };
    match adb(serial, &["backup", "-all", "-apk", "-shared", "-f", path]) {
        Ok(out) => format!("Backup initiated to '{}'.\nConfirm on device screen.\n{}", path, out),
        Err(e) => format!("Backup failed: {}", e),
    }
}

/// Full device restore from ADB backup.
pub fn full_restore(serial: &str, backup_path: &str) -> String {
    if backup_path.is_empty() {
        return "Backup file path is required.".to_string();
    }
    match adb(serial, &["restore", backup_path]) {
        Ok(out) => format!("Restore initiated from '{}'.\nConfirm on device screen.\n{}", backup_path, out),
        Err(e) => format!("Restore failed: {}", e),
    }
}

// -- Screenshot and Recording --

/// Take a device screenshot and pull it to local machine.
pub fn take_screenshot(serial: &str, local_path: &str) -> String {
    let device_path = "/sdcard/FOEM/screenshot.png";
    let local = if local_path.is_empty() { "screenshot.png" } else { local_path };
    match adb_shell(serial, &["screencap", "-p", device_path]) {
        Ok(_) => match adb(serial, &["pull", device_path, local]) {
            Ok(out) => format!("Screenshot saved to '{}'.\n{}", local, out),
            Err(e) => format!("Screenshot taken but pull failed: {}", e),
        },
        Err(e) => format!("Screenshot failed: {}", e),
    }
}

/// Start screen recording on device.
pub fn start_screen_record(serial: &str) -> String {
    let device_path = "/sdcard/FOEM/screenrecord.mp4";
    match adb_shell(serial, &["screenrecord", "--time-limit", "180", device_path]) {
        Ok(out) => format!("Recording saved to device: {}\n{}", device_path, out),
        Err(e) => format!("Screen recording failed: {}\nNote: Some devices restrict screen recording.", e),
    }
}

// -- Device Reboot --

/// Reboot the device to system.
pub fn reboot(serial: &str) -> String {
    match adb(serial, &["reboot"]) {
        Ok(_) => "Device rebooting to system.".to_string(),
        Err(e) => format!("Reboot failed: {}", e),
    }
}

/// Reboot to recovery mode.
pub fn reboot_recovery(serial: &str) -> String {
    match adb(serial, &["reboot", "recovery"]) {
        Ok(_) => "Device rebooting to recovery.".to_string(),
        Err(e) => format!("Reboot to recovery failed: {}", e),
    }
}

/// Reboot to bootloader/fastboot mode.
pub fn reboot_bootloader(serial: &str) -> String {
    match adb(serial, &["reboot", "bootloader"]) {
        Ok(_) => "Device rebooting to bootloader/fastboot.".to_string(),
        Err(e) => format!("Reboot to bootloader failed: {}", e),
    }
}

/// Enable developer options by simulating build number taps.
pub fn enable_developer_options(serial: &str) -> String {
    // Open About Phone
    let _ = adb_shell(serial, &[
        "am", "start", "-a", "android.settings.DEVICE_INFO_SETTINGS",
    ]);
    "Developer Options:\n\
     Opened device info settings.\n\
     Tap 'Build Number' 7 times to enable Developer Options.\n\
     Then enable 'USB Debugging' in Developer Options."
        .to_string()
}

/// Get device uptime.
pub fn get_uptime(serial: &str) -> String {
    match adb_shell(serial, &["uptime"]) {
        Ok(val) => format!("Device uptime: {}", val),
        Err(e) => format!("Uptime check failed: {}", e),
    }
}

/// Get running processes.
pub fn get_processes(serial: &str) -> String {
    match adb_shell(serial, &["ps", "-A"]) {
        Ok(val) => {
            let count = val.lines().count();
            let summary = if val.len() > 2000 { &val[..2000] } else { &val };
            format!("Running processes ({}):\n{}\n...", count, summary)
        }
        Err(e) => format!("Process list failed: {}", e),
    }
}

/// Get memory information.
pub fn get_memory_info(serial: &str) -> String {
    match adb_shell(serial, &["cat", "/proc/meminfo"]) {
        Ok(val) => {
            let mut output = String::from("Memory Info:\n");
            for line in val.lines().take(10) {
                output.push_str(&format!("  {}\n", line));
            }
            output
        }
        Err(e) => format!("Memory info failed: {}", e),
    }
}

/// Get CPU information.
pub fn get_cpu_info(serial: &str) -> String {
    match adb_shell(serial, &["cat", "/proc/cpuinfo"]) {
        Ok(val) => {
            let mut output = String::from("CPU Info:\n");
            for line in val.lines().take(20) {
                output.push_str(&format!("  {}\n", line));
            }
            output
        }
        Err(e) => format!("CPU info failed: {}", e),
    }
}


/// Start screen mirroring using scrcpy.
pub fn start_scrcpy(serial: &str) -> String {
    match std::process::Command::new("scrcpy")
        .arg("-s")
        .arg(serial)
        .spawn() {
        Ok(_) => format!("Launched scrcpy for device {}", serial),
        Err(e) => format!("Failed to launch scrcpy: {}\nIs scrcpy installed on your system?", e),
    }
}