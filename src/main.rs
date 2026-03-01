mod app;
mod diagnostics;
mod gms_repair;
mod update_manager;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!("FOEM v{}", VERSION))
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FOEM",
        options,
        Box::new(|cc| Ok(Box::new(app::FOEMApp::new(cc)))),
    )
}
