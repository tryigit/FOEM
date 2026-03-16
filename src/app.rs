/// Main application window with sidebar navigation.
///
/// UI inspired by iOS and NothingOS: black/white theme,
/// clean typography, rounded cards, minimal accent colors.
use eframe::egui;

use crate::diagnostics::DeviceDiagnostics;
use crate::display_version;
use crate::features::ai_assistant::{
    self, AiAssistantState, AiSettings, Provider, TelemetrySnapshot,
};
use crate::features::{self, Manufacturer};
use crate::license_text::{COMMUNITY_LINKS, CRYPTO_DONATIONS, FIAT_DONATIONS, LICENSE_TEXT};
use crate::theme;
use crate::update_manager::UpdateManager;

#[derive(Default, PartialEq, Clone, Copy)]
enum Panel {
    #[default]
    Device,
    Bootloader,
    Repair,
    Network,
    Flash,
    Diagnostics,
    Tools,
    AiAssistant,
    Updates,
    License,
}

pub struct FOEMApp {
    panel: Panel,
    diagnostics: DeviceDiagnostics,
    update_manager: UpdateManager,
    manufacturer_idx: usize,
    log: String,
    // Input fields
    imei_input: String,
    imei_input_2: String,
    csc_input: String,
    adb_command: String,
    nck_input: String,
    flash_path: String,
    partition_idx: usize,
    package_filter: String,
    remote_path: String,
    local_path: String,
    show_full_license: bool,
    ai_settings: AiSettings,
    ai_state: AiAssistantState,
    ai_attachment_path: String,
}

impl FOEMApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let native_scale = cc.egui_ctx.native_pixels_per_point().unwrap_or({
            if cfg!(target_os = "windows") {
                1.25
            } else {
                1.0
            }
        });
        cc.egui_ctx.set_pixels_per_point(native_scale.max(1.0));
        theme::apply(&cc.egui_ctx);
        Self {
            panel: Panel::Device,
            diagnostics: DeviceDiagnostics::new(),
            update_manager: UpdateManager::new(),
            manufacturer_idx: 0,
            log: String::new(),
            imei_input: String::new(),
            imei_input_2: String::new(),
            csc_input: String::new(),
            adb_command: String::new(),
            nck_input: String::new(),
            flash_path: String::new(),
            partition_idx: 0,
            package_filter: String::new(),
            remote_path: String::new(),
            local_path: String::new(),
            show_full_license: false,
            ai_settings: AiSettings::default(),
            ai_state: AiAssistantState::default(),
            ai_attachment_path: String::new(),
        }
    }

    fn serial(&self) -> Option<&str> {
        self.diagnostics.connected_device()
    }

    fn require_device(&self) -> Result<&str, ()> {
        match self.serial() {
            Some(s) => Ok(s),
            None => Err(()),
        }
    }

    fn manufacturer(&self) -> &Manufacturer {
        &Manufacturer::ALL[self.manufacturer_idx]
    }

    fn panel_name(&self) -> &'static str {
        match self.panel {
            Panel::Device => "Device",
            Panel::Bootloader => "Bootloader",
            Panel::Repair => "Repair",
            Panel::Network => "Network",
            Panel::Flash => "Flash",
            Panel::Diagnostics => "Diagnostics",
            Panel::Tools => "Tools",
            Panel::AiAssistant => "AI Agent",
            Panel::Updates => "Updates",
            Panel::License => "License",
        }
    }

    fn navigate_to_panel(&mut self, target: &str) -> bool {
        let normalized = target.trim().to_ascii_lowercase();
        let next = match normalized.as_str() {
            "device" => Panel::Device,
            "bootloader" => Panel::Bootloader,
            "repair" => Panel::Repair,
            "network" | "network & security" => Panel::Network,
            "flash" => Panel::Flash,
            "diagnostics" => Panel::Diagnostics,
            "tools" => Panel::Tools,
            "ai" | "ai agent" | "ai assistant" => Panel::AiAssistant,
            "updates" => Panel::Updates,
            "license" | "license & support" => Panel::License,
            _ => return false,
        };
        self.panel = next;
        true
    }
}

