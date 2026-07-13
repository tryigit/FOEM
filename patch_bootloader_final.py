import sys

def main():
    with open("src/features/bootloader.rs", "r") as f:
        content = f.read()

    new_fn = """
/// Exploit security vulnerability in some devices (e.g. 8 Elite Gen 5) to bypass bootloader unlock restrictions.
/// This method only works for devices without the February security patch.
pub fn bypass_unlock(serial: &str) -> String {
    let mut log = String::new();

    log.push_str("Switching to bootloader...\\n");
    let _ = super::adb(serial, &["reboot", "bootloader"]);
    std::thread::sleep(std::time::Duration::from_secs(5));

    log.push_str("Running fastboot oem set-gpu-preemption-value 0 androidboot.selinux=permissive...\\n");
    match super::fastboot(serial, &["oem", "set-gpu-preemption-value", "0", "androidboot.selinux=permissive"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\n", e)),
    }

    log.push_str("Continuing boot...\\n");
    let _ = super::fastboot(serial, &["continue"]);
    std::thread::sleep(std::time::Duration::from_secs(15));

    log.push_str("Pushing gbl_efi_unlock.efi to /data/local/tmp/...\\n");
    match super::adb(serial, &["push", "D:\\\\unlock\\\\data\\\\mqsas\\\\gbl_efi_unlock.efi", "/data/local/tmp"]) {
        Ok(out) => log.push_str(&format!("Result: {}\\n", out)),
        Err(e) => log.push_str(&format!("Error: {}\\n", e)),
    }

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

    # We need to replace the old bypass_unlock with the new one
    if "pub fn bypass_unlock" in content:
        # It's already there, let's remove it and add the new one
        start_idx = content.find("pub fn bypass_unlock")
        # Find the last closing brace of bypass_unlock function
        end_idx = content.find("}\n\n", start_idx)
        if end_idx == -1:
             end_idx = content.find("}\n", start_idx)

        # Remove old function completely
        lines = content.split('\n')
        new_lines = []
        skip = False
        brace_count = 0
        found_start = False
        for line in lines:
            if "pub fn bypass_unlock" in line:
                skip = True
                found_start = True
                brace_count = line.count('{') - line.count('}')
                continue

            if skip:
                brace_count += line.count('{') - line.count('}')
                if brace_count <= 0:
                    skip = False
                continue

            new_lines.append(line)

        content = '\n'.join(new_lines)

    content += new_fn

    with open("src/features/bootloader.rs", "w") as f:
        f.write(content)
    print("Patched bootloader.rs with final bypass_unlock")

if __name__ == "__main__":
    main()
