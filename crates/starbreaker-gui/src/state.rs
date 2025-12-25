//! Application state management

use starbreaker_vfs::VfsTree;
use std::path::PathBuf;

/// Application state
pub struct AppState {
    /// Currently opened VFS tree
    pub vfs: Option<VfsTree>,
    
    /// Currently selected file path
    pub selected_file: Option<String>,
    
    /// Last opened P4K path
    pub last_p4k_path: Option<PathBuf>,
    
    /// Status message
    pub status_message: String,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {
            vfs: None,
            selected_file: None,
            last_p4k_path: None,
            status_message: "Ready".to_string(),
        }
    }
    
    /// Open a P4K archive
    pub fn open_archive(&mut self, path: PathBuf) -> anyhow::Result<()> {
        use starbreaker_vfs::mount::P4kMount;
        use std::sync::Arc;
        
        let vfs = VfsTree::new();
        let mount = P4kMount::new(0, "game", &path);
        vfs.add_mount(Arc::new(mount));
        
        self.vfs = Some(vfs);
        self.last_p4k_path = Some(path.clone());
        self.status_message = format!("Opened: {}", path.display());
        
        Ok(())
    }
    
    /// Select a file in the VFS
    pub fn select_file(&mut self, path: String) {
        self.selected_file = Some(path);
    }
    
    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_file = None;
    }
    
    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