impl eframe::App for FOEMApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sidebar
        egui::SidePanel::left("sidebar")
            .exact_width(theme::SIDEBAR_WIDTH)
            .frame(
                egui::Frame::none()
                    .fill(theme::SIDEBAR_BG)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("FOEM")
                        .size(24.0)
                        .strong()
                        .color(theme::FG),
                );
                ui.label(
                    egui::RichText::new(format!("v{}", display_version()))
                        .size(11.0)
                        .color(theme::TERTIARY),
                );
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                let items: &[(&str, Panel)] = &[
                    ("Device", Panel::Device),
                    ("Bootloader", Panel::Bootloader),
                    ("Repair", Panel::Repair),
                    ("Network", Panel::Network),
                    ("Flash", Panel::Flash),
                    ("Diagnostics", Panel::Diagnostics),
                    ("Tools", Panel::Tools),
                    ("AI Agent", Panel::AiAssistant),
                    ("Updates", Panel::Updates),
                    ("License & Support", Panel::License),
                ];
                for (label, panel) in items {
                    ui.selectable_value(
                        &mut self.panel,
                        *panel,
                        egui::RichText::new(*label).size(13.0),
                    );
                    ui.add_space(2.0);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Manufacturer selector
                ui.label(
                    egui::RichText::new("Manufacturer")
                        .size(11.0)
                        .color(theme::SECONDARY),
                );
                egui::ComboBox::from_id_salt("mfr")
                    .width(150.0)
                    .selected_text(Manufacturer::ALL[self.manufacturer_idx].name())
                    .show_ui(ui, |ui| {
                        for (i, m) in Manufacturer::ALL.iter().enumerate() {
                            ui.selectable_value(&mut self.manufacturer_idx, i, m.name());
                        }
                    });

                ui.add_space(8.0);
                // Connection status
                if let Some(s) = self.diagnostics.connected_device() {
                    ui.label(
                        egui::RichText::new(format!("Connected: {}", s))
                            .size(10.0)
                            .color(theme::SUCCESS),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No device")
                            .size(10.0)
                            .color(theme::DESTRUCTIVE),
                    );
                }
            });

        // Main content
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::BG).inner_margin(20.0))
            .show(ctx, |ui| match self.panel {
                Panel::Device => self.panel_device(ui),
                Panel::Bootloader => self.panel_bootloader(ui),
                Panel::Repair => self.panel_repair(ui),
                Panel::Network => self.panel_network(ui),
                Panel::Flash => self.panel_flash(ui),
                Panel::Diagnostics => self.panel_diagnostics(ui),
                Panel::Tools => self.panel_tools(ui),
                Panel::AiAssistant => self.panel_ai_assistant(ui),
                Panel::Updates => self.panel_updates(ui),
                Panel::License => self.panel_license(ui),
            });
    }
}

// -- Helper macros / small fns --
fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(title)
            .size(13.0)
            .strong()
            .color(theme::SECONDARY),
    );
    ui.add_space(4.0);
}

fn heading(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(20.0)
            .strong()
            .color(theme::FG),
    );
    ui.add_space(8.0);
}

fn log_area(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(theme::CARD_BG)
        .rounding(theme::ROUNDING)
        .inner_margin(theme::CARD_PADDING)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(380.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 8.0);
                    ui.label(
                        egui::RichText::new(text)
                            .monospace()
                            .size(13.0)
                            .color(theme::FG),
                    );
                });
        });
}

fn btn(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(12.5))
            .min_size(egui::vec2(132.0, 36.0))
            .wrap(),
    )
    .clicked()
}

fn btn_accent(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(12.5)
                .color(egui::Color32::WHITE),
        )
        .fill(theme::ACCENT)
        .min_size(egui::vec2(132.0, 36.0))
        .wrap(),
    )
    .clicked()
}

// ---- Panels ----

