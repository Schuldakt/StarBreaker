//! Main application state and UI logic

use crate::state::AppState;
use crate::theme::Theme;
use crate::panels::{FileBrowserPanel, PreviewPanel, StatusPanel, InspectorPanel, SearchPanel};
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// StarBreaker GUI application
pub struct StarBreakerApp {
    /// Application state
    #[allow(dead_code)]
    state: Arc<RwLock<AppState>>,
    
    /// UI theme
    theme: Theme,
    
    /// File browser panel
    file_browser: FileBrowserPanel,
    
    /// Preview panel
    preview: PreviewPanel,
    
    /// Status bar panel
    status: StatusPanel,
    
    /// Inspector panel
    inspector: InspectorPanel,
    
    /// Search panel
    search: SearchPanel,
}

impl StarBreakerApp {
    /// Create new application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts and visuals
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        cc.egui_ctx.set_style(style);
        
        let state = Arc::new(RwLock::new(AppState::new()));
        
        Self {
            state: state.clone(),
            theme: Theme::dark(),
            file_browser: FileBrowserPanel::new(state.clone()),
            preview: PreviewPanel::new(state.clone()),
            status: StatusPanel::new(state.clone()),
            inspector: InspectorPanel::new(state.clone()),
            search: SearchPanel::new(state),
        }
    }
    
    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            // Open P4K file
            self.file_browser.open_archive_dialog();
        }F)) {
            // Toggle search
            self.search.toggle();
        }
        
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::
        
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Q)) {
            // Quit application
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for StarBreakerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle shortcuts
        self.handle_shortcuts(ctx);
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open P4K Archive...").clicked() {
                        self.file_browser.open_archive_dialog();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button("Toggle Theme").clicked() {
                        self.theme.toggle();
                        let style = if self.theme.is_dark() {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        };
                        ctx.set_visuals(style);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // TODO: Show about dialog
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status.show(ui);
        });
        
        // File browser (left panel)
        eguInspector (right panel)
        egui::SidePanel::right("inspector")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.inspector.show(ui);
            });
        
        // Preview panel (center)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Search bar if enabled
            self.search.show(ui);
            if self.search.show_search {
                ui.separator();
            }
            
            .resizable(true)
            .show(ctx, |ui| {
                self.file_browser.show(ui);
            });
        
        // Preview panel (center)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.preview.show(ui);
        });
    }
}
