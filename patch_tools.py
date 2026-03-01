import sys

def main():
    with open("src/features/tools.rs", "r") as f:
        content = f.read()

    insert_str = """

/// Start screen mirroring using scrcpy.
pub fn start_scrcpy(serial: &str) -> String {
    match std::process::Command::new("scrcpy")
        .arg("-s")
        .arg(serial)
        .spawn() {
        Ok(_) => format!("Launched scrcpy for device {}", serial),
        Err(e) => format!("Failed to launch scrcpy: {}\\nIs scrcpy installed on your system?", e),
    }
}"""

    if "pub fn start_scrcpy" not in content:
        with open("src/features/tools.rs", "a") as f:
            f.write(insert_str)
        print("Patched tools.rs")
    else:
        print("Already patched")

if __name__ == "__main__":
    main()
