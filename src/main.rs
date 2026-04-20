mod adaptive_engine;
mod app;
mod diagnostics;
mod exec;
mod features;
mod license_text;
mod theme;
mod update_manager;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn display_version() -> String {
    if VERSION.starts_with('0') {
        format!("{} Beta", VERSION)
    } else {
        VERSION.to_string()
    }
}

fn load_icon() -> Option<eframe::egui::IconData> {
    let bytes = include_bytes!("../favicon.ico");
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Some(eframe::egui::IconData {
        rgba,
        width,
        height,
    })
}

fn main() -> eframe::Result {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(format!("FOEM v{}", display_version()))
        .with_inner_size([1060.0, 680.0])
        .with_min_inner_size([800.0, 520.0]);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "FOEM",
        options,
        Box::new(|cc| Ok(Box::new(app::FOEMApp::new(cc)))),
    )
}
