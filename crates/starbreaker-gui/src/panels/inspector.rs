//! Inspector panel for viewing file properties

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// Inspector panel
pub struct InspectorPanel {
    state: Arc<RwLock<AppState>>,
}

impl InspectorPanel {
    /// Create new inspector panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }
    
    /// Show inspector UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        ui.separator();
        
        let state = self.state.read();
        
        if let Some(file_path) = &state.selected_file {
            let filename = file_path.rsplit('/').next().unwrap_or(file_path);
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            
            // File properties
            ui.group(|ui| {
                ui.label(egui::RichText::new("File Properties").strong());
                ui.separator();
                
                Self::property_row(ui, "Name", filename);
                Self::property_row(ui, "Extension", &ext.to_uppercase());
                Self::property_row(ui, "Path", file_path);
                
                // Type-specific properties
                match ext.as_str() {
                    "dcb" => {
                        ui.separator();
                        ui.label(egui::RichText::new("DataCore").strong());
                        Self::property_row(ui, "Records", "TODO");
                        Self::property_row(ui, "Structs", "TODO");
                    }
                    "cgf" | "chr" | "skin" => {
                        ui.separator();
                        ui.label(egui::RichText::new("Mesh").strong());
                        Self::property_row(ui, "Vertices", "TODO");
                        Self::property_row(ui, "Faces", "TODO");
                        Self::property_row(ui, "Materials", "TODO");
                    }
                    "dds" => {
                        ui.separator();
                        ui.label(egui::RichText::new("Texture").strong());
                        Self::property_row(ui, "Format", "TODO");
                        Self::property_row(ui, "Dimensions", "TODO");
                        Self::property_row(ui, "Mipmaps", "TODO");
                    }
                    _ => {}
                }
            });
            
            // Actions
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("Actions").strong());
                ui.separator();
                
                if ui.button("📋 Copy Path").clicked() {
                    ui.output_mut(|o| o.copied_text = file_path.clone());
                }
                
                if ui.button("💾 Extract").clicked() {
                    // TODO: Trigger extract
                }
                
                if ui.button("📤 Export").clicked() {
                    // TODO: Trigger export
                }
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No file selected");
            });
        }
    }
    
    fn property_row(ui: &mut egui::Ui, label: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).strong());
            ui.label(value);
        });
    }
}
