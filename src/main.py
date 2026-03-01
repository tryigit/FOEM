"""Application entry point for FOEM."""

import sys
import logging

logging.basicConfig(
    level=logging.INFO,
    format="%(levelname)s: %(message)s",
)

logger = logging.getLogger(__name__)


def main():
    """Launch the FOEM application."""
    logger.info("Starting FOEM...")

    try:
        from src.ui.app import FOEMApp
    except ImportError:
        logger.error(
            "Could not import GUI module. "
            "Make sure tkinter is installed on your system."
        )
        sys.exit(1)

    app = FOEMApp()
    app.run()


if __name__ == "__main__":
    main()
