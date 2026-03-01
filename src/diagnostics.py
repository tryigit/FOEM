"""Device detection and diagnostic utilities."""

import logging
import subprocess
import shutil

logger = logging.getLogger(__name__)


class DeviceDiagnostics:
    """Handles device detection and basic diagnostics via ADB/Fastboot."""

    def __init__(self):
        self.connected_device = None
        self.device_info = {}

    def _run_command(self, args):
        """Run a shell command and return its output."""
        try:
            result = subprocess.run(
                args,
                capture_output=True,
                text=True,
                timeout=15,
            )
            return result.stdout.strip(), result.returncode
        except FileNotFoundError:
            logger.warning("Command not found: %s", args[0])
            return "", 1
        except subprocess.TimeoutExpired:
            logger.warning("Command timed out: %s", " ".join(args))
            return "", 1

    def is_adb_available(self):
        """Check if ADB is installed and accessible."""
        return shutil.which("adb") is not None

    def is_fastboot_available(self):
        """Check if Fastboot is installed and accessible."""
        return shutil.which("fastboot") is not None

    def detect_device(self):
        """Detect connected device via ADB."""
        logger.info("Detecting connected device...")

        if not self.is_adb_available():
            logger.error("ADB is not installed or not in PATH.")
            return None

        output, code = self._run_command(["adb", "devices"])
        if code != 0:
            logger.error("Failed to run adb devices.")
            return None

        lines = output.strip().splitlines()
        devices = []
        for line in lines[1:]:
            parts = line.split()
            if len(parts) >= 2 and parts[1] == "device":
                devices.append(parts[0])

        if not devices:
            logger.info("No devices detected.")
            return None

        self.connected_device = devices[0]
        logger.info("Device detected: %s", self.connected_device)
        return self.connected_device

    def get_device_info(self):
        """Retrieve basic device properties via ADB."""
        if not self.connected_device:
            logger.error("No device connected.")
            return {}

        props = {
            "manufacturer": "ro.product.manufacturer",
            "model": "ro.product.model",
            "android_version": "ro.build.version.release",
            "sdk_version": "ro.build.version.sdk",
            "serial": "ro.serialno",
        }

        info = {}
        for key, prop in props.items():
            output, code = self._run_command(
                ["adb", "-s", self.connected_device, "shell", "getprop", prop]
            )
            if code == 0 and output:
                info[key] = output

        self.device_info = info
        return info

    def run_health_check(self):
        """Run a basic health check on the connected device."""
        if not self.connected_device:
            logger.error("No device connected.")
            return False

        logger.info("Running health check for %s...", self.connected_device)
        info = self.get_device_info()

        if not info:
            logger.warning("Could not retrieve device info.")
            return False

        logger.info("Device info: %s", info)
        return True
