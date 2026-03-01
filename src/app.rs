/// Main application window and UI layout.
///
/// Design inspired by iOS, NothingOS, and similar modern design systems.
/// Dark theme with clean typography, rounded cards, and accent colors.
use eframe::egui;

use crate::diagnostics::DeviceDiagnostics;
use crate::gms_repair::GMSRepairManager;
use crate::update_manager::UpdateManager;
use crate::VERSION;

// -- Theme constants --
const BG: egui::Color32 = egui::Color32::from_rgb(13, 13, 13);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(26, 26, 26);
const FG: egui::Color32 = egui::Color32::from_rgb(224, 224, 224);
const MUTED: egui::Color32 = egui::Color32::from_rgb(136, 136, 136);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(58, 123, 213);

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    Device,
    GmsRepair,
    Updates,
    About,
}

pub struct FOEMApp {
    tab: Tab,
    diagnostics: DeviceDiagnostics,
    gms_manager: Option<GMSRepairManager>,
    update_manager: UpdateManager,
    device_log: String,
    gms_log: String,
    update_log: String,
}

impl FOEMApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = CARD_BG;
        style.visuals.override_text_color = Some(FG);
        cc.egui_ctx.set_style(style);

        Self {
            tab: Tab::Device,
            diagnostics: DeviceDiagnostics::new(),
            gms_manager: None,
            update_manager: UpdateManager::new(),
            device_log: String::new(),
            gms_log: String::new(),
            update_log: String::new(),
        }
    }
}

impl eframe::App for FOEMApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("FOEM")
                        .size(26.0)
                        .strong()
                        .color(FG),
                );
                ui.label(
                    egui::RichText::new(format!("v{}", VERSION))
                        .size(12.0)
                        .color(MUTED),
                );
                ui.add_space(24.0);
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Device,
                    egui::RichText::new("Device").size(14.0),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::GmsRepair,
                    egui::RichText::new("GMS Repair").size(14.0),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::Updates,
                    egui::RichText::new("Updates").size(14.0),
                );
                ui.selectable_value(
                    &mut self.tab,
                    Tab::About,
                    egui::RichText::new("About").size(14.0),
                );
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Device => self.device_tab(ui),
                Tab::GmsRepair => self.gms_tab(ui),
                Tab::Updates => self.update_tab(ui),
                Tab::About => Self::about_tab(ui),
            }
        });
    }
}

