"""Google Mobile Services repair logic."""

import logging
import subprocess

logger = logging.getLogger(__name__)

GMS_PACKAGES = [
    "com.google.android.gms",
    "com.google.android.gsf",
    "com.android.vending",
    "com.google.android.apps.setup",
]


class GMSRepairManager:
    """Handles diagnosis and repair of Google Mobile Services on a device."""

    def __init__(self, device_serial):
        self.device_serial = device_serial

    def _adb_shell(self, command):
        """Run an ADB shell command on the connected device."""
        try:
            result = subprocess.run(
                ["adb", "-s", self.device_serial, "shell"] + command.split(),
                capture_output=True,
                text=True,
                timeout=30,
            )
            return result.stdout.strip(), result.returncode
        except (FileNotFoundError, subprocess.TimeoutExpired) as exc:
            logger.warning("ADB command failed: %s", exc)
            return "", 1

    def check_gms_status(self):
        """Check which GMS packages are installed and their status."""
        logger.info("Checking GMS status on %s...", self.device_serial)
        results = {}

        for package in GMS_PACKAGES:
            output, code = self._adb_shell(f"pm list packages {package}")
            installed = package in output if code == 0 else False
            results[package] = "installed" if installed else "missing"
            logger.info("  %s: %s", package, results[package])

        return results

    def clear_gms_cache(self):
        """Clear cache for all GMS packages."""
        logger.info("Clearing GMS caches...")
        for package in GMS_PACKAGES:
            self._adb_shell(f"pm clear {package}")
            logger.info("  Cleared cache for %s", package)

    def repair_gms(self):
        """Run the GMS repair sequence."""
        logger.info("Initiating GMS repair for %s...", self.device_serial)

        status = self.check_gms_status()
        missing = [pkg for pkg, state in status.items() if state == "missing"]

        if missing:
            logger.warning("Missing GMS packages: %s", missing)
            logger.info(
                "Manual installation of missing packages may be required."
            )

        self.clear_gms_cache()

        # Force-stop and restart GMS core
        self._adb_shell("am force-stop com.google.android.gms")
        self._adb_shell(
            "am start -n com.google.android.gms/.app.settings.GoogleSettingsActivity"
        )

        logger.info("GMS repair sequence completed.")
        return status
