// starbreaker-parsers/src/cgf/mesh.rs
//! CGF mesh data structures

use serde::{Deserialize, Serialize};

/// A 3D mesh from CGF file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    /// Mesh name
    pub name: String,
    /// All vertices
    pub vertices: Vec<Vertex>,
    /// Face indices
    pub faces: Vec<Face>,
    /// Mesh subsets (material groups)
    pub subsets: Vec<MeshSubset>,
    /// Axis-aligned bounding box
    pub bounding_box: Option<BoundingBox>,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            faces: Vec::new(),
            subsets: Vec::new(),
            bounding_box: None,
        }
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get face count
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Get triangle count (same as face count for triangulated meshes)
    pub fn triangle_count(&self) -> usize {
        self.faces.len()
    }

    /// Check if mesh has UV coordinates
    pub fn has_uvs(&self) -> bool {
        self.vertices.first()
            .map(|v| !v.uv.is_empty() && v.uv[0] != [0.0, 0.0])
            .unwrap_or(false)
    }

    /// Check if mesh has vertex colors
    pub fn has_colors(&self) -> bool {
        self.vertices.iter().any(|v| v.color.is_some())
    }

    /// Check if mesh has bone weights (skinned)
    pub fn has_bone_weights(&self) -> bool {
        self.vertices.iter().any(|v| v.bone_weights.is_some())
    }

    /// Check if mesh has tangent data
    pub fn has_tangents(&self) -> bool {
        self.vertices.iter().any(|v| v.tangent.is_some())
    }

    /// Get UV channel count
    pub fn uv_channel_count(&self) -> usize {
        self.vertices.first()
            .map(|v| v.uv.len())
            .unwrap_or(0)
    }

    /// Calculate bounding box from vertices
    pub fn calculate_bounding_box(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min = self.vertices[0].position;
        let mut max = self.vertices[0].position;

        for vertex in &self.vertices {
            for i in 0..3 {
                min[i] = min[i].min(vertex.position[i]);
                max[i] = max[i].max(vertex.position[i]);
            }
        }

        self.bounding_box = Some(BoundingBox { min, max });
    }

    /// Flip normals (reverse face winding)
    pub fn flip_normals(&mut self) {
        // Flip vertex normals
        for vertex in &mut self.vertices {
            for n in &mut vertex.normal {
                *n = -*n;
            }
        }

        // Reverse face winding
        for face in &mut self.faces {
            face.indices.swap(1, 2);
        }
    }

    /// Get all unique material IDs used by faces
    pub fn material_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.faces.iter()
            .map(|f| f.material_id)
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Split mesh by material ID
    pub fn split_by_material(&self) -> Vec<Mesh> {
        let mut result = Vec::new();

        for mat_id in self.material_ids() {
            let mut sub_mesh = Mesh::new(format!("{}_{}", self.name, mat_id));

            // Collect faces for this material
            let faces: Vec<&Face> = self.faces.iter()
                .filter(|f| f.material_id == mat_id)
                .collect();

            // Build vertex index mapping
            let mut index_map = std::collections::HashMap::new();
            let mut new_vertices = Vec::new();

            for face in &faces {
                let mut new_indices = [0u32; 3];
                for (i, &idx) in face.indices.iter().enumerate() {
                    let new_idx = *index_map.entry(idx).or_insert_with(|| {
                        let idx = new_vertices.len() as u32;
                        new_vertices.push(self.vertices[idx as usize].clone());
                        idx
                    });
                    new_indices[i] = new_idx;
                }
                sub_mesh.faces.push(Face {
                    indices: new_indices,
                    material_id: mat_id,
                    smoothing_group: face.smoothing_group,
                });
            }

            sub_mesh.vertices = new_vertices;
            sub_mesh.calculate_bounding_box();
            result.push(sub_mesh);
        }

        result
    }

    /// Merge another mesh into this one
    pub fn merge(&mut self, other: &Mesh) {
        let vertex_offset = self.vertices.len() as u32;

        // Add vertices
        self.vertices.extend_from_slice(&other.vertices);

        // Add faces with adjusted indices
        for face in &other.faces {
            self.faces.push(Face {
                indices: [
                    face.indices[0] + vertex_offset,
                    face.indices[1] + vertex_offset,
                    face.indices[2] + vertex_offset,
                ],
                material_id: face.material_id,
                smoothing_group: face.smoothing_group,
            });
        }

        // Recalculate bounding box
        self.calculate_bounding_box();
    }

    /// Get positions as flat f32 array (for GPU upload)
    pub fn positions_flat(&self) -> Vec<f32> {
        self.vertices.iter()
            .flat_map(|v| v.position)
            .collect()
    }

    /// Get normals as flat f32 array
    pub fn normals_flat(&self) -> Vec<f32> {
        self.vertices.iter()
            .flat_map(|v| v.normal)
            .collect()
    }

    /// Get indices as flat u32 array
    pub fn indices_flat(&self) -> Vec<u32> {
        self.faces.iter()
            .flat_map(|f| f.indices)
            .collect()
    }
}

/// A single vertex with all attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// Position in 3D space
    pub position: [f32; 3],
    /// Normal vector
    pub normal: [f32; 3],
    /// UV coordinates (multiple channels)
    pub uv: Vec<[f32; 2]>,
    /// Vertex color (RGBA)
    pub color: Option<[u8; 4]>,
    /// Tangent vector (with handedness in W)
    pub tangent: Option<[f32; 4]>,
    /// Bone weights (up to 4 influences)
    pub bone_weights: Option<[f32; 4]>,
    /// Bone indices (up to 4 influences)
    pub bone_indices: Option<[u8; 4]>,
}

