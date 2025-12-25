use std::sync::Arc;
use parking_lot::RwLock;
use crate::state::AppState;

/// Debug console panel for logging and debugging
pub struct DebugConsolePanel {
    state: Arc<RwLock<AppState>>,
    pub show: bool,
    messages: Vec<LogMessage>,
    auto_scroll: bool,
    filter_level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Debug => egui::Color32::GRAY,
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
        }
    }
    
    fn icon(&self) -> &str {
        match self {
            LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
        }
    }
}

#[derive(Debug, Clone)]
struct LogMessage {
    timestamp: String,
    level: LogLevel,
    message: String,
}

impl DebugConsolePanel {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let mut panel = Self {
            state,
            show: false,
            messages: Vec::new(),
            auto_scroll: true,
            filter_level: LogLevel::Debug,
        };
        
        // Add welcome message
        panel.log(LogLevel::Info, "Debug console initialized");
        panel.log(LogLevel::Debug, "Press ` to toggle console");
        
        panel
    }
    
    pub fn toggle(&mut self) {
        self.show = !self.show;
    }
    
    pub fn open(&mut self) {
        self.show = true;
    }
    
    pub fn close(&mut self) {
        self.show = false;
    }
    
    /// Add a log message
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.messages.push(LogMessage {
            timestamp,
            level,
            message: message.into(),
        });
        
        // Keep only last 1000 messages
        if self.messages.len() > 1000 {
            self.messages.remove(0);
        }
    }
    
    /// Add debug message
    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }
    
    /// Add info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }
    
    /// Add warning message
    pub fn warn(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }
    
    /// Add error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }
    
    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.log(LogLevel::Info, "Console cleared");
    }
    
    /// Show debug console UI
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }
        
        egui::Window::new("🐛 Debug Console")
            .default_width(800.0)
            .default_height(400.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.clear();
                    }
                    
                    ui.separator();
                    
                    ui.label("Filter:");
                    ui.radio_value(&mut self.filter_level, LogLevel::Debug, "Debug");
                    ui.radio_value(&mut self.filter_level, LogLevel::Info, "Info");
                    ui.radio_value(&mut self.filter_level, LogLevel::Warning, "Warning");
                    ui.radio_value(&mut self.filter_level, LogLevel::Error, "Error");
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} messages", self.messages.len()));
                    });
                });
                
                ui.separator();
                
                // Log messages
                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.auto_scroll);
                
                scroll_area.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    
                    for msg in &self.messages {
                        // Filter by level
                        if msg.level < self.filter_level {
                            continue;
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&msg.timestamp)
                                    .color(egui::Color32::DARK_GRAY)
                                    .monospace()
                            );
                            
                            ui.label(msg.level.icon());
                            
                            ui.label(
                                egui::RichText::new(&msg.message)
                                    .color(msg.level.color())
                                    .monospace()
                            );
                        });
                    }
                });
                
                ui.separator();
                
                // Copy all button
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy All").clicked() {
                        let all_text: String = self.messages
                            .iter()
                            .map(|m| format!("[{}] {:?}: {}", m.timestamp, m.level, m.message))
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.output_mut(|o| o.copied_text = all_text);
                        self.info("Copied to clipboard");
                    }
                    
                    if ui.button("Close").clicked() {
                        self.close();
                    }
                });
            });
    }
}