impl FOEMApp {
    fn panel_device(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Device");

        let adb_ok = DeviceDiagnostics::is_adb_available();
        let fb_ok = DeviceDiagnostics::is_fastboot_available();
        ui.label(
            egui::RichText::new(format!(
                "ADB: {}   Fastboot: {}",
                if adb_ok { "available" } else { "not found" },
                if fb_ok { "available" } else { "not found" },
            ))
            .size(11.0)
            .color(theme::SECONDARY),
        );

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if btn_accent(ui, "Detect Device") {
                self.log = match self.diagnostics.detect_device() {
                    Ok(Some(s)) => format!("Device detected: {}", s),
                    Ok(None) => "No device detected. Check USB debugging.".into(),
                    Err(e) => format!("Detect device failed: {}", e),
                };
            }
            if btn(ui, "Device Info") {
                if self.require_device().is_ok() {
                    let info = self.diagnostics.get_device_info();
                    self.log.clear();
                    for (k, v) in &info {
                        self.log.push_str(&format!("{}: {}\n", k, v));
                    }
                    if self.log.is_empty() {
                        self.log = "Could not retrieve device info.".into();
                    }
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Build Props") {
                if let Ok(s) = self.require_device() {
                    self.log = features::repair::read_build_props(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
        });

        ui.add_space(8.0);
        log_area(ui, &self.log);
    }

    fn panel_bootloader(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Bootloader");

        let mfr = *self.manufacturer();
        ui.label(
            egui::RichText::new(format!(
                "Selected: {} ({})",
                mfr.name(),
                mfr.platform_hint()
            ))
            .size(11.0)
            .color(theme::SECONDARY),
        );

        section(ui, "Status");
        ui.horizontal_wrapped(|ui| {
            if btn(ui, "Check BL Status") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::check_status(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Check OEM Unlock") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::check_oem_unlock_setting(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Device Variables") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::get_device_vars(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
        });

        section(ui, "Actions");
        ui.horizontal_wrapped(|ui| {
            if btn_accent(ui, "Unlock Bootloader") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::unlock(s, &mfr);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Relock Bootloader") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::relock(s);
                } else {
                    self.log = "Connect a device first.".into();
                }
            }
            if btn(ui, "Bypass Bootloader Unlock (Pre-Feb)") {
                if let Ok(s) = self.require_device() {
                    self.log = features::bootloader::bypass_unlock(s);
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
            }
        });

        section(ui, "Notes");
        ui.label(
            egui::RichText::new(features::bootloader::manufacturer_notes(&mfr))
                .size(11.0)
                .color(theme::SECONDARY),
        );

        ui.add_space(8.0);
        log_area(ui, &self.log);
    }

    fn panel_repair(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Repair");
        let mfr = *self.manufacturer();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // IMEI
            section(ui, "IMEI Management");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Read IMEI") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::read_imei(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Backup IMEI") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::backup_imei(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("IMEI 1:")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.imei_input).desired_width(180.0));
                ui.label(
                    egui::RichText::new("IMEI 2 (optional):")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.imei_input_2).desired_width(180.0));
                if btn(ui, "Write IMEI") {
                    if let Ok(s) = self.require_device() {
                        let imei_payload = if self.imei_input_2.trim().is_empty() {
                            self.imei_input.clone()
                        } else {
                            format!("{},{}", self.imei_input.trim(), self.imei_input_2.trim())
                        };
                        self.log = features::repair::write_imei(s, &imei_payload, &mfr);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // GMS
            section(ui, "Google Mobile Services");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Check GMS") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::check_gms(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn_accent(ui, "Repair GMS") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::repair_gms(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // EFS / NV
            section(ui, "EFS / NV Data");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Backup EFS") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::backup_efs(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Restore EFS") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::restore_efs(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Backup NV") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::backup_nv_data(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Restore NV") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::restore_nv_data(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Samsung-specific
            section(ui, "Samsung Specific");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "DRK Repair") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::repair_drk(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Knox Counter") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::check_knox_counter(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("CSC:")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.csc_input).desired_width(80.0));
                if btn(ui, "Change CSC") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::change_csc(s, &self.csc_input);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Baseband
            section(ui, "Baseband / Modem");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Check Baseband") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::check_baseband(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Repair Baseband") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::repair::repair_baseband(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_network(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Network & Security");

        egui::ScrollArea::vertical().show(ui, |ui| {
            // FRP
            section(ui, "FRP (Factory Reset Protection)");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Check FRP") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::check_frp_status(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn_accent(ui, "Bypass FRP (ADB)") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::bypass_frp(
                            s,
                            &features::network::FrpMethod::AdbBypass,
                        );
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Skip Setup") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::bypass_frp(
                            s,
                            &features::network::FrpMethod::SetupWizardSkip,
                        );
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Remove Accounts") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::bypass_frp(
                            s,
                            &features::network::FrpMethod::AccountManagerRemove,
                        );
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Content Provider") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::bypass_frp(
                            s,
                            &features::network::FrpMethod::ContentProviderReset,
                        );
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Remove Google Acc") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::remove_google_account(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Carrier
            section(ui, "Carrier / SIM Unlock");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Check Lock Status") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::check_carrier_lock(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("NCK:")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.nck_input).desired_width(160.0));
                if btn(ui, "Unlock Carrier") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::unlock_carrier(s, &self.nck_input);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // MDM
            section(ui, "MDM / Enterprise");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Check MDM") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::check_mdm_status(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn_accent(ui, "Remove MDM") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::remove_mdm(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Knox Bypass") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::network::bypass_knox(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_flash(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Flash");
        let mfr = *self.manufacturer();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // EDL
            section(ui, "EDL Mode (Qualcomm 9008)");
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Enter EDL Mode") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::enter_edl_mode(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Flash via EDL") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::flash_edl(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Fastboot
            section(ui, "Fastboot Flash");
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Partition:")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                egui::ComboBox::from_id_salt("part")
                    .width(120.0)
                    .selected_text(features::flash::FASTBOOT_PARTITIONS[self.partition_idx])
                    .show_ui(ui, |ui| {
                        for (i, p) in features::flash::FASTBOOT_PARTITIONS.iter().enumerate() {
                            ui.selectable_value(&mut self.partition_idx, i, *p);
                        }
                    });
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Image:")
                        .size(12.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.flash_path).desired_width(300.0));
            });
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Flash Partition") {
                    if let Ok(s) = self.require_device() {
                        let part = features::flash::FASTBOOT_PARTITIONS[self.partition_idx];
                        self.log = features::flash::flash_partition(s, part, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Erase Partition") {
                    if let Ok(s) = self.require_device() {
                        let part = features::flash::FASTBOOT_PARTITIONS[self.partition_idx];
                        self.log = features::flash::erase_partition(s, part);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Flash vbmeta (no verify)") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::flash_vbmeta_disabled(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Root
            section(ui, "Root (Magisk / KernelSU)");
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Install Magisk") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::install_magisk(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn_accent(ui, "Install KernelSU") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::install_kernelsu(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Recovery
            section(ui, "Recovery");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Flash Recovery") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::flash_recovery(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Temp Boot Recovery") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::boot_recovery_temp(s, &self.flash_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Firmware
            section(ui, "Firmware");
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Flash Firmware") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::flash_firmware(s, &self.flash_path, &mfr);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // MediaTek
            section(ui, "MediaTek SP Flash");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Enter BROM Mode") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::flash::enter_brom_mode(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "SP Flash Info") {
                    self.log = features::flash::sp_flash_info();
                }
            });

            // Reboot modes
            section(ui, "Reboot");
            ui.horizontal_wrapped(|ui| {
                let modes = [
                    "system",
                    "recovery",
                    "bootloader",
                    "edl",
                    "download",
                    "sideload",
                ];
                for mode in &modes {
                    if btn(ui, mode) {
                        if let Ok(s) = self.require_device() {
                            self.log = features::flash::reboot_to(s, mode);
                        } else {
                            self.log = "Connect a device first.".into();
                        }
                    }
                }
            });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_diagnostics(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Hardware Diagnostics");

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Run All Tests") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::run_all(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            section(ui, "Individual Tests");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Battery") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::check_battery(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Display") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_display(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Sensors") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_sensors(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Audio") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_audio(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Cameras") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_cameras(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Connectivity") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_connectivity(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Biometrics") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_biometrics(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Storage") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_storage(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "USB") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_usb(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Telephony") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::test_telephony(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Battery Stats") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::hardware_test::battery_stats(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_tools(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Tools");

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ADB Shell
            section(ui, "ADB Shell");
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.adb_command)
                        .desired_width(400.0)
                        .hint_text("Enter ADB shell command..."),
                );
                if btn_accent(ui, "Execute") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::execute_shell(s, &self.adb_command);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Logcat
            section(ui, "Logcat");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Logcat (100 lines)") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::capture_logcat(s, 100);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Clear Logcat") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::clear_logcat(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // File Manager
            section(ui, "File Manager");
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Remote:")
                        .size(11.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.remote_path).desired_width(200.0));
                ui.label(
                    egui::RichText::new("Local:")
                        .size(11.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.local_path).desired_width(200.0));
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Pull File") {
                    if let Ok(s) = self.require_device() {
                        self.log =
                            features::tools::pull_file(s, &self.remote_path, &self.local_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Push File") {
                    if let Ok(s) = self.require_device() {
                        self.log =
                            features::tools::push_file(s, &self.local_path, &self.remote_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "List Files") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::list_files(s, &self.remote_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // APK & Packages
            section(ui, "APK & Packages");
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Filter:")
                        .size(11.0)
                        .color(theme::SECONDARY),
                );
                ui.add(egui::TextEdit::singleline(&mut self.package_filter).desired_width(180.0));
                if btn(ui, "Install APK") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::install_apk(s, &self.local_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "List All") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::list_packages(s, &self.package_filter);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "User Apps") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::list_user_packages(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "System Apps") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::list_system_packages(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Disable Package") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::disable_package(s, &self.package_filter);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Enable Package") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::enable_package(s, &self.package_filter);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // Backup & Restore
            section(ui, "Backup & Restore");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Full Backup") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::full_backup(s, &self.local_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Full Restore") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::full_restore(s, &self.local_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Screenshot") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::take_screenshot(s, &self.local_path);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Screen Mirror (scrcpy)") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::start_scrcpy(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            // System
            section(ui, "System");
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Reboot") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::reboot(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Recovery") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::reboot_recovery(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Bootloader") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::reboot_bootloader(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Dev Options") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::enable_developer_options(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                if btn(ui, "Uptime") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::get_uptime(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Memory") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::get_memory_info(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "CPU") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::get_cpu_info(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
                if btn(ui, "Processes") {
                    if let Ok(s) = self.require_device() {
                        self.log = features::tools::get_processes(s);
                    } else {
                        self.log = "Connect a device first.".into();
                    }
                }
            });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_ai_assistant(&mut self, ui: &mut egui::Ui) {
        heading(ui, "AI Agent");

        egui::ScrollArea::vertical().show(ui, |ui| {
            section(ui, "Provider");
            egui::ComboBox::from_id_salt("ai_provider")
                .width(180.0)
                .selected_text(self.ai_settings.provider.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.ai_settings.provider,
                        Provider::OpenRouter,
                        Provider::OpenRouter.label(),
                    );
                    ui.selectable_value(
                        &mut self.ai_settings.provider,
                        Provider::OpenAI,
                        Provider::OpenAI.label(),
                    );
                    ui.selectable_value(
                        &mut self.ai_settings.provider,
                        Provider::Gemini,
                        Provider::Gemini.label(),
                    );
                    ui.selectable_value(
                        &mut self.ai_settings.provider,
                        Provider::Local,
                        Provider::Local.label(),
                    );
                });
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Model").size(11.0).color(theme::SECONDARY));
            ui.add(egui::TextEdit::singleline(&mut self.ai_settings.model).desired_width(320.0));
            ui.label(
                egui::RichText::new("API Key (for Local provider, leave empty if not needed)")
                    .size(11.0)
                    .color(theme::SECONDARY),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.ai_settings.api_key)
                    .password(true)
                    .desired_width(420.0),
            );
            if self.ai_settings.provider == Provider::Local {
                ui.label(egui::RichText::new("Local Endpoint").size(11.0).color(theme::SECONDARY));
                ui.add(
                    egui::TextEdit::singleline(&mut self.ai_settings.custom_endpoint)
                        .desired_width(420.0),
                );
            }
            section(ui, "Advanced");
            ui.checkbox(&mut self.ai_settings.vision_enabled, "Enable vision/image analysis");
            ui.add(
                egui::Slider::new(&mut self.ai_settings.temperature, 0.0..=1.0)
                    .text("Temperature")
                    .step_by(0.05),
            );
            ui.label(
                egui::RichText::new("Detected commands are listed below for manual copy/execute.")
                    .size(11.0)
                    .color(theme::TERTIARY),
            );

            section(ui, "Attachment");
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.ai_attachment_path).desired_width(420.0),
                );
                if btn(ui, "Attach File") {
                    match self
                        .ai_state
                        .add_attachment_from_path(self.ai_attachment_path.trim())
                    {
                        Ok(()) => {
                            self.log = format!(
                                "Attachment added. Total attachments: {}",
                                self.ai_state.attachments.len()
                            );
                            self.ai_attachment_path.clear();
                        }
                        Err(e) => {
                            self.log = e;
                        }
                    }
                }
            });

            section(ui, "Chat");
            ui.add(
                egui::TextEdit::multiline(&mut self.ai_state.input)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .hint_text("Ask the AI agent for repair steps, diagnostics help, or command suggestions..."),
            );
            ui.horizontal_wrapped(|ui| {
                if btn_accent(ui, "Send to AI") {
                    if self.ai_state.input.trim().is_empty() {
                        self.log = "Please enter a prompt first.".into();
                    } else {
                        let telemetry = TelemetrySnapshot {
                            active_panel: self.panel_name().to_string(),
                            recent_actions: vec![self.log.clone()],
                            device_summary: self
                                .serial()
                                .map(|s| format!("Connected device: {}", s))
                                .unwrap_or_else(|| "No device connected".to_string()),
                        };
                        let user_input = self.ai_state.input.clone();
                        self.ai_state.push_user_message(user_input);
                        match ai_assistant::send_chat(
                            &mut self.ai_state,
                            &self.ai_settings,
                            telemetry,
                        ) {
                            Ok(resp) => {
                                if let Some(target) = self.ai_state.last_action_target.clone() {
                                    if self.navigate_to_panel(&target) {
                                        self.log = format!(
                                            "AI Response:\n{}\n\nAI navigated to panel: {}",
                                            resp, target
                                        );
                                    } else {
                                        self.log = format!(
                                            "AI Response:\n{}\n\nAI requested unknown panel: {}",
                                            resp, target
                                        );
                                    }
                                } else {
                                    self.log = format!("AI Response:\n{}", resp);
                                }
                                self.ai_state.last_error = None;
                            }
                            Err(e) => {
                                self.log = format!("AI request failed: {}", e);
                                self.ai_state.last_error = Some(e);
                            }
                        }
                    }
                }
                if btn(ui, "Clear Conversation") {
                    self.ai_state = AiAssistantState::default();
                    self.log = "AI conversation cleared.".into();
                }
                if btn(ui, "Clear Detected Commands") {
                    self.ai_state.pending_commands.clear();
                    self.log = "Detected command list cleared.".into();
                }
            });

            if !self.ai_state.pending_commands.is_empty() {
                section(ui, "Detected Commands");
                for cmd in &self.ai_state.pending_commands {
                    ui.label(egui::RichText::new(cmd).monospace().size(11.0).color(theme::FG));
                }
            }

            section(ui, "Conversation");
            egui::Frame::none()
                .fill(theme::CARD_BG)
                .rounding(theme::ROUNDING)
                .inner_margin(theme::CARD_PADDING)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .id_salt("ai_history_scroll")
                        .show(ui, |ui| {
                            for msg in &self.ai_state.history {
                                let role = match msg.role {
                                    ai_assistant::ChatRole::System => "System",
                                    ai_assistant::ChatRole::User => "You",
                                    ai_assistant::ChatRole::Assistant => "AI",
                                };
                                ui.label(
                                    egui::RichText::new(format!("{}: {}", role, msg.content))
                                        .size(11.0)
                                        .color(theme::SECONDARY),
                                );
                                ui.add_space(4.0);
                            }
                        });
                });

            ui.add_space(8.0);
            log_area(ui, &self.log);
        });
    }

    fn panel_updates(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Updates");

        ui.horizontal_wrapped(|ui| {
            if btn_accent(ui, "Check for Updates") {
                match self.update_manager.check_for_updates() {
                    Ok(Some(info)) => {
                        self.log = format!(
                            "New version available: {}\nCurrent: {}\nDownload: {}",
                            info.latest_version,
                            display_version(),
                            info.download_url,
                        );
                    }
                    Ok(None) => {
                        self.log = format!(
                            "Current version: {}\nYou are running the latest version.",
                            display_version()
                        );
                    }
                    Err(e) => {
                        self.log = format!("Update check failed: {}", e);
                    }
                }
            }
            if btn(ui, "Open Releases Page") {
                let _ = open::that("https://github.com/tryigit/FOEM/releases");
            }
        });

        ui.add_space(8.0);
        log_area(ui, &self.log);
    }

    fn panel_license(&mut self, ui: &mut egui::Ui) {
        heading(ui, "License & Support");

        egui::ScrollArea::vertical().show(ui, |ui| {
            // About
            section(ui, "About");
            ui.label(
                egui::RichText::new("FOEM -- Free Open Ecosystem for Mobile Devices")
                    .size(13.0)
                    .color(theme::FG),
            );
            ui.label(
                egui::RichText::new(format!("Version {}", display_version()))
                    .size(11.0)
                    .color(theme::SECONDARY),
            );
            ui.label(
                egui::RichText::new("Design inspired by iOS and NothingOS")
                    .size(11.0)
                    .color(theme::TERTIARY),
            );
            ui.label(
                egui::RichText::new("Non-Commercial Software License. See full text below.")
                    .size(11.0)
                    .color(theme::SECONDARY),
            );

            // License
            section(ui, "License");
            if btn(
                ui,
                if self.show_full_license {
                    "Hide License"
                } else {
                    "Show Full License"
                },
            ) {
                self.show_full_license = !self.show_full_license;
            }
            if self.show_full_license {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(theme::CARD_BG)
                    .rounding(theme::ROUNDING)
                    .inner_margin(theme::CARD_PADDING)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .id_salt("license_scroll")
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(LICENSE_TEXT)
                                        .monospace()
                                        .size(10.5)
                                        .color(theme::SECONDARY),
                                );
                            });
                    });
            }

            // Donate
            section(ui, "Support the Development");
            ui.label(
                egui::RichText::new(
                    "If you find this project helpful, consider supporting the development.",
                )
                .size(12.0)
                .color(theme::SECONDARY),
            );
            ui.add_space(4.0);

            egui::Frame::none()
                .fill(theme::CARD_BG)
                .rounding(theme::ROUNDING)
                .inner_margin(theme::CARD_PADDING)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Crypto Addresses")
                            .size(12.0)
                            .strong()
                            .color(theme::FG),
                    );
                    ui.add_space(4.0);
                    for &(text, size, monospace, space) in CRYPTO_DONATIONS {
                        let mut t = egui::RichText::new(text).size(size).color(theme::SECONDARY);
                        if monospace {
                            t = t.monospace();
                        }
                        ui.label(t);
                        if space > 0.0 {
                            ui.add_space(space);
                        }
                    }
                });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for &(text, link) in FIAT_DONATIONS {
                    if btn(ui, text) {
                        let _ = open::that(link);
                    }
                }
            });

            // Links
            section(ui, "Links");
            ui.horizontal_wrapped(|ui| {
                for &(text, link) in COMMUNITY_LINKS {
                    if btn(ui, text) {
                        let _ = open::that(link);
                    }
                }
            });
        });
    }
}
