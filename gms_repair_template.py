import logging
import subprocess

class UpdateManager:
    def __init__(self, device):
        self.device = device

    def repair_gms(self):
        """Initiates the GMS repair process."""
        logging.info(f"Initiating GMS repair for {self.device}...")

        gms_packages = [
            "com.google.android.gms",
            "com.google.android.gsf",
            "com.android.vending",
            "com.google.android.apps.setup",
            "com.google.android.setupwizard"
        ]

        try:
            for pkg in gms_packages:
                logging.info(f"Clearing cache for {pkg}...")
                subprocess.run(["adb", "-s", self.device, "shell", "pm", "clear", pkg], check=True, capture_output=True, text=True)

            logging.info("Force-stopping GMS...")
            subprocess.run(["adb", "-s", self.device, "shell", "am", "force-stop", "com.google.android.gms"], check=True, capture_output=True, text=True)

            logging.info("Sending BOOT_COMPLETED broadcast...")
            subprocess.run(["adb", "-s", self.device, "shell", "am", "broadcast", "-a", "android.intent.action.BOOT_COMPLETED"], check=True, capture_output=True, text=True)

        except subprocess.CalledProcessError as e:
            logging.error(f"Command failed during GMS repair: {e.stderr}")
            return False
        except Exception as e:
            logging.error(f"Unexpected error during GMS repair: {str(e)}")
            return False

        logging.info("GMS repair completed successfully (Template).")
        return True
