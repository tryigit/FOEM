/// ADB utility tools: shell, logcat, file operations, reboot,
/// backup/restore, APK management, bloatware removal, screenshots.
use super::{adb, adb_shell};

// -- ADB Shell --

/// Execute an arbitrary ADB shell command.
pub fn execute_shell(serial: &str, command: &str) -> String {
    execute_shell_internal(serial, command, adb_shell)
}
fn execute_shell_internal<F>(serial: &str, command: &str, adb_shell_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    if command.is_empty() {
        return "No command entered.".to_string();
    }
    let args: Vec<&str> = command.split_whitespace().collect();
    match adb_shell_fn(serial, &args) {
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
    install_apk_internal(serial, apk_path, super::adb)
}

fn install_apk_internal<F>(serial: &str, apk_path: &str, adb_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    if apk_path.is_empty() {
        return "APK file path is required.".to_string();
    }
    match adb_fn(serial, &["install", "-r", "-d", apk_path]) {
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
    list_user_packages_internal(serial, adb_shell)
}
fn list_user_packages_internal<F>(serial: &str, adb_shell_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    match adb_shell_fn(serial, &["pm", "list", "packages", "-3"]) {
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
    enable_package_internal(serial, package, adb_shell)
}
fn enable_package_internal<F>(serial: &str, package: &str, adb_shell_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    if package.is_empty() {
        return "Package name is required.".to_string();
    }
    match adb_shell_fn(serial, &["cmd", "package", "install-existing", package]) {
        Ok(out) => format!("Enable '{}': {}", package, out),
        Err(e) => format!("Enable '{}' failed: {}", package, e),
    }
}
// -- Backup and Restore --

/// Full device backup via ADB backup.
pub fn full_backup(serial: &str, backup_path: &str) -> String {
    full_backup_internal(serial, backup_path, adb)
}
fn full_backup_internal<F>(serial: &str, backup_path: &str, adb_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    let path = if backup_path.is_empty() {
        "foem_backup.ab"
    } else {
        backup_path
    };
    match adb_fn(serial, &["backup", "-all", "-apk", "-shared", "-f", path]) {
        Ok(out) => format!(
            "Backup initiated to '{}'.
Confirm on device screen.
{}",
            path, out
        ),
        Err(e) => format!("Backup failed: {}", e),
    }
}
/// Full device restore from ADB backup.
pub fn full_restore(serial: &str, backup_path: &str) -> String {
    if backup_path.is_empty() {
        return "Backup file path is required.".to_string();
    }
    match adb(serial, &["restore", backup_path]) {
        Ok(out) => format!(
            "Restore initiated from '{}'.\nConfirm on device screen.\n{}",
            backup_path, out
        ),
        Err(e) => format!("Restore failed: {}", e),
    }
}
// -- Screenshot and Recording --

/// Take a device screenshot and pull it to local machine.
pub fn take_screenshot(serial: &str, local_path: &str) -> String {
    let device_path = "/sdcard/FOEM/screenshot.png";
    let local = if local_path.is_empty() {
        "screenshot.png"
    } else {
        local_path
    };
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
    match adb_shell(
        serial,
        &["screenrecord", "--time-limit", "180", device_path],
    ) {
        Ok(out) => format!("Recording saved to device: {}\n{}", device_path, out),
        Err(e) => format!(
            "Screen recording failed: {}\nNote: Some devices restrict screen recording.",
            e
        ),
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
    reboot_bootloader_internal(serial, adb)
}

fn reboot_bootloader_internal<F>(serial: &str, adb_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    match adb_fn(serial, &["reboot", "bootloader"]) {
        Ok(_) => "Device rebooting to bootloader/fastboot.".to_string(),
        Err(e) => format!("Reboot to bootloader failed: {}", e),
    }
}
/// Enable developer options by simulating build number taps.
pub fn enable_developer_options(serial: &str) -> String {
    // Open About Phone
    let _ = adb_shell(
        serial,
        &["am", "start", "-a", "android.settings.DEVICE_INFO_SETTINGS"],
    );
    "Developer Options:\n\
     Opened device info settings.\n\
     Tap 'Build Number' 7 times to enable Developer Options.\n\
     Then enable 'USB Debugging' in Developer Options."
        .to_string()
}
/// Get device uptime.
pub fn get_uptime(serial: &str) -> String {
    get_uptime_internal(serial, adb_shell)
}

fn get_uptime_internal<F>(serial: &str, adb_shell_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    match adb_shell_fn(serial, &["uptime"]) {
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
    get_cpu_info_internal(serial, adb_shell)
}

fn get_cpu_info_internal<F>(serial: &str, adb_shell_fn: F) -> String
where
    F: Fn(&str, &[&str]) -> Result<String, String>,
{
    match adb_shell_fn(serial, &["cat", "/proc/cpuinfo"]) {
        Ok(val) => {
            let mut output = String::from("CPU Info:
");
            for line in val.lines().take(20) {
                output.push_str(&format!("  {}
", line));
            }
            output
        }
        Err(e) => format!("CPU info failed: {}", e),
    }
}
/// Start screen mirroring using scrcpy.
pub fn start_scrcpy(serial: &str) -> String {
    start_scrcpy_with_cmd("scrcpy", serial)
}
/// Internal function to start scrcpy, allowing dependency injection of the command name for testing.
fn start_scrcpy_with_cmd(cmd: &str, serial: &str) -> String {
    match std::process::Command::new(cmd)
        .arg("-s")
        .arg(serial)
        .spawn()
    {
        Ok(_) => format!("Launched scrcpy for device {}", serial),
        Err(e) => format!(
            "Failed to launch scrcpy: {}\nIs scrcpy installed on your system?",
            e
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_shell_internal_empty_command() {
        let result = execute_shell_internal("device1", "", |_, _| {
            panic!("Should not be called");
        });
        assert_eq!(result, "No command entered.");
    }
    #[test]
    fn test_execute_shell_internal_success() {
        let result = execute_shell_internal("device1", "ls -la", |serial, args| {
            assert_eq!(serial, "device1");
            assert_eq!(args, &["ls", "-la"]);
            Ok("file1\nfile2".to_string())
        });
        assert_eq!(result, "file1\nfile2");
    }
    #[test]
    fn test_execute_shell_internal_success_empty_output() {
        let result = execute_shell_internal("device1", "touch test.txt", |serial, args| {
            assert_eq!(serial, "device1");
            assert_eq!(args, &["touch", "test.txt"]);
            Ok("".to_string())
        });
        assert_eq!(result, "(command returned no output)");
    }
    #[test]
    fn test_execute_shell_internal_failure() {
        let result = execute_shell_internal("device1", "badcmd", |serial, args| {
            assert_eq!(serial, "device1");
            assert_eq!(args, &["badcmd"]);
            Err("command not found".to_string())
        });
        assert_eq!(result, "Error: command not found");
    }
    #[test]
    fn test_enable_package_empty() {
        let result = enable_package_internal("device_123", "", |_, _| {
            panic!("Should not be called");
        });
        assert_eq!(result, "Package name is required.");
    }
    #[test]
    fn test_enable_package_success() {
        let package = "com.example.app";
        let result = enable_package_internal("device_123", package, |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(
                args,
                &["cmd", "package", "install-existing", "com.example.app"]
            );
            Ok("Package com.example.app installed for user: 0".to_string())
        });
        assert_eq!(
            result,
            format!(
                "Enable '{}': Package com.example.app installed for user: 0",
                package
            )
        );
    }
    #[test]
    fn test_enable_package_failure() {
        let package = "com.example.app";
        let result = enable_package_internal("device_123", package, |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(
                args,
                &["cmd", "package", "install-existing", "com.example.app"]
            );
            Err("error: device not found".to_string())
        });
        assert_eq!(
            result,
            format!("Enable '{}' failed: error: device not found", package)
        );
    }
    #[test]
    fn test_start_scrcpy_with_cmd_success() {
        let serial = "test_device_success";

        // Using "cargo" as a dummy command since it is available cross-platform during test runs.
        let result = start_scrcpy_with_cmd("cargo", serial);
        assert_eq!(result, format!("Launched scrcpy for device {}", serial));
    }
    #[test]
    fn test_start_scrcpy_with_cmd_failure() {
        let serial = "test_device_failure";

        // A command that does not exist should fail to spawn.
        let result = start_scrcpy_with_cmd("this_command_does_not_exist_12345", serial);
        assert!(
            result.starts_with("Failed to launch scrcpy: "),
            "Expected failure message, got: {}",
            result
        );
        assert!(
            result.contains("Is scrcpy installed on your system?"),
            "Expected troubleshooting hint, got: {}",
            result
        );
    }

    #[test]
    fn test_list_user_packages_success() {
        let result = list_user_packages_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["pm", "list", "packages", "-3"]);
            Ok("package:com.example.app1\npackage:com.example.app2".to_string())
        });
        assert_eq!(
            result,
            "User-installed packages (2):\npackage:com.example.app1\npackage:com.example.app2"
        );
    }
    #[test]
    fn test_list_user_packages_empty() {
        let result = list_user_packages_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["pm", "list", "packages", "-3"]);
            Ok("".to_string())
        });
        assert_eq!(result, "User-installed packages (0):\n");
    }
    #[test]
    fn test_list_user_packages_failure() {
        let result = list_user_packages_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["pm", "list", "packages", "-3"]);
            Err("device offline".to_string())
        });
        assert_eq!(result, "List failed: device offline");
    }
    #[test]
    fn test_full_backup_internal_default_path() {
        let result = full_backup_internal("device_123", "", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(
                args,
                &["backup", "-all", "-apk", "-shared", "-f", "foem_backup.ab"]
            );
            Ok("adb output".to_string())
        });
        assert_eq!(
            result,
            "Backup initiated to 'foem_backup.ab'.
Confirm on device screen.
adb output"
        );
    }

    #[test]
    fn test_full_backup_internal_custom_path() {
        let result = full_backup_internal("device_123", "custom_backup.ab", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(
                args,
                &[
                    "backup",
                    "-all",
                    "-apk",
                    "-shared",
                    "-f",
                    "custom_backup.ab"
                ]
            );
            Ok("adb output".to_string())
        });
        assert_eq!(
            result,
            "Backup initiated to 'custom_backup.ab'.
Confirm on device screen.
adb output"
        );
    }

    #[test]
    fn test_full_backup_internal_failure() {
        let result = full_backup_internal("device_123", "custom_backup.ab", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(
                args,
                &[
                    "backup",
                    "-all",
                    "-apk",
                    "-shared",
                    "-f",
                    "custom_backup.ab"
                ]
            );
            Err("device disconnected".to_string())
        });
        assert_eq!(result, "Backup failed: device disconnected");
    }

    #[test]
    fn test_get_uptime_success() {
        let result = get_uptime_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["uptime"]);
            Ok("10:00:00 up 1 day, 2:00".to_string())
        });
        assert_eq!(result, "Device uptime: 10:00:00 up 1 day, 2:00");
    }

    #[test]
    fn test_get_uptime_failure() {
        let result = get_uptime_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["uptime"]);
            Err("device offline".to_string())
        });
        assert_eq!(result, "Uptime check failed: device offline");
    }
    #[test]
    fn test_install_apk_empty_path() {
        let result = install_apk_internal("device_123", "", |_, _| {
            panic!("Should not be called");
        });
        assert_eq!(result, "APK file path is required.");
    }

    #[test]
    fn test_install_apk_success() {
        let result = install_apk_internal("device_123", "app.apk", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["install", "-r", "-d", "app.apk"]);
            Ok("Success".to_string())
        });
        assert_eq!(result, "Install result:\nSuccess");
    }

    #[test]
    fn test_install_apk_failure() {
        let result = install_apk_internal("device_123", "app.apk", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["install", "-r", "-d", "app.apk"]);
            Err("error: device not found".to_string())
        });
        assert_eq!(result, "Install failed: error: device not found");
    }

    #[test]
    fn test_reboot_bootloader_success() {
        let result = reboot_bootloader_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["reboot", "bootloader"]);
            Ok("".to_string())
        });
        assert_eq!(result, "Device rebooting to bootloader/fastboot.");
    }

    #[test]
    fn test_reboot_bootloader_failure() {
        let result = reboot_bootloader_internal("device_123", |serial, args| {
            assert_eq!(serial, "device_123");
            assert_eq!(args, &["reboot", "bootloader"]);
            Err("error: device not found".to_string())
        });
        assert_eq!(
            result,
            "Reboot to bootloader failed: error: device not found"
        );
    }
}
