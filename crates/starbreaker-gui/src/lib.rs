//! StarBreaker GUI
//!
//! Graphical user interface for viewing and extracting Star Citizen assets

pub mod app;
pub mod state;
pub mod theme;
pub mod panels;
pub mod widgets;

// Re-export main app for easy access
pub use app::StarBreakerApp;
