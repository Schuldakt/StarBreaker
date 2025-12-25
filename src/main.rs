//! StarBreaker - Star Citizen Asset Browser and Extractor
//! 
//! Main entry point that launches the GUI application

fn main() -> Result<(), eframe::Error> {
    // Configure native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("StarBreaker - Star Citizen Asset Browser"),
        ..Default::default()
    };

    // Run the GUI application
    eframe::run_native(
        "StarBreaker",
        options,
        Box::new(|cc| Ok(Box::new(starbreaker_gui::StarBreakerApp::new(cc)))),
    )
}

