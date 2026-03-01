"""Main application window.

The GUI design is inspired by iOS, NothingOS, and similar modern design systems.
This module defines the main window layout and navigation structure.
"""

import tkinter as tk
from tkinter import ttk, messagebox
import logging

from src import __version__
from src.diagnostics import DeviceDiagnostics
from src.gms_repair import GMSRepairManager
from src.update_manager import UpdateManager

logger = logging.getLogger(__name__)

WINDOW_TITLE = "FOEM"
WINDOW_WIDTH = 900
WINDOW_HEIGHT = 620
BG_COLOR = "#0d0d0d"
FG_COLOR = "#e0e0e0"
ACCENT_COLOR = "#3a7bd5"
CARD_BG = "#1a1a1a"


class FOEMApp:
    """Main FOEM application window."""

    def __init__(self):
        self.root = tk.Tk()
        self.root.title(f"{WINDOW_TITLE} v{__version__}")
        self.root.geometry(f"{WINDOW_WIDTH}x{WINDOW_HEIGHT}")
        self.root.configure(bg=BG_COLOR)
        self.root.resizable(True, True)

        self.diagnostics = DeviceDiagnostics()
        self.update_manager = UpdateManager()

        self._build_ui()

    def _build_ui(self):
        """Build the main UI layout."""
        # Header
        header = tk.Frame(self.root, bg=BG_COLOR)
        header.pack(fill=tk.X, padx=20, pady=(20, 10))

        tk.Label(
            header,
            text="FOEM",
            font=("Helvetica", 28, "bold"),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(side=tk.LEFT)

        tk.Label(
            header,
            text=f"v{__version__}",
            font=("Helvetica", 12),
            bg=BG_COLOR,
            fg="#888888",
        ).pack(side=tk.LEFT, padx=(8, 0), pady=(10, 0))

        # Navigation tabs
        notebook = ttk.Notebook(self.root)
        notebook.pack(fill=tk.BOTH, expand=True, padx=20, pady=10)

        self._build_device_tab(notebook)
        self._build_gms_tab(notebook)
        self._build_update_tab(notebook)
        self._build_about_tab(notebook)

        # Status bar
        self.status_var = tk.StringVar(value="Ready")
        status_bar = tk.Label(
            self.root,
            textvariable=self.status_var,
            font=("Helvetica", 10),
            bg=CARD_BG,
            fg="#888888",
            anchor=tk.W,
            padx=10,
        )
        status_bar.pack(fill=tk.X, side=tk.BOTTOM)

    def _build_device_tab(self, notebook):
        """Build the Device Diagnostics tab."""
        frame = tk.Frame(notebook, bg=BG_COLOR)
        notebook.add(frame, text="  Device  ")

        tk.Label(
            frame,
            text="Device Diagnostics",
            font=("Helvetica", 18, "bold"),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(pady=(20, 10))

        self.device_info_text = tk.Text(
            frame, height=12, bg=CARD_BG, fg=FG_COLOR,
            font=("Courier", 11), insertbackground=FG_COLOR,
            relief=tk.FLAT, padx=10, pady=10,
        )
        self.device_info_text.pack(fill=tk.BOTH, expand=True, padx=20, pady=10)

        btn_frame = tk.Frame(frame, bg=BG_COLOR)
        btn_frame.pack(pady=10)

        tk.Button(
            btn_frame, text="Detect Device", command=self._on_detect_device,
            bg=ACCENT_COLOR, fg="white", font=("Helvetica", 11, "bold"),
            relief=tk.FLAT, padx=20, pady=8,
        ).pack(side=tk.LEFT, padx=5)

        tk.Button(
            btn_frame, text="Health Check", command=self._on_health_check,
            bg=CARD_BG, fg=FG_COLOR, font=("Helvetica", 11),
            relief=tk.FLAT, padx=20, pady=8,
        ).pack(side=tk.LEFT, padx=5)

    def _build_gms_tab(self, notebook):
        """Build the GMS Repair tab."""
        frame = tk.Frame(notebook, bg=BG_COLOR)
        notebook.add(frame, text="  GMS Repair  ")

        tk.Label(
            frame,
            text="GMS Repair",
            font=("Helvetica", 18, "bold"),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(pady=(20, 10))

        self.gms_info_text = tk.Text(
            frame, height=12, bg=CARD_BG, fg=FG_COLOR,
            font=("Courier", 11), insertbackground=FG_COLOR,
            relief=tk.FLAT, padx=10, pady=10,
        )
        self.gms_info_text.pack(fill=tk.BOTH, expand=True, padx=20, pady=10)

        btn_frame = tk.Frame(frame, bg=BG_COLOR)
        btn_frame.pack(pady=10)

        tk.Button(
            btn_frame, text="Check GMS", command=self._on_check_gms,
            bg=ACCENT_COLOR, fg="white", font=("Helvetica", 11, "bold"),
            relief=tk.FLAT, padx=20, pady=8,
        ).pack(side=tk.LEFT, padx=5)

        tk.Button(
            btn_frame, text="Repair GMS", command=self._on_repair_gms,
            bg=CARD_BG, fg=FG_COLOR, font=("Helvetica", 11),
            relief=tk.FLAT, padx=20, pady=8,
        ).pack(side=tk.LEFT, padx=5)

    def _build_update_tab(self, notebook):
        """Build the Updates tab."""
        frame = tk.Frame(notebook, bg=BG_COLOR)
        notebook.add(frame, text="  Updates  ")

        tk.Label(
            frame,
            text="Update Checker",
            font=("Helvetica", 18, "bold"),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(pady=(20, 10))

        self.update_info_text = tk.Text(
            frame, height=8, bg=CARD_BG, fg=FG_COLOR,
            font=("Courier", 11), insertbackground=FG_COLOR,
            relief=tk.FLAT, padx=10, pady=10,
        )
        self.update_info_text.pack(fill=tk.BOTH, expand=True, padx=20, pady=10)

        tk.Button(
            frame, text="Check for Updates", command=self._on_check_updates,
            bg=ACCENT_COLOR, fg="white", font=("Helvetica", 11, "bold"),
            relief=tk.FLAT, padx=20, pady=8,
        ).pack(pady=10)

    def _build_about_tab(self, notebook):
        """Build the About tab."""
        frame = tk.Frame(notebook, bg=BG_COLOR)
        notebook.add(frame, text="  About  ")

        tk.Label(
            frame,
            text="FOEM",
            font=("Helvetica", 24, "bold"),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(pady=(30, 5))

        tk.Label(
            frame,
            text=f"Version {__version__}",
            font=("Helvetica", 12),
            bg=BG_COLOR,
            fg="#888888",
        ).pack()

        tk.Label(
            frame,
            text="Free Open Ecosystem for Mobile Devices",
            font=("Helvetica", 13),
            bg=BG_COLOR,
            fg=FG_COLOR,
        ).pack(pady=(10, 20))

        tk.Label(
            frame,
            text="Design inspired by iOS and NothingOS",
            font=("Helvetica", 11),
            bg=BG_COLOR,
            fg="#888888",
        ).pack()

        tk.Label(
            frame,
            text="https://github.com/tryigit/FOEM",
            font=("Helvetica", 11, "underline"),
            bg=BG_COLOR,
            fg=ACCENT_COLOR,
            cursor="hand2",
        ).pack(pady=(20, 5))

        tk.Label(
            frame,
            text=(
                "If you find this project helpful, consider supporting the development.\n"
                "Visit the GitHub repository for donation options."
            ),
            font=("Helvetica", 11),
            bg=BG_COLOR,
            fg="#888888",
            justify=tk.CENTER,
        ).pack(pady=(20, 0))

    # -- Event handlers --

    def _on_detect_device(self):
        self.status_var.set("Detecting device...")
        self.device_info_text.delete("1.0", tk.END)

        device = self.diagnostics.detect_device()
        if device:
            self.device_info_text.insert(tk.END, f"Connected device: {device}\n")
            self.status_var.set(f"Device detected: {device}")
        else:
            self.device_info_text.insert(
                tk.END, "No device detected.\nMake sure USB debugging is enabled.\n"
            )
            self.status_var.set("No device detected")

    def _on_health_check(self):
        if not self.diagnostics.connected_device:
            messagebox.showwarning("FOEM", "Please detect a device first.")
            return

        self.status_var.set("Running health check...")
        info = self.diagnostics.get_device_info()

        self.device_info_text.delete("1.0", tk.END)
        if info:
            for key, value in info.items():
                self.device_info_text.insert(tk.END, f"{key}: {value}\n")
            self.status_var.set("Health check complete")
        else:
            self.device_info_text.insert(tk.END, "Could not retrieve device info.\n")
            self.status_var.set("Health check failed")

    def _on_check_gms(self):
        if not self.diagnostics.connected_device:
            messagebox.showwarning("FOEM", "Please detect a device first.")
            return

        self.status_var.set("Checking GMS status...")
        manager = GMSRepairManager(self.diagnostics.connected_device)
        status = manager.check_gms_status()

        self.gms_info_text.delete("1.0", tk.END)
        for package, state in status.items():
            self.gms_info_text.insert(tk.END, f"{package}: {state}\n")
        self.status_var.set("GMS check complete")

    def _on_repair_gms(self):
        if not self.diagnostics.connected_device:
            messagebox.showwarning("FOEM", "Please detect a device first.")
            return

        self.status_var.set("Repairing GMS...")
        manager = GMSRepairManager(self.diagnostics.connected_device)
        status = manager.repair_gms()

        self.gms_info_text.delete("1.0", tk.END)
        for package, state in status.items():
            self.gms_info_text.insert(tk.END, f"{package}: {state}\n")
        self.gms_info_text.insert(tk.END, "\nRepair sequence completed.\n")
        self.status_var.set("GMS repair complete")

    def _on_check_updates(self):
        self.status_var.set("Checking for updates...")
        self.update_info_text.delete("1.0", tk.END)

        has_update = self.update_manager.check_for_updates()
        if has_update:
            self.update_info_text.insert(
                tk.END,
                f"New version available: {self.update_manager.latest_version}\n"
                f"Current version: {self.update_manager.current_version}\n\n"
                f"Download: {self.update_manager.download_url}\n",
            )
            self.status_var.set("Update available")
        else:
            self.update_info_text.insert(
                tk.END,
                f"Current version: {self.update_manager.current_version}\n"
                "You are running the latest version.\n",
            )
            self.status_var.set("Up to date")

    def run(self):
        """Start the application main loop."""
        self.root.mainloop()
