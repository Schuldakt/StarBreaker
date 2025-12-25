//! Preview panel for viewing files

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// File preview mode
enum PreviewMode {
    Text(String),
    Hex(Vec<u8>),
    Image,
    Model,
    Unsupported,
}

/// Preview panel
pub struct PreviewPanel {
    state: Arc<RwLock<AppState>>,
    current_preview: Option<PreviewMode>,
}

impl PreviewPanel {
    /// Create new preview panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            current_preview: None,
        }
    }
    
    /// Determine preview mode from file extension
    fn get_preview_mode(file_path: &str) -> PreviewMode {
        let ext = file_path.rsplit('.').next().unwrap_or("");
        
        match ext.to_lowercase().as_str() {
            "txt" | "xml" | "json" | "cfg" | "ini" => PreviewMode::Text(String::new()),
            "dds" | "png" | "jpg" | "jpeg" => PreviewMode::Image,
            "cgf" | "chr" | "skin" | "cga" => PreviewMode::Model,
            _ => PreviewMode::Hex(Vec::new()),
        }
    }
    
    /// Show preview UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let state = self.state.read();
        
        if let Some(file_path) = &state.selected_file {
            ui.heading(format!("Preview: {}", file_path.rsplit('/').next().unwrap_or(file_path)));
            ui.separator();
            
            // Determine what kind of preview to show
            let preview_mode = Self::get_preview_mode(file_path);
            
            match preview_mode {
                PreviewMode::Text(_) => {
                    ui.heading("Text Preview");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label("TODO: Load and display text file");
                            ui.monospace("Sample text content would appear here...");
                        });
                }
                PreviewMode::Hex(_) => {
                    ui.heading("Hex Viewer");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label("TODO: Load and display hex dump");
                            ui.monospace("00000000  50 4B 03 04 14 00 00 00  |PK......|");
                        });
                }
                PreviewMode::Image => {
                    ui.heading("Image Preview");
                    ui.separator();
                    ui.label("TODO: Load and display image");
                    ui.label("(DDS decompression will be implemented)");
                }
                PreviewMode::Model => {
                    ui.heading("3D Model Preview");
                    ui.separator();
                    ui.label("TODO: 3D viewport with mesh rendering");
                    ui.label("(Model preview requires renderer integration)");
                }
                PreviewMode::Unsupported => {
                    ui.heading("Binary File");
                    ui.separator();
                    ui.label("Preview not available for this file type");
                    ui.label("Use context menu to extract or export");
                }
            }
            
            ui.separator();
            
            // File info
            ui.group(|ui| {
                ui.label("File Information");
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.monospace(file_path);
                });
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(file_path.rsplit('.').next().unwrap_or("unknown").to_uppercase());
                });
            });
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
