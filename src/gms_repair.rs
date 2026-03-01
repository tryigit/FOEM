/// Google Mobile Services repair logic.
use std::collections::BTreeMap;

const GMS_PACKAGES: &[&str] = &[
    "com.google.android.gms",
    "com.google.android.gsf",
    "com.android.vending",
    "com.google.android.apps.setup",
];

pub struct GMSRepairManager {
    device_serial: String,
}

impl GMSRepairManager {
    pub fn new(serial: String) -> Self {
        Self {
            device_serial: serial,
        }
    }

    /// Run an ADB shell command on the target device.
    fn adb_shell(&self, args: &[&str]) -> Option<String> {
        let mut cmd = std::process::Command::new("adb");
        cmd.args(["-s", &self.device_serial, "shell"]);
        cmd.args(args);
        cmd.output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Check which GMS packages are installed.
    pub fn check_gms_status(&self) -> BTreeMap<String, String> {
        let mut results = BTreeMap::new();
        for pkg in GMS_PACKAGES {
            let installed = self
                .adb_shell(&["pm", "list", "packages", pkg])
                .map(|out| out.contains(pkg))
                .unwrap_or(false);
            results.insert(
                pkg.to_string(),
                if installed { "installed" } else { "missing" }.to_string(),
            );
        }
        results
    }

    /// Clear cache for all GMS packages.
    pub fn clear_gms_cache(&self) {
        for pkg in GMS_PACKAGES {
            let _ = self.adb_shell(&["pm", "clear", pkg]);
        }
    }

    /// Run the full GMS repair sequence.
    pub fn repair_gms(&self) -> BTreeMap<String, String> {
        let status = self.check_gms_status();
        self.clear_gms_cache();

        // Force-stop and restart GMS core
        let _ = self.adb_shell(&["am", "force-stop", "com.google.android.gms"]);
        let _ = self.adb_shell(&[
            "am", "start", "-n",
            "com.google.android.gms/.app.settings.GoogleSettingsActivity",
        ]);

        status
    }
}
