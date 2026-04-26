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
    let scale = ctx.pixels_per_point().max(1.0);
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
    style.visuals.window_rounding = egui::Rounding::same(ROUNDING);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(ROUNDING / 2.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(ROUNDING / 2.0);
    style.spacing.item_spacing = egui::vec2(10.0 * scale, 8.0 * scale);
    style.spacing.button_padding = egui::vec2(14.0 * scale, 10.0 * scale);
    style.spacing.interact_size = egui::vec2(120.0 * scale, 38.0 * scale);
    style.spacing.window_margin = egui::Margin::symmetric(12.0 * scale, 12.0 * scale);
    style.text_styles = [
        (
            egui::TextStyle::Heading,
            egui::FontId::proportional(22.0 * scale),
        ),
        (
            egui::TextStyle::Name("Section".into()),
            egui::FontId::proportional(16.0 * scale),
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::proportional(14.0 * scale),
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::proportional(13.0 * scale),
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::monospace(13.0 * scale),
        ),
        (
            egui::TextStyle::Small,
            egui::FontId::proportional(12.0 * scale),
        ),
    ]
    .into();
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_theme() {
        let ctx = egui::Context::default();

        // Initial state


        // Apply our theme
        apply(&ctx);

        // Verify it was applied by checking a couple of properties
        let new_style = ctx.style();
        assert_eq!(new_style.visuals.panel_fill, BG);
        assert_eq!(new_style.visuals.window_fill, CARD_BG);
        assert_eq!(new_style.visuals.override_text_color, Some(FG));
        assert_eq!(new_style.visuals.widgets.active.bg_fill, ACCENT);
    }
}
