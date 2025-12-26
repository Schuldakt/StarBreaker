pub struct AssetDependencyGraph {
    nodes: HashMap<AssetId, AssetNode>,
    edges: Vec<(AssetId, AssetId, DependencyType)>,
}

impl AssetDependencyGraph {
    pub fn build_from_dcb(dcb: &DataCore) -> Self { /* ... */}
    pub fn get_dependencies(&self, id: AssetId) -> Vec<&AssetNode> {/* ... */}
    pub fn get_dependants(&self, id: AssetId) -> Vec<&AssetNode> {/* ... */}
}