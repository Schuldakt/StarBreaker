//! Preview panel for viewing files

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// Preview panel
pub struct PreviewPanel {
    state: Arc<RwLock<AppState>>,
}

impl PreviewPanel {
    /// Create new preview panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }
    
    /// Show preview UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let state = self.state.read();
        
        if let Some(file_path) = &state.selected_file {
            ui.heading(format!("Preview: {}", file_path));
            ui.separator();
            
            // TODO: Implement file preview based on type
            ui.label("(Preview coming soon)");
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                ui.heading("StarBreaker");
                ui.label("Star Citizen Asset Viewer & Extractor");
                ui.add_space(20.0);
                ui.label("Select a file from the browser to preview");
            });
        }
    }
}