impl Vertex {
    /// Create a vertex with just position
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            normal: [0.0, 1.0, 0.0],
            uv: vec![[0.0, 0.0]],
            color: None,
            tangent: None,
            bone_weights: None,
            bone_indices: None,
        }
    }

    /// Check if vertex has valid bone weights
    pub fn is_skinned(&self) -> bool {
        if let Some(weights) = &self.bone_weights {
            weights.iter().any(|&w| w > 0.0)
        } else {
            false
        }
    }

    /// Normalize bone weights to sum to 1.0
    pub fn normalize_bone_weights(&mut self) {
        if let Some(ref mut weights) = self.bone_weights {
            let sum: f32 = weights.iter().sum();
            if sum > 0.0 {
                for w in weights.iter_mut() {
                    *w /= sum;
                }
            }
        }
    }
}

/// A triangle face
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    /// Vertex indices (always triangles)
    pub indices: [u32; 3],
    /// Material ID for this face
    pub material_id: u32,
    /// Smoothing group
    pub smoothing_group: u32,
}

impl Face {
    /// Create a face from three vertex indices
    pub fn new(i0: u32, i1: u32, i2: u32) -> Self {
        Self {
            indices: [i0, i1, i2],
            material_id: 0,
            smoothing_group: 0,
        }
    }

    /// Get face normal from vertex positions
    pub fn calculate_normal(&self, vertices: &[Vertex]) -> [f32; 3] {
        let v0 = vertices[self.indices[0] as usize].position;
        let v1 = vertices[self.indices[1] as usize].position;
        let v2 = vertices[self.indices[2] as usize].position;

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product
        let normal = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        // Normalize
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if len > 0.0 {
            [normal[0] / len, normal[1] / len, normal[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }
}

/// Mesh subset (for multi-material meshes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSubset {
    /// First index in the index buffer
    pub first_index: u32,
    /// Number of indices
    pub num_indices: u32,
    /// First vertex index
    pub first_vertex: u32,
    /// Number of vertices
    pub num_vertices: u32,
    /// Material ID
    pub material_id: u32,
    /// Bounding box
    pub bounding_box: Option<BoundingBox>,
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner
    pub min: [f32; 3],
    /// Maximum corner
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Get the center point
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }

    /// Get the size (extent)
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Get the largest dimension
    pub fn max_extent(&self) -> f32 {
        let size = self.size();
        size[0].max(size[1]).max(size[2])
    }

    /// Check if a point is inside the box
    pub fn contains(&self, point: [f32; 3]) -> bool {
        point[0] >= self.min[0] && point[0] <= self.max[0] &&
        point[1] >= self.min[1] && point[1] <= self.max[1] &&
        point[2] >= self.min[2] && point[2] <= self.max[2]
    }

    /// Expand to include a point
    pub fn expand(&mut self, point: [f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(point[i]);
            self.max[i] = self.max[i].max(point[i]);
        }
    }

    /// Merge with another bounding box
    pub fn merge(&mut self, other: &BoundingBox) {
        self.expand(other.min);
        self.expand(other.max);
    }
}

/// Sub-mesh for rendering
#[derive(Debug, Clone)]
pub struct SubMesh {
    /// Name
    pub name: String,
    /// Vertex range start
    pub vertex_start: u32,
    /// Vertex count
    pub vertex_count: u32,
    /// Index range start
    pub index_start: u32,
    /// Index count
    pub index_count: u32,
    /// Material index
    pub material_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mesh() -> Mesh {
        let mut mesh = Mesh::new("test");
        
        mesh.vertices = vec![
            Vertex::new([0.0, 0.0, 0.0]),
            Vertex::new([1.0, 0.0, 0.0]),
            Vertex::new([0.0, 1.0, 0.0]),
            Vertex::new([1.0, 1.0, 0.0]),
        ];

        mesh.faces = vec![
            Face::new(0, 1, 2),
            Face::new(1, 3, 2),
        ];

        mesh
    }

    #[test]
    fn test_mesh_counts() {
        let mesh = make_test_mesh();
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn test_bounding_box() {
        let mut mesh = make_test_mesh();
        mesh.calculate_bounding_box();

        let bb = mesh.bounding_box.unwrap();
        assert_eq!(bb.min, [0.0, 0.0, 0.0]);
        assert_eq!(bb.max, [1.0, 1.0, 0.0]);
        assert_eq!(bb.center(), [0.5, 0.5, 0.0]);
    }

    #[test]
    fn test_face_normal() {
        let mesh = make_test_mesh();
        let normal = mesh.faces[0].calculate_normal(&mesh.vertices);
        
        // Face should be pointing in +Z direction
        assert!(normal[2] > 0.9);
    }

    #[test]
    fn test_positions_flat() {
        let mesh = make_test_mesh();
        let positions = mesh.positions_flat();
        
        assert_eq!(positions.len(), 12); // 4 vertices * 3 components
        assert_eq!(&positions[0..3], &[0.0, 0.0, 0.0]);
        assert_eq!(&positions[3..6], &[1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_vertex_normalize_weights() {
        let mut vertex = Vertex::new([0.0, 0.0, 0.0]);
        vertex.bone_weights = Some([0.5, 0.3, 0.1, 0.1]);
        vertex.normalize_bone_weights();

        let weights = vertex.bone_weights.unwrap();
        let sum: f32 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }
}