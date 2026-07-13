import sys

def main():
    with open("src/features/bootloader.rs", "r") as f:
        content = f.read()

    new_fn = """
/// Exploit security vulnerability in some devices (e.g. 8 Elite Gen 5) to bypass bootloader unlock restrictions.
/// This method only works for devices without the February security patch.
pub fn bypass_unlock(serial: &str) -> String {
    let mut log = String::new();

    // Switch to fastboot mode (assuming device is in adb mode)
    log.push_str("Switching to bootloader...\\n");
    let _ = super::adb(serial, &["reboot", "bootloader"]);
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Run the exploit commands
    log.push_str("Running fastboot oem set-gpu-preemption-value 0 androidboot.selinux=permissive...\\n");
    match super::fastboot(serial, &["oem", "set-gpu-preemption-value", "0", "androidboot.selinux=permissive"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\n", e)),
    }

    log.push_str("Continuing boot...\\n");
    let _ = super::fastboot(serial, &["continue"]);
    std::thread::sleep(std::time::Duration::from_secs(15));

    // Push the efi unlock file (assuming it's present in the app's directory or bundled)
    // Note: This relies on the file gbl_efi_unlock.efi being available
    log.push_str("Pushing gbl_efi_unlock.efi to /data/local/tmp/...\\n");
    match super::adb(serial, &["push", "gbl_efi_unlock.efi", "/data/local/tmp"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\nMake sure gbl_efi_unlock.efi is in the same directory as the executable.\\n", e)),
    }

    // Run the service call
    log.push_str("Calling miui.mqsas.IMQSNative...\\n");
    match super::adb_shell(serial, &["service", "call", "miui.mqsas.IMQSNative", "21", "i32", "1", "s16", "dd", "i32", "1", "s16", "if=/data/local/tmp/gbl_efi_unlock.efi of=/dev/block/by-name/efisp", "s16", "/data/mqsas/log.txt", "i32", "60"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\n", e)),
    }

    log.push_str("Rebooting to bootloader...\\n");
    let _ = super::adb(serial, &["reboot", "bootloader"]);
    std::thread::sleep(std::time::Duration::from_secs(5));

    log.push_str("Checking unlock status...\\n");
    match super::fastboot(serial, &["getvar", "unlocked"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\n", e)),
    }

    log.push_str("Erasing efips...\\n");
    let _ = super::fastboot(serial, &["erase", "efips"]);

    log.push_str("Rebooting device...\\n");
    let _ = super::fastboot(serial, &["reboot"]);

    log.push_str("Done.");

    log
}
"""

    if "pub fn bypass_unlock" not in content:
        with open("src/features/bootloader.rs", "a") as f:
            f.write(new_fn)
        print("Patched bootloader.rs with bypass_unlock")
    else:
        print("Already patched")

if __name__ == "__main__":
    main()
