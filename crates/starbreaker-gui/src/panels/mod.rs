//! UI panels

mod file_browser;
mod preview;
mod status;
mod inspector;
mod search;
mod settings;

pub use file_browser::FileBrowserPanel;
pub use preview::PreviewPanel;
pub use status::StatusPanel;
pub use inspector::InspectorPanel;
pub use search::SearchPanel;
pub use settings::SettingsPanel;
