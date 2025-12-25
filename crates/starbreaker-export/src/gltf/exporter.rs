//! glTF exporter implementation

use super::*;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// glTF export options
#[derive(Debug, Clone)]
pub struct GltfExportOptions {
    /// Export as GLB (single binary file) instead of separate JSON + BIN
    pub use_glb: bool,
    /// Include normals in export
    pub export_normals: bool,
    /// Include UVs in export
    pub export_uvs: bool,
    /// Include vertex colors
    pub export_colors: bool,
    /// Include tangents
    pub export_tangents: bool,
    /// Include skin weights/indices
    pub export_skin: bool,
    /// Pretty-print JSON
    pub pretty_json: bool,
}

impl Default for GltfExportOptions {
    fn default() -> Self {
        Self {
            use_glb: false,
            export_normals: true,
            export_uvs: true,
            export_colors: false,
            export_tangents: false,
            export_skin: true,
            pretty_json: true,
        }
    }
}

/// glTF export errors
#[derive(Debug, thiserror::Error)]
pub enum GltfExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid mesh data: {0}")]
    InvalidMeshData(String),
}

pub type GltfResult<T> = Result<T, GltfExportError>;

/// glTF exporter
pub struct GltfExporter {
    options: GltfExportOptions,
    binary_data: Vec<u8>,
    accessors: Vec<Accessor>,
    buffer_views: Vec<BufferView>,
}

impl GltfExporter {
    /// Create a new glTF exporter
    pub fn new(options: GltfExportOptions) -> Self {
        Self {
            options,
            binary_data: Vec::new(),
            accessors: Vec::new(),
            buffer_views: Vec::new(),
        }
    }

    /// Export CGF mesh to glTF file
    pub fn export_mesh(
        &mut self,
        mesh: &starbreaker_parsers::cgf::Mesh,
        output_path: impl AsRef<Path>,
    ) -> GltfResult<()> {
        let output_path = output_path.as_ref();
        
        // Build glTF structure
        let gltf = self.build_gltf_from_mesh(mesh)?;
        
        if self.options.use_glb {
            self.write_glb(&gltf, output_path)?;
        } else {
            self.write_separate_files(&gltf, output_path)?;
        }
        
        Ok(())
    }

    /// Build glTF structure from CGF mesh
    fn build_gltf_from_mesh(&mut self, mesh: &starbreaker_parsers::cgf::Mesh) -> GltfResult<Gltf> {
        // Reset state
        self.binary_data.clear();
        self.accessors.clear();
        self.buffer_views.clear();

        // Build primitive with attributes
        let mut attributes = HashMap::new();
        
        // Positions (required)
        let position_accessor = self.add_positions(&mesh.vertices)?;
        attributes.insert("POSITION".to_string(), position_accessor);

        // Normals
        if self.options.export_normals {
            let normal_accessor = self.add_normals(&mesh.vertices)?;
            attributes.insert("NORMAL".to_string(), normal_accessor);
        }

        // UVs
        if self.options.export_uvs && !mesh.vertices.is_empty() && !mesh.vertices[0].uv.is_empty() {
            let uv_accessor = self.add_uvs(&mesh.vertices)?;
            attributes.insert("TEXCOORD_0".to_string(), uv_accessor);
        }

        // Indices
        let indices_accessor = self.add_indices(&mesh.faces)?;

        // Build primitive
        let primitive = Primitive {
            attributes,
            indices: Some(indices_accessor),
            material: Some(0), // Default material
            mode: Some(MODE_TRIANGLES),
        };

        // Build mesh
        let gltf_mesh = Mesh {
            name: Some(mesh.name.clone()),
            primitives: vec![primitive],
        };

        // Build default material
        let material = Material {
            name: Some("DefaultMaterial".to_string()),
            pbr_metallic_roughness: Some(PbrMetallicRoughness {
                base_color_factor: Some([1.0, 1.0, 1.0, 1.0]),
                metallic_factor: Some(0.0),
                roughness_factor: Some(0.5),
            }),
        };

        // Build buffer
        let buffer = Buffer {
            uri: Some("data.bin".to_string()),
            byte_length: self.binary_data.len(),
        };

        // Build scene
        let scene = Scene {
            name: Some("Scene".to_string()),
            nodes: vec![0],
        };

        // Build node
        let node = Node {
            name: Some("MeshNode".to_string()),
            mesh: Some(0),
            skin: None,
            translation: None,
            rotation: None,
            scale: None,
            children: vec![],
        };

        // Build final glTF
        Ok(Gltf {
            asset: Asset {
                version: "2.0".to_string(),
                generator: Some("StarBreaker glTF Exporter".to_string()),
            },
            scene: Some(0),
            scenes: vec![scene],
            nodes: vec![node],
            meshes: vec![gltf_mesh],
            materials: vec![material],
            accessors: self.accessors.clone(),
            buffer_views: self.buffer_views.clone(),
            buffers: vec![buffer],
            skins: vec![],
        })
    }

