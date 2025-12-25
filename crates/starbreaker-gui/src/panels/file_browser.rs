//! File browser panel

use crate::state::AppState;
use crate::widgets::{TreeView, TreeNode};
use eframe::egui;
use std::sync::Arc;
use parking_lot::RwLock;

/// File browser panel
pub struct FileBrowserPanel {
    state: Arc<RwLock<AppState>>,
    tree_view: TreeView,
    tree_root: Option<TreeNode>,
}

impl FileBrowserPanel {
    /// Create new file browser panel
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            tree_view: TreeView::new(),
            tree_root: None,
        }
    }
    
    /// Open archive dialog
    pub fn open_archive_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("P4K Archive", &["p4k"])
            .pick_file()
        {
            let mut state = self.state.write();
            if let Err(e) = state.open_archive(path) {
                state.set_status(format!("Error opening archive: {}", e));
            } else {
                // Build tree from VFS
                drop(state); // Release write lock
                self.rebuild_tree();
            }
        }
    }
    
    /// Rebuild tree from current VFS
    fn rebuild_tree(&mut self) {
        let state = self.state.read();
        
        if let Some(_vfs) = &state.vfs {
            // For now, create a simple tree structure
            // TODO: Actually enumerate VFS contents when mount points support it
            let mut root = TreeNode::new("Archive", "/", true);
            
            // Placeholder structure
            let mut data_node = TreeNode::new("Data", "/Data", true);
            data_node.add_child(TreeNode::new("Objects", "/Data/Objects", true));
            data_node.add_child(TreeNode::new("Textures", "/Data/Textures", true));
            
            let mut libs_node = TreeNode::new("Libs", "/Libs", true);
            libs_node.add_child(TreeNode::new("Materials", "/Libs/Materials", true));
            
            root.add_child(data_node);
            root.add_child(libs_node);
            root.sort_children();
            
            self.tree_root = Some(root);
        } else {
            self.tree_root = None;
        }
    }
    
    /// Show file browser UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("File Browser");
        ui.separator();
        
        // Open archive button
        if ui.button("📁 Open P4K Archive").clicked() {
            self.open_archive_dialog();
        }
        
        ui.separator();
        
        // Show file tree if available
        if let Some(root) = &self.tree_root {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let state = self.state.clone();
                    self.tree_view.show(ui, root, &mut |path| {
                        let mut state = state.write();
                        state.select_file(path.to_string());
                        state.set_status(format!("Selected: {}", path));
                    });
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("No archive loaded");
                ui.label("Click 'Open P4K Archive' to begin");
            });
        }
    }
}
