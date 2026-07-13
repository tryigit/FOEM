/// Black and white theme inspired by iOS and NothingOS.
///
/// Pure black background, white text, dark gray cards,
/// minimal accent colors for interactive elements only.
use eframe::egui;

// -- Background --
pub const BG: egui::Color32 = egui::Color32::from_rgb(0, 0, 0);
pub const CARD_BG: egui::Color32 = egui::Color32::from_rgb(28, 28, 30);
pub const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(18, 18, 20);
pub const SEPARATOR: egui::Color32 = egui::Color32::from_rgb(56, 56, 58);

// -- Text --
pub const FG: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(142, 142, 147);
pub const TERTIARY: egui::Color32 = egui::Color32::from_rgb(99, 99, 102);

// -- Accent --
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(10, 132, 255);
pub const DESTRUCTIVE: egui::Color32 = egui::Color32::from_rgb(255, 69, 58);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(48, 209, 88);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 214, 10);

// -- Dimensions --
pub const SIDEBAR_WIDTH: f32 = 180.0;
pub const ROUNDING: f32 = 10.0;
pub const CARD_PADDING: f32 = 14.0;

/// Apply the FOEM dark theme to an egui context.
pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = CARD_BG;
    style.visuals.override_text_color = Some(FG);
    style.visuals.widgets.noninteractive.bg_fill = CARD_BG;
    style.visuals.widgets.inactive.bg_fill = CARD_BG;
    style.visuals.widgets.hovered.bg_fill = SEPARATOR;
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.selection.bg_fill = ACCENT;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(10, 10, 10);
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    ctx.set_style(style);
}
