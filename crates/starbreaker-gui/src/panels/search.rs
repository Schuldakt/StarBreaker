//! Search panel for finding files

use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// Search panel
pub struct SearchPanel {
    state: Arc<RwLock<AppState>>,
    query: String,
    filter_type: String,
    results: Vec<String>,
    pub show_search: bool,
}

impl SearchPanel {
    /// Create new search panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            query: String::new(),
            filter_type: "All".to_string(),
            results: Vec::new(),
            show_search: false,
        }
    }
    
    /// Toggle search visibility
    pub fn toggle(&mut self) {
        self.show_search = !self.show_search;
    }
    
    /// Show search panel if visible
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.show_search {
            return;
        }
        
        ui.heading("🔍 Search");
        ui.separator();
        
        // Search input
        ui.horizontal(|ui| {
            ui.label("Query:");
            let response = ui.text_edit_singleline(&mut self.query);
            
            if response.changed() || ui.button("Search").clicked() {
                self.perform_search();
            }
        });
        
        // Filter options
        ui.horizontal(|ui| {
            ui.label("Type:");
            egui::ComboBox::new("search_filter", "")
                .selected_text(&self.filter_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_type, "All".to_string(), "All");
                    ui.selectable_value(&mut self.filter_type, "Models".to_string(), "Models (.cgf, .chr)");
                    ui.selectable_value(&mut self.filter_type, "Textures".to_string(), "Textures (.dds)");
                    ui.selectable_value(&mut self.filter_type, "Data".to_string(), "Data (.dcb, .xml)");
                });
        });
        
        ui.separator();
        
        // Results
        if self.results.is_empty() {
            ui.label(egui::RichText::new("No results").italics());
        } else {
            ui.label(format!("{} results found", self.results.len()));
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for result in &self.results {
                        if ui.selectable_label(false, result).clicked() {
                            let mut state = self.state.write();
                            state.select_file(result.clone());
                        }
                    }
                });
        }
    }
    
    fn perform_search(&mut self) {
        // TODO: Implement actual VFS search
        // For now, just placeholder results
        self.results.clear();
        
        if !self.query.is_empty() {
            self.results.push(format!("Result matching '{}'", self.query));
            self.results.push("/Data/Objects/example.cgf".to_string());
            self.results.push("/Textures/example.dds".to_string());
        }
    }
}
