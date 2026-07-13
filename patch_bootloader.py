import sys

def main():
    with open("src/features/bootloader.rs", "r") as f:
        content = f.read()

    insert_str = """

/// Attempt to root the device even with a locked bootloader.
/// This will try `adb root`. If it fails, it provides an informative message
/// about the difficulty of this process.
pub fn attempt_locked_root(serial: &str) -> String {
    match super::adb(serial, &["root"]) {
        Ok(out) => {
            if out.contains("cannot run as root") {
                "Attempting to root with a locked bootloader requires specific vulnerabilities (exploits) \
                 for your device's exact firmware version. There is no universal method.\\n\
                 Please research your specific model (e.g., MTK-SU for older MediaTek devices, \
                 or specific Qualcomm EDL exploits).".to_string()
            } else {
                format!("ADB root command sent.\\nOutput: {}", out)
            }
        }
        Err(e) => format!("Error sending adb root command: {}", e),
    }
}"""

    if "pub fn attempt_locked_root" not in content:
        with open("src/features/bootloader.rs", "a") as f:
            f.write(insert_str)
        print("Patched bootloader.rs")
    else:
        print("Already patched")

if __name__ == "__main__":
    main()
