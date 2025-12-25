//! Status bar panel

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// Status bar panel
pub struct StatusPanel {
    state: Arc<RwLock<AppState>>,
}

impl StatusPanel {
    /// Create new status panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }
    
    /// Show status bar UI
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let state = self.state.read();
            ui.label(&state.status_message);
            
            // Show archive info if loaded
            if let Some(path) = &state.last_p4k_path {
                ui.separator();
                ui.label(format!("Archive: {}", path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")));
            }
        });
    }
}
