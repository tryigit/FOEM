import sys

def main():
    with open("src/app.rs", "r") as f:
        content = f.read()

    target_code = """            if btn(ui, "Relock Bootloader") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::relock(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }"""

    replacement_code = """            if btn(ui, "Relock Bootloader") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::relock(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Attempt Locked Root") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::attempt_locked_root(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }"""

    if target_code in content:
        content = content.replace(target_code, replacement_code)
        with open("src/app.rs", "w") as f:
            f.write(content)
        print("Patched app.rs")
    else:
        print("Target code not found")

if __name__ == "__main__":
    main()
