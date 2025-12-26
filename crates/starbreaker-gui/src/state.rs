//! Application state management

use starbreaker_vfs::VfsTree;
use starbreaker_parsers::P4kArchive;
use std::path::PathBuf;
use std::sync::Arc;

/// Application state
pub struct AppState {
    /// Currently opened VFS tree
    pub vfs: Option<VfsTree>,
    
    /// Currently opened P4K archive (for direct access)
    pub archive: Option<Arc<P4kArchive>>,
    
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
            archive: None,
            selected_file: None,
            last_p4k_path: None,
            status_message: "Ready".to_string(),
        }
    }
    
    /// Open a P4K archive
    pub fn open_archive(&mut self, path: PathBuf) -> anyhow::Result<()> {
        use starbreaker_parsers::traits::Parser;
        use starbreaker_parsers::P4kParser;
        use starbreaker_vfs::mount::P4kMount;
        
        // Parse the P4K archive
        eprintln!("[DEBUG] Loading P4K: {}", path.display());
        self.status_message = format!("Loading {}...", path.display());
        let parser = P4kParser::new();
        
        eprintln!("[DEBUG] Parsing archive...");
        let archive = parser.parse_file(&path)?;
        eprintln!("[INFO] Parsed {} entries", archive.entries.len());
        
        let archive = Arc::new(archive);
        
        // Create VFS mount
        let vfs = VfsTree::new();
        let mount = P4kMount::new(0, "game", &path, archive.clone());
        vfs.add_mount(Arc::new(mount));
        
        self.vfs = Some(vfs);
        self.archive = Some(archive.clone());
        self.last_p4k_path = Some(path.clone());
        self.status_message = format!("Opened: {} ({} files)", 
            path.display(), 
            archive.entries.len());
        
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
