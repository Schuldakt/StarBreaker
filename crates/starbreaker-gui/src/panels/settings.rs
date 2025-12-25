use std::sync::Arc;
use parking_lot::RwLock;
use crate::state::AppState;
use crate::theme::Theme;

/// Settings panel for application configuration
pub struct SettingsPanel {
    state: Arc<RwLock<AppState>>,
    pub show: bool,
    
    // Settings (editable)
    game_path: String,
    theme_mode: ThemeMode,
    default_export_format: String,
    export_include_mipmaps: bool,
    export_pretty_json: bool,
    cache_size_mb: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum ThemeMode {
    Dark,
    Light,
}

impl SettingsPanel {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            show: false,
            game_path: String::new(),
            theme_mode: ThemeMode::Dark,
            default_export_format: "glTF".to_string(),
            export_include_mipmaps: true,
            export_pretty_json: true,
            cache_size_mb: 512,
        }
    }
    
    pub fn open(&mut self) {
        self.show = true;
    }
    
    pub fn close(&mut self) {
        self.show = false;
    }
    
    /// Show settings dialog
    pub fn show(&mut self, ctx: &egui::Context, theme: &mut Theme) {
        if !self.show {
            return;
        }
        
        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Game Path Section
                    ui.heading("Game Settings");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Star Citizen Path:");
                        ui.text_edit_singleline(&mut self.game_path);
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.game_path = path.display().to_string();
                            }
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.label("📁 The folder containing Star Citizen's Data.p4k file");
                    
                    ui.add_space(16.0);
                    
                    // Appearance Section
                    ui.heading("Appearance");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        ui.radio_value(&mut self.theme_mode, ThemeMode::Dark, "Dark");
                        ui.radio_value(&mut self.theme_mode, ThemeMode::Light, "Light");
                    });
                    
                    ui.add_space(16.0);
                    
                    // Export Settings Section
                    ui.heading("Export Defaults");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Default Format:");
                        egui::ComboBox::new("default_export_format", "")
                            .selected_text(&self.default_export_format)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.default_export_format, "glTF".to_string(), "glTF (.gltf)");
                                ui.selectable_value(&mut self.default_export_format, "GLB".to_string(), "glTF Binary (.glb)");
                                ui.selectable_value(&mut self.default_export_format, "FBX".to_string(), "FBX (.fbx)");
                                ui.selectable_value(&mut self.default_export_format, "JSON".to_string(), "JSON (.json)");
                            });
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.checkbox(&mut self.export_include_mipmaps, "Include mipmaps in texture exports");
                    ui.checkbox(&mut self.export_pretty_json, "Pretty-print JSON exports");
                    
                    ui.add_space(16.0);
                    
                    // Performance Section
                    ui.heading("Performance");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Cache Size:");
                        ui.add(egui::Slider::new(&mut self.cache_size_mb, 128..=2048)
                            .suffix(" MB")
                            .step_by(128.0));
                    });
                    
                    ui.add_space(8.0);
                    
                    if ui.button("Clear Cache").clicked() {
                        let mut state = self.state.write();
                        state.set_status("Cache cleared".to_string());
                    }
                    
                    ui.add_space(16.0);
                    
                    // Keyboard Shortcuts Section
                    ui.heading("Keyboard Shortcuts");
                    ui.separator();
                    
                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Open Archive:");
                            ui.label("⌘ + O");
                            ui.end_row();
                            
                            ui.label("Search:");
                            ui.label("⌘ + F");
                            ui.end_row();
                            
                            ui.label("Export:");
                            ui.label("⌘ + E");
                            ui.end_row();
                            
                            ui.label("Settings:");
                            ui.label("⌘ + ,");
                            ui.end_row();
                            
                            ui.label("Quit:");
                            ui.label("⌘ + Q");
                            ui.end_row();
                        });
                });
                
                ui.separator();
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.apply_settings(theme);
                        self.show = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show = false;
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Reset to Defaults").clicked() {
                            self.reset_defaults();
                        }
                    });
                });
            });
    }
    
    fn apply_settings(&mut self, theme: &mut Theme) {
        // Apply theme change
        match self.theme_mode {
            ThemeMode::Dark => {
                if theme.is_light() {
                    theme.toggle();
                }
            }
            ThemeMode::Light => {
                if !theme.is_light() {
                    theme.toggle();
                }
            }
        }
        
        // Update state with game path
        if !self.game_path.is_empty() {
            let mut state = self.state.write();
            state.set_status(format!("Settings saved. Game path: {}", self.game_path));
        }
        
        // TODO: Persist settings to file
    }
    
    fn reset_defaults(&mut self) {
        self.game_path.clear();
        self.theme_mode = ThemeMode::Dark;
        self.default_export_format = "glTF".to_string();
        self.export_include_mipmaps = true;
        self.export_pretty_json = true;
        self.cache_size_mb = 512;
    }
}
