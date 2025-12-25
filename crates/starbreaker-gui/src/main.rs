//! StarBreaker GUI entry point

use starbreaker_gui::StarBreakerApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("StarBreaker - Star Citizen Asset Viewer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "StarBreaker",
        native_options,
        Box::new(|cc| Ok(Box::new(StarBreakerApp::new(cc)))),
    )
}

