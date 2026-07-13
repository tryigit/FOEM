import sys

def main():
    with open("src/app.rs", "r") as f:
        content = f.read()

    target_code = """                if btn(ui, "Screenshot") {
                    if let Ok(s) = self.require_device() { self.log = features::tools::take_screenshot(s, &self.local_path); }
                    else { self.log = "Connect a device first.".into(); }
                }"""

    replacement_code = """                if btn(ui, "Screenshot") {
                    if let Ok(s) = self.require_device() { self.log = features::tools::take_screenshot(s, &self.local_path); }
                    else { self.log = "Connect a device first.".into(); }
                }
                if btn(ui, "Screen Mirror (scrcpy)") {
                    if let Ok(s) = self.require_device() { self.log = features::tools::start_scrcpy(s); }
                    else { self.log = "Connect a device first.".into(); }
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
