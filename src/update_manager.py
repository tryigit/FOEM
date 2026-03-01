"""Application update checking against GitHub releases."""

import logging
import json
import urllib.request
import urllib.error

from src import __version__

logger = logging.getLogger(__name__)

GITHUB_API_URL = "https://api.github.com/repos/tryigit/FOEM/releases/latest"
RELEASES_URL = "https://github.com/tryigit/FOEM/releases"


class UpdateManager:
    """Check for application updates from the GitHub repository."""

    def __init__(self):
        self.current_version = __version__
        self.latest_version = None
        self.download_url = None

    def check_for_updates(self):
        """Query the GitHub API for the latest release.

        Returns True if a newer version is available, False otherwise.
        """
        logger.info("Checking for updates...")
        logger.info("Current version: %s", self.current_version)

        try:
            req = urllib.request.Request(
                GITHUB_API_URL,
                headers={"Accept": "application/vnd.github.v3+json"},
            )
            with urllib.request.urlopen(req, timeout=10) as response:
                data = json.loads(response.read().decode())
        except (urllib.error.URLError, json.JSONDecodeError) as exc:
            logger.warning("Could not check for updates: %s", exc)
            return False

        tag = data.get("tag_name", "")
        self.latest_version = tag.lstrip("v")
        self.download_url = data.get("html_url", RELEASES_URL)

        if self.latest_version and self.latest_version != self.current_version:
            logger.info(
                "New version available: %s (current: %s)",
                self.latest_version,
                self.current_version,
            )
            logger.info("Download: %s", self.download_url)
            return True

        logger.info("You are running the latest version.")
        return False
