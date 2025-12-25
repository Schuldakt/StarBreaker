//! Tree view widget for file browser

use eframe::egui;
use std::collections::HashMap;

/// Tree node state
#[derive(Default)]
pub struct TreeState {
    /// Expanded nodes (path -> is_expanded)
    expanded: HashMap<String, bool>,
    /// Selected node path
    selected: Option<String>,
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_expanded(&self, path: &str) -> bool {
        self.expanded.get(path).copied().unwrap_or(false)
    }
    
    pub fn toggle(&mut self, path: &str) {
        let current = self.is_expanded(path);
        self.expanded.insert(path.to_string(), !current);
    }
    
    pub fn select(&mut self, path: &str) {
        self.selected = Some(path.to_string());
    }
    
    pub fn is_selected(&self, path: &str) -> bool {
        self.selected.as_ref().map(|s| s.as_str()) == Some(path)
    }
}

/// Tree node data
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(name: impl Into<String>, path: impl Into<String>, is_directory: bool) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_directory,
            children: Vec::new(),
        }
    }
    
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }
    
    /// Sort children: directories first, then alphabetically
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        // Recursively sort
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

/// Tree view widget
pub struct TreeView {
    state: TreeState,
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            state: TreeState::new(),
        }
    }
    
    /// Show tree view UI
    pub fn show<F>(&mut self, ui: &mut egui::Ui, root: &TreeNode, on_select: &mut F)
    where
        F: FnMut(&str),
    {
        self.show_node(ui, root, 0, on_select);
    }
    
    fn show_node<F>(&mut self, ui: &mut egui::Ui, node: &TreeNode, depth: usize, on_select: &mut F)
    where
        F: FnMut(&str),
    {
        let indent = depth as f32 * 16.0;
        
        ui.horizontal(|ui| {
            ui.add_space(indent);
            
            // Expand/collapse icon for directories
            if node.is_directory && !node.children.is_empty() {
                let is_expanded = self.state.is_expanded(&node.path);
                let icon = if is_expanded { "▼" } else { "▶" };
                
                if ui.small_button(icon).clicked() {
                    self.state.toggle(&node.path);
                }
            } else {
                ui.add_space(20.0); // Space for alignment
            }
            
            // Icon
            let icon = if node.is_directory {
                "📁"
            } else {
                match node.name.rsplit('.').next() {
                    Some("dds") => "🖼",
                    Some("cgf") | Some("chr") | Some("skin") => "🎭",
                    Some("mtl") => "🎨",
                    Some("xml") | Some("json") => "📄",
                    Some("dcb") => "💾",
                    _ => "📄",
                }
            };
            
            ui.label(icon);
            
            // Node name
            let is_selected = self.state.is_selected(&node.path);
            let text = if is_selected {
                egui::RichText::new(&node.name).strong()
            } else {
                egui::RichText::new(&node.name)
            };
            
            if ui.selectable_label(is_selected, text).clicked() {
                self.state.select(&node.path);
                on_select(&node.path);
            }
        });
        
        // Show children if expanded
        if node.is_directory && self.state.is_expanded(&node.path) {
            for child in &node.children {
                self.show_node(ui, child, depth + 1, on_select);
            }
        }
    }
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}