    /// Add position data
    fn add_positions(&mut self, vertices: &[starbreaker_parsers::cgf::Vertex]) -> GltfResult<usize> {
        let offset = self.binary_data.len();
        let mut min = [f32::MAX, f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN, f32::MIN];

        for vertex in vertices {
            for i in 0..3 {
                self.binary_data.extend_from_slice(&vertex.position[i].to_le_bytes());
                min[i] = min[i].min(vertex.position[i]);
                max[i] = max[i].max(vertex.position[i]);
            }
        }

        self.add_accessor(offset, vertices.len(), "VEC3", COMPONENT_TYPE_FLOAT, Some(min.to_vec()), Some(max.to_vec()), Some(TARGET_ARRAY_BUFFER))
    }

    /// Add normal data
    fn add_normals(&mut self, vertices: &[starbreaker_parsers::cgf::Vertex]) -> GltfResult<usize> {
        let offset = self.binary_data.len();

        for vertex in vertices {
            for i in 0..3 {
                self.binary_data.extend_from_slice(&vertex.normal[i].to_le_bytes());
            }
        }

        self.add_accessor(offset, vertices.len(), "VEC3", COMPONENT_TYPE_FLOAT, None, None, Some(TARGET_ARRAY_BUFFER))
    }

    /// Add UV data
    fn add_uvs(&mut self, vertices: &[starbreaker_parsers::cgf::Vertex]) -> GltfResult<usize> {
        let offset = self.binary_data.len();

        for vertex in vertices {
            let uv = vertex.uv.first().unwrap_or(&[0.0, 0.0]);
            self.binary_data.extend_from_slice(&uv[0].to_le_bytes());
            self.binary_data.extend_from_slice(&uv[1].to_le_bytes());
        }

        self.add_accessor(offset, vertices.len(), "VEC2", COMPONENT_TYPE_FLOAT, None, None, Some(TARGET_ARRAY_BUFFER))
    }

    /// Add index data
    fn add_indices(&mut self, faces: &[starbreaker_parsers::cgf::Face]) -> GltfResult<usize> {
        let offset = self.binary_data.len();
        let count = faces.len() * 3;

        for face in faces {
            for &index in &face.indices {
                self.binary_data.extend_from_slice(&(index as u16).to_le_bytes());
            }
        }

        self.add_accessor(offset, count, "SCALAR", COMPONENT_TYPE_UNSIGNED_SHORT, None, None, Some(TARGET_ELEMENT_ARRAY_BUFFER))
    }

    /// Add accessor and buffer view
    fn add_accessor(&mut self, offset: usize, count: usize, accessor_type: &str, component_type: u32, min: Option<Vec<f32>>, max: Option<Vec<f32>>, target: Option<u32>) -> GltfResult<usize> {
        let byte_length = self.binary_data.len() - offset;
        
        let buffer_view_index = self.buffer_views.len();
        self.buffer_views.push(BufferView {
            buffer: 0,
            byte_offset: Some(offset),
            byte_length,
            byte_stride: None,
            target,
        });

        let accessor_index = self.accessors.len();
        self.accessors.push(Accessor {
            buffer_view: Some(buffer_view_index),
            byte_offset: None,
            component_type,
            count,
            accessor_type: accessor_type.to_string(),
            max,
            min,
        });

        Ok(accessor_index)
    }

    /// Write separate JSON + BIN files
    fn write_separate_files(&self, gltf: &Gltf, output_path: &Path) -> GltfResult<()> {
        // Write JSON
        let json_path = output_path.with_extension("gltf");
        let json = if self.options.pretty_json {
            serde_json::to_string_pretty(gltf)?
        } else {
            serde_json::to_string(gltf)?
        };
        std::fs::write(&json_path, json)?;

        // Write BIN
        let bin_path = output_path.with_extension("bin");
        std::fs::write(&bin_path, &self.binary_data)?;

        Ok(())
    }

    /// Write GLB (binary glTF)
    fn write_glb(&self, gltf: &Gltf, output_path: &Path) -> GltfResult<()> {
        let glb_path = output_path.with_extension("glb");
        let mut file = std::fs::File::create(&glb_path)?;

        // GLB header
        file.write_all(b"glTF")?; // Magic
        file.write_all(&2u32.to_le_bytes())?; // Version

        // Calculate lengths
        let json = serde_json::to_string(gltf)?;
        let json_len = json.len();
        let json_padding = (4 - (json_len % 4)) % 4;
        let bin_len = self.binary_data.len();
        let bin_padding = (4 - (bin_len % 4)) % 4;
        
        let total_len = 12 + 8 + json_len + json_padding + 8 + bin_len + bin_padding;
        file.write_all(&(total_len as u32).to_le_bytes())?;

        // JSON chunk
        file.write_all(&((json_len + json_padding) as u32).to_le_bytes())?;
        file.write_all(&0x4E4F534Au32.to_le_bytes())?; // "JSON"
        file.write_all(json.as_bytes())?;
        for _ in 0..json_padding {
            file.write_all(&[0x20])?; // Space padding
        }

        // BIN chunk
        file.write_all(&((bin_len + bin_padding) as u32).to_le_bytes())?;
        file.write_all(&0x004E4942u32.to_le_bytes())?; // "BIN\0"
        file.write_all(&self.binary_data)?;
        for _ in 0..bin_padding {
            file.write_all(&[0x00])?; // Zero padding
        }

        Ok(())
    }
}