impl FOEMApp {
    fn device_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Device Diagnostics")
                .size(20.0)
                .strong()
                .color(FG),
        );
        ui.add_space(4.0);
        let adb_ok = DeviceDiagnostics::is_adb_available();
        let fb_ok = DeviceDiagnostics::is_fastboot_available();
        ui.label(
            egui::RichText::new(format!(
                "ADB: {}  |  Fastboot: {}",
                if adb_ok { "available" } else { "not found" },
                if fb_ok { "available" } else { "not found" },
            ))
            .size(11.0)
            .color(MUTED),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui
                .button(egui::RichText::new("Detect Device").size(13.0).color(egui::Color32::WHITE))
                .clicked()
            {
                self.device_log.clear();
                match self.diagnostics.detect_device() {
                    Some(serial) => {
                        self.device_log
                            .push_str(&format!("Device detected: {}\n", serial));
                        self.gms_manager = Some(GMSRepairManager::new(serial));
                    }
                    None => {
                        self.device_log
                            .push_str("No device detected.\nMake sure USB debugging is enabled and ADB is installed.\n");
                    }
                }
            }

            if ui
                .button(egui::RichText::new("Health Check").size(13.0))
                .clicked()
            {
                if self.diagnostics.connected_device().is_some() {
                    let info = self.diagnostics.get_device_info();
                    self.device_log.clear();
                    if info.is_empty() {
                        self.device_log
                            .push_str("Could not retrieve device info.\n");
                    } else {
                        for (key, value) in &info {
                            self.device_log
                                .push_str(&format!("{}: {}\n", key, value));
                        }
                    }
                } else {
                    self.device_log = "Please detect a device first.\n".into();
                }
            }
        });

        ui.add_space(8.0);
        egui::Frame::none()
            .fill(CARD_BG)
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.device_log)
                                .monospace()
                                .size(12.0)
                                .color(FG),
                        );
                    });
            });
    }

    fn gms_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("GMS Repair")
                .size(20.0)
                .strong()
                .color(FG),
        );
        ui.add_space(12.0);

        let has_device = self.gms_manager.is_some();

        ui.horizontal(|ui| {
            let check_btn =
                ui.button(egui::RichText::new("Check GMS").size(13.0).color(egui::Color32::WHITE));
            if check_btn.clicked() {
                if let Some(manager) = &self.gms_manager {
                    self.gms_log.clear();
                    let status = manager.check_gms_status();
                    for (pkg, state) in &status {
                        self.gms_log
                            .push_str(&format!("{}: {}\n", pkg, state));
                    }
                } else {
                    self.gms_log = "Please detect a device first.\n".into();
                }
            }

            let repair_btn = ui.button(egui::RichText::new("Repair GMS").size(13.0));
            if repair_btn.clicked() {
                if let Some(manager) = &self.gms_manager {
                    self.gms_log.clear();
                    let status = manager.repair_gms();
                    for (pkg, state) in &status {
                        self.gms_log
                            .push_str(&format!("{}: {}\n", pkg, state));
                    }
                    self.gms_log.push_str("\nRepair sequence completed.\n");
                } else {
                    self.gms_log = "Please detect a device first.\n".into();
                }
            }
        });

        if !has_device {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Connect a device in the Device tab first.")
                    .size(12.0)
                    .color(MUTED),
            );
        }

        ui.add_space(8.0);
        egui::Frame::none()
            .fill(CARD_BG)
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.gms_log)
                                .monospace()
                                .size(12.0)
                                .color(FG),
                        );
                    });
            });
    }

    fn update_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Update Checker")
                .size(20.0)
                .strong()
                .color(FG),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new("Check for Updates")
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                )
                .clicked()
            {
                self.update_log.clear();
                match self.update_manager.check_for_updates() {
                    Ok(Some(info)) => {
                        self.update_log.push_str(&format!(
                            "New version available: {}\nCurrent version: {}\nDownload: {}\n",
                            info.latest_version, VERSION, info.download_url,
                        ));
                    }
                    Ok(None) => {
                        self.update_log.push_str(&format!(
                            "Current version: {}\nYou are running the latest version.\n",
                            VERSION,
                        ));
                    }
                    Err(e) => {
                        self.update_log
                            .push_str(&format!("Could not check for updates: {}\n", e));
                    }
                }
            }

            if ui
                .button(egui::RichText::new("Open Releases Page").size(13.0))
                .clicked()
            {
                let _ = open::that("https://github.com/tryigit/FOEM/releases");
            }
        });

        ui.add_space(8.0);
        egui::Frame::none()
            .fill(CARD_BG)
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&self.update_log)
                                .monospace()
                                .size(12.0)
                                .color(FG),
                        );
                    });
            });
    }

    fn about_tab(ui: &mut egui::Ui) {
        ui.add_space(32.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("FOEM")
                    .size(32.0)
                    .strong()
                    .color(FG),
            );
            ui.label(
                egui::RichText::new(format!("Version {}", VERSION))
                    .size(13.0)
                    .color(MUTED),
            );
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("Free Open Ecosystem for Mobile Devices")
                    .size(15.0)
                    .color(FG),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Design inspired by iOS and NothingOS")
                    .size(12.0)
                    .color(MUTED),
            );
            ui.add_space(24.0);

            if ui
                .button(egui::RichText::new("GitHub Repository").size(13.0).color(ACCENT))
                .clicked()
            {
                let _ = open::that("https://github.com/tryigit/FOEM");
            }

            ui.add_space(8.0);

            if ui
                .button(egui::RichText::new("Report Issue").size(13.0))
                .clicked()
            {
                let _ = open::that("https://github.com/tryigit/FOEM/issues");
            }

            ui.add_space(24.0);
            ui.label(
                egui::RichText::new(
                    "If you find this project helpful, consider supporting the development.\n\
                     Visit the GitHub repository for donation options.",
                )
                .size(12.0)
                .color(MUTED),
            );
        });
    }
}
