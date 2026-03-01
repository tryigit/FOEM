mod app;
mod diagnostics;
#[allow(dead_code)]
mod features;
mod license_text;
#[allow(dead_code)]
mod theme;
mod update_manager;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!("FOEM v{}", VERSION))
            .with_inner_size([1060.0, 680.0])
            .with_min_inner_size([800.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FOEM",
        options,
        Box::new(|cc| Ok(Box::new(app::FOEMApp::new(cc)))),
    )
}
