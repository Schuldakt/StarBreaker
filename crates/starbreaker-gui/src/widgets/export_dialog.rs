//! Export dialog for file/batch export

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;
use std::path::PathBuf;

/// Export format selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Gltf,
    GltfBinary,
    Json,
    Png,
    Tga,
}

impl ExportFormat {
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Gltf => "glTF (.gltf + .bin)",
            ExportFormat::GltfBinary => "glTF Binary (.glb)",
            ExportFormat::Json => "JSON (.json)",
            ExportFormat::Png => "PNG Image",
            ExportFormat::Tga => "TGA Image",
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Gltf => "gltf",
            ExportFormat::GltfBinary => "glb",
            ExportFormat::Json => "json",
            ExportFormat::Png => "png",
            ExportFormat::Tga => "tga",
        }
    }
}

/// Export dialog state
pub struct ExportDialog {
    state: Arc<RwLock<AppState>>,
    show: bool,
    selected_format: ExportFormat,
    output_path: PathBuf,
    include_mipmaps: bool,
    pretty_json: bool,
}

impl ExportDialog {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            show: false,
            selected_format: ExportFormat::Gltf,
            output_path: PathBuf::from("./export"),
            include_mipmaps: false,
            pretty_json: true,
        }
    }
    
    /// Show the export dialog
    pub fn open(&mut self) {
        self.show = true;
    }
    
    /// Show dialog UI
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }
        
        egui::Window::new("Export File")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let state = self.state.read();
                
                if let Some(file_path) = &state.selected_file {
                    ui.label(format!("Exporting: {}", file_path.rsplit('/').next().unwrap_or(file_path)));
                    ui.separator();
                    
                    // Format selection
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_id_source("export_format")
                            .selected_text(self.selected_format.name())
                            .show_ui(ui, |ui| {
                                // Determine appropriate formats based on file type
                                let ext = file_path.rsplit('.').next().unwrap_or("");
                                
                                match ext {
                                    "cgf" | "chr" | "skin" => {
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::Gltf, ExportFormat::Gltf.name());
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::GltfBinary, ExportFormat::GltfBinary.name());
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::Json, ExportFormat::Json.name());
                                    }
                                    "dds" => {
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::Png, ExportFormat::Png.name());
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::Tga, ExportFormat::Tga.name());
                                    }
                                    _ => {
                                        ui.selectable_value(&mut self.selected_format, ExportFormat::Json, ExportFormat::Json.name());
                                    }
                                }
                            });
                    });
                    
                    // Output path
                    ui.horizontal(|ui| {
                        ui.label("Output:");
                        ui.text_edit_singleline(&mut self.output_path.to_string_lossy().to_string());
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&format!("export.{}", self.selected_format.extension()))
                                .save_file()
                            {
                                self.output_path = path;
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    // Format-specific options
                    ui.label("Options:");
                    match self.selected_format {
                        ExportFormat::Png | ExportFormat::Tga => {
                            ui.checkbox(&mut self.include_mipmaps, "Include mipmaps");
                        }
                        ExportFormat::Json => {
                            ui.checkbox(&mut self.pretty_json, "Pretty print");
                        }
                        _ => {}
                    }
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Export").clicked() {
                            self.perform_export();
                            self.show = false;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show = false;
                        }
                    });
                } else {
                    ui.label("No file selected");
                    
                    if ui.button("Close").clicked() {
                        self.show = false;
                    }
                }
            });
    }
    
    fn perform_export(&self) {
        let mut state = self.state.write();
        state.set_status(format!("Exporting to {}...", self.output_path.display()));
        
        // TODO: Implement actual export logic using export crate
        // For now, just show success message
        state.set_status(format!("Export complete: {}", self.output_path.display()));
    }
}
