//! UI panels

mod file_browser;
mod preview;
mod status;
mod inspector;
mod search;
mod settings;
mod debug_console;

pub use file_browser::FileBrowserPanel;
pub use preview::PreviewPanel;
pub use status::StatusPanel;
pub use inspector::InspectorPanel;
pub use search::SearchPanel;
pub use settings::SettingsPanel;
pub use debug_console::DebugConsolePanel;
