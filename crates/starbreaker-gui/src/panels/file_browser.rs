//! File browser panel

use crate::state::AppState;
use crate::widgets::{TreeNode, TreeView};
use crate::panels::DebugConsolePanel;
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
    pub fn open_archive_dialog(&mut self, debug_console: &mut DebugConsolePanel) {
        debug_console.info("File dialog opened");
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("P4K Archive", &["p4k"])
            .pick_file()
        {
            let path_str = path.display().to_string();
            debug_console.info(format!("Selected file: {}", path_str));
            
            let mut state = self.state.write();
            if let Err(e) = state.open_archive(path) {
                let error_msg = format!("Error opening archive: {}", e);
                state.set_status(error_msg.clone());
                debug_console.error(&error_msg);
                eprintln!("[ERROR] {}", error_msg);
            } else {
                debug_console.info(format!("Successfully opened archive: {}", path_str));
                eprintln!("[INFO] Successfully opened archive: {}", path_str);
                // Build tree from VFS
                drop(state); // Release write lock
                
                debug_console.debug("Building tree from archive...");
                self.rebuild_tree();
                
                let entry_count = self.tree_root.as_ref().map(|t| t.children.len()).unwrap_or(0);
                debug_console.info(format!("Tree rebuilt with {} root entries", entry_count));
                eprintln!("[DEBUG] Tree rebuilt with {} entries", entry_count);
            }
        } else {
            debug_console.debug("File dialog cancelled");
        }
    }
    
    /// Rebuild tree from current VFS
    fn rebuild_tree(&mut self) {
        let state = self.state.read();
        
        if let Some(archive) = &state.archive {
            // Build tree from P4K archive
            let dir_tree = archive.build_tree();
            
            // Convert P4K DirectoryNode to our TreeNode
            fn convert_node(name: &str, path: &str, dir_node: &starbreaker_parsers::p4k::DirectoryNode) -> TreeNode {
                let mut node = TreeNode::new(name, path, !dir_node.is_file);
                
                for child_name in dir_node.sorted_children() {
                    if let Some(child_dir_node) = dir_node.children.get(child_name) {
                        let child_path = if path == "/" || path.is_empty() {
                            format!("/{}", child_name)
                        } else {
                            format!("{}/{}", path, child_name)
                        };
                        
                        let child_tree_node = convert_node(child_name, &child_path, child_dir_node);
                        node.add_child(child_tree_node);
                    }
                }
                
                node
            }
            
            let root = convert_node("Archive", "/", &dir_tree);
            self.tree_root = Some(root);
            // Expand root by default
            self.tree_view.set_expanded("/", true);
        } else {
            self.tree_root = None;
        }
    }
    
    /// Show file browser UI
    pub fn show(&mut self, ui: &mut egui::Ui, debug_console: &mut DebugConsolePanel) {
        ui.heading("File Browser");
        ui.separator();
        
        // Open archive button
        if ui.button("📁 Open P4K Archive").clicked() {
            self.open_archive_dialog(debug_console);
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
                    // Context menu
                    ui.interact(ui.max_rect(), ui.id().with("tree_context"), egui::Sense::click())
                        .context_menu(|ui| {
                            if let Some(selected) = state.read().selected_file.clone() {
                                ui.label(format!("Actions for: {}", 
                                    selected.rsplit('/').next().unwrap_or(&selected)));
                                ui.separator();
                                
                                if ui.button("📋 Copy Path").clicked() {
                                    ui.output_mut(|o| o.copied_text = selected.clone());
                                    ui.close_menu();
                                }
                                
                                if ui.button("💾 Extract...").clicked() {
                                    let mut state_write = state.write();
                                    state_write.set_status("Extract not yet implemented");
                                    ui.close_menu();
                                }
                                
                                if ui.button("📤 Export...").clicked() {
                                    let mut state_write = state.write();
                                    state_write.set_status("Export not yet implemented");
                                    ui.close_menu();
                                }
                            } else {
                                ui.label("No file selected");
                            }
                        });                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("No archive loaded");
                ui.label("Click 'Open P4K Archive' to begin");
            });
        }
    }
}
