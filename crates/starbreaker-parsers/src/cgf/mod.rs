// starbreaker-parsers/src/cgf/mod.rs
//! CGF (CryEngine Geometry Format) Parser
//!
//! CGF files are the primary 3D model format used by CryEngine and Star Citizen.
//! They contain mesh geometry, materials, bones, and other model data organized
//! as a series of chunks.
//!
//! # Format Structure
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      CGF File Structure                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                    File Header                          ││
//! │  │  - Magic: "CryTek" / "#ivo" / "CrCh"                    ││
//! │  │  - Version, Chunk Count, Chunk Table Offset             ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                   Chunk Table                           ││
//! │  │  - Array of chunk headers with type, offset, size       ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                    Chunk Data                           ││
//! │  │  - Mesh, Material, Node, Bone chunks                    ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod chunks;
mod mesh;
mod bones;

pub use chunks::{ChunkType, ChunkHeader, CgfChunk};
pub use mesh::{Mesh, Vertex, Face, SubMesh, MeshSubset};
pub use bones::{Skeleton, Bone, BonePhysics};

use std::io::{Read, Seek, SeekFrom, BufReader};
use std::collections::HashMap;
use std::path::Path;

use crate::traits::{
    Parser, ParseResult, ParseError,
    ParseOptions, ParseProgress, ParsePhase, ProgressCallback
};

/// CGF file magic signatures
const CRYTEK_MAGIC: &[u8; 8] = b"CryTek\0\0";
const IVO_MAGIC: &[u8; 4] = b"#ivo";
const CRCH_MAGIC: &[u8; 4] = b"CrCh";

/// CGF file versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgfVersion {
    /// Legacy CryEngine 2/3 format
    Legacy(u32),
    /// Star Citizen Ivo format
    Ivo(u32),
    /// CrCh format (newer)
    CrCh(u32),
}

/// Parsed CGF model
#[derive(Debug)]
pub struct CgfModel {
    /// File version
    pub version: CgfVersion,
    /// All chunks in the file
    pub chunks: Vec<CgfChunk>,
    /// Mesh data (extracted from mesh chunks)
    pub meshes: Vec<Mesh>,
    /// Material references
    pub materials: Vec<MaterialRef>,
    /// Skeleton (if present)
    pub skeleton: Option<Skeleton>,
    /// Node hierarchy
    pub nodes: Vec<Node>,
    /// Morph targets
    pub morph_targets: Vec<MorphTarget>,
    /// Physics data
    pub physics: Option<PhysicsProxy>,
}

impl CgfModel {
    /// Create empty model
    pub fn new(version: CgfVersion) -> Self {
        Self {
            version,
            chunks: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            skeleton: None,
            nodes: Vec::new(),
            morph_targets: Vec::new(),
            physics: None,
        }
    }

    /// Get total vertex count across all meshes
    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|m| m.vertices.len()).sum()
    }

    /// Get total face count across all meshes
    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|m| m.faces.len()).sum()
    }

    /// Check if model has skeletal animation data
    pub fn is_skinned(&self) -> bool {
        self.skeleton.is_some() && self.meshes.iter().any(|m| m.has_bone_weights())
    }

    /// Get all unique texture paths referenced by materials
    pub fn texture_paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self.materials.iter()
            .flat_map(|m| m.texture_paths())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Material reference
#[derive(Debug, Clone)]
pub struct MaterialRef {
    /// Material name
    pub name: String,
    /// Material index
    pub index: u32,
    /// Shader name
    pub shader: String,
    /// Texture slots
    pub textures: MaterialTextures,
    /// Shader parameters
    pub params: HashMap<String, ShaderParam>,
    /// Sub-materials (for multi-materials)
    pub sub_materials: Vec<MaterialRef>,
}

impl MaterialRef {
    /// Get all texture paths
    pub fn texture_paths(&self) -> Vec<&str> {
        let mut paths = Vec::new();
        if let Some(ref p) = self.textures.diffuse { paths.push(p.as_str()); }
        if let Some(ref p) = self.textures.normal { paths.push(p.as_str()); }
        if let Some(ref p) = self.textures.specular { paths.push(p.as_str()); }
        if let Some(ref p) = self.textures.emissive { paths.push(p.as_str()); }
        for sub in &self.sub_materials {
            paths.extend(sub.texture_paths());
        }
        paths
    }
}

/// Material texture slots
#[derive(Debug, Clone, Default)]
pub struct MaterialTextures {
    pub diffuse: Option<String>,
    pub normal: Option<String>,
    pub specular: Option<String>,
    pub emissive: Option<String>,
    pub detail: Option<String>,
    pub blend: Option<String>,
    pub height: Option<String>,
    pub decal: Option<String>,
    pub custom: HashMap<String, String>,
}

/// Shader parameter value
#[derive(Debug, Clone)]
pub enum ShaderParam {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Int(i32),
    Bool(bool),
    String(String),
    Texture(String),
}

/// Scene node in hierarchy
#[derive(Debug, Clone)]
pub struct Node {
    /// Node name
    pub name: String,
    /// Node ID
    pub id: u32,
    /// Parent node ID (0 = root)
    pub parent_id: u32,
    /// Local transform matrix (4x4 row-major)
    pub transform: [[f32; 4]; 4],
    /// Position
    pub position: [f32; 3],
    /// Rotation (quaternion)
    pub rotation: [f32; 4],
    /// Scale
    pub scale: [f32; 3],
    /// Attached mesh index (if any)
    pub mesh_index: Option<usize>,
    /// Material index
    pub material_index: Option<u32>,
    /// Properties
    pub properties: HashMap<String, String>,
}

impl Node {
    /// Check if this is a root node
    pub fn is_root(&self) -> bool {
        self.parent_id == 0
    }
}

/// Morph target for facial animation
#[derive(Debug, Clone)]
pub struct MorphTarget {
    /// Target name
    pub name: String,
    /// Target mesh index
    pub mesh_index: usize,
    /// Vertex deltas (index, position delta)
    pub vertex_deltas: Vec<(u32, [f32; 3])>,
    /// Normal deltas
    pub normal_deltas: Vec<(u32, [f32; 3])>,
}

/// Physics proxy for collision
#[derive(Debug, Clone)]
pub struct PhysicsProxy {
    /// Proxy type
    pub proxy_type: PhysicsProxyType,
    /// Vertices
    pub vertices: Vec<[f32; 3]>,
    /// Indices
    pub indices: Vec<u32>,
    /// Physics material
    pub material: String,
}

/// Physics proxy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsProxyType {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Mesh,
    Convex,
}

/// CGF Parser
pub struct CgfParser;

impl CgfParser {
    /// Create a new CGF parser
    pub fn new() -> Self {
        Self
    }

    /// Parse file header
    fn parse_header<R: Read + Seek>(&self, reader: &mut R) -> ParseResult<(CgfVersion, CgfHeader)> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        let (version, header_size) = if &magic[0..8] == CRYTEK_MAGIC {
            // Legacy CryTek format
            let mut header_data = [0u8; 8];
            reader.read_exact(&mut header_data)?;

            let file_type = u32::from_le_bytes([header_data[0], header_data[1], header_data[2], header_data[3]]);
            let version = u32::from_le_bytes([header_data[4], header_data[5], header_data[6], header_data[7]]);

            (CgfVersion::Legacy(version), 16)
        } else if &magic[0..4] == IVO_MAGIC {
            // Ivo format (Star Citizen)
            let version = u32::from_le_bytes([magic[4], magic[5], magic[6], magic[7]]);
            (CgfVersion::Ivo(version), 8)
        } else if &magic[0..4] == CRCH_MAGIC {
            // CrCh format
            let version = u32::from_le_bytes([magic[4], magic[5], magic[6], magic[7]]);
            (CgfVersion::CrCh(version), 8)
        } else {
            return Err(ParseError::InvalidMagic {
                expected: CRYTEK_MAGIC.to_vec(),
                found: magic.to_vec(),
            });
        };

        // Read rest of header
        let mut header_bytes = [0u8; 8];
        reader.read_exact(&mut header_bytes)?;

        let chunk_count = u32::from_le_bytes([
            header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3]
        ]);
        let chunk_table_offset = u32::from_le_bytes([
            header_bytes[4], header_bytes[5], header_bytes[6], header_bytes[7]
        ]);

        Ok((version, CgfHeader {
            chunk_count,
            chunk_table_offset,
        }))
    }

    /// Parse chunk table
    fn parse_chunk_table<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &CgfHeader,
        version: CgfVersion,
    ) -> ParseResult<Vec<ChunkHeader>> {
        reader.seek(SeekFrom::Start(header.chunk_table_offset as u64))?;

        let mut chunks = Vec::with_capacity(header.chunk_count as usize);

        for _ in 0..header.chunk_count {
            let chunk_header = self.parse_chunk_header(reader, version)?;
            chunks.push(chunk_header);
        }

        Ok(chunks)
    }

    /// Parse a single chunk header
    fn parse_chunk_header<R: Read>(&self, reader: &mut R, version: CgfVersion) -> ParseResult<ChunkHeader> {
        let mut header_data = [0u8; 16];
        reader.read_exact(&mut header_data)?;

        let chunk_type = u32::from_le_bytes([
            header_data[0], header_data[1], header_data[2], header_data[3]
        ]);
        let chunk_version = u32::from_le_bytes([
            header_data[4], header_data[5], header_data[6], header_data[7]
        ]);
        let offset = u32::from_le_bytes([
            header_data[8], header_data[9], header_data[10], header_data[11]
        ]);
        let id = u32::from_le_bytes([
            header_data[12], header_data[13], header_data[14], header_data[15]
        ]);

        // For Ivo format, read additional size field
        let size = match version {
            CgfVersion::Ivo(_) | CgfVersion::CrCh(_) => {
                let mut size_bytes = [0u8; 4];
                reader.read_exact(&mut size_bytes)?;
                u32::from_le_bytes(size_bytes)
            }
            CgfVersion::Legacy(_) => 0, // Size determined by next chunk offset
        };

        Ok(ChunkHeader {
            chunk_type: ChunkType::from_u32(chunk_type),
            version: chunk_version,
            offset,
            id,
            size,
        })
    }

    /// Parse mesh chunk data
    fn parse_mesh_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<Mesh> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read mesh header
        let mut mesh_header = [0u8; 48];
        reader.read_exact(&mut mesh_header)?;

        let flags = u32::from_le_bytes([mesh_header[0], mesh_header[1], mesh_header[2], mesh_header[3]]);
        let vertex_count = u32::from_le_bytes([mesh_header[4], mesh_header[5], mesh_header[6], mesh_header[7]]) as usize;
        let face_count = u32::from_le_bytes([mesh_header[8], mesh_header[9], mesh_header[10], mesh_header[11]]) as usize;
        let uv_count = u32::from_le_bytes([mesh_header[12], mesh_header[13], mesh_header[14], mesh_header[15]]) as usize;

        // Read vertices
        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            let mut pos_data = [0u8; 12];
            reader.read_exact(&mut pos_data)?;

            let position = [
                f32::from_le_bytes([pos_data[0], pos_data[1], pos_data[2], pos_data[3]]),
                f32::from_le_bytes([pos_data[4], pos_data[5], pos_data[6], pos_data[7]]),
                f32::from_le_bytes([pos_data[8], pos_data[9], pos_data[10], pos_data[11]]),
            ];

            vertices.push(Vertex {
                position,
                normal: [0.0, 1.0, 0.0],
                uv: [[0.0, 0.0]],
                color: None,
                tangent: None,
                bone_weights: None,
                bone_indices: None,
            });
        }

        // Read normals
        for vertex in vertices.iter_mut() {
            let mut normal_data = [0u8; 12];
            reader.read_exact(&mut normal_data)?;

            vertex.normal = [
                f32::from_le_bytes([normal_data[0], normal_data[1], normal_data[2], normal_data[3]]),
                f32::from_le_bytes([normal_data[4], normal_data[5], normal_data[6], normal_data[7]]),
                f32::from_le_bytes([normal_data[8], normal_data[9], normal_data[10], normal_data[11]]),
            ];
        }

        // Read UVs
        for vertex in vertices.iter_mut() {
            let mut uv_data = [0u8; 8];
            reader.read_exact(&mut uv_data)?;

            vertex.uv = [[
                f32::from_le_bytes([uv_data[0], uv_data[1], uv_data[2], uv_data[3]]),
                f32::from_le_bytes([uv_data[4], uv_data[5], uv_data[6], uv_data[7]]),
            ]];
        }

        // Read faces
        let mut faces = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            let mut face_data = [0u8; 12];
            reader.read_exact(&mut face_data)?;

            faces.push(Face {
                indices: [
                    u32::from_le_bytes([face_data[0], face_data[1], face_data[2], face_data[3]]),
                    u32::from_le_bytes([face_data[4], face_data[5], face_data[6], face_data[7]]),
                    u32::from_le_bytes([face_data[8], face_data[9], face_data[10], face_data[11]]),
                ],
                material_id: 0,
                smoothing_group: 0,
            });
        }

        Ok(Mesh {
            name: String::new(),
            vertices,
            faces,
            subsets: Vec::new(),
            bounding_box: None,
        })
    }
}

impl Default for CgfParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for CgfParser {
    type Output = CgfModel;

    fn extensions(&self) -> &[&str] {
        &["cgf", "cga", "skin", "chr"]
    }

    fn magic_bytes(&self) -> Option<&[u8]> {
        // Can't return single magic - multiple formats supported
        None
    }

    fn name(&self) -> &str {
        "CryEngine Geometry Parser"
    }

    fn parse_with_options<R: Read + Seek>(
        &self,
        mut reader: R,
        options: &ParseOptions,
        progress: Option<ProgressCallback>,
    ) -> ParseResult<Self::Output> {
        // Report start
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::ReadingHeader,
                bytes_processed: 0,
                total_bytes: None,
                current_item: None,
                items_processed: 0,
                total_items: None,
            });
        }

        // Parse header
        let (version, header) = self.parse_header(&mut reader)?;

        // Parse chunk table
        let chunk_headers = self.parse_chunk_table(&mut reader, &header, version)?;

        // Create model
        let mut model = CgfModel::new(version);

        // Parse each chunk
        for (idx, chunk_header) in chunk_headers.iter().enumerate() {
            if let Some(ref cb) = progress {
                cb(ParseProgress {
                    phase: ParsePhase::ParsingRecords,
                    bytes_processed: chunk_header.offset as u64,
                    total_bytes: None,
                    current_item: Some(format!("{:?}", chunk_header.chunk_type)),
                    items_processed: idx as u64,
                    total_items: Some(chunk_headers.len() as u64),
                });
            }

            match chunk_header.chunk_type {
                ChunkType::Mesh | ChunkType::MeshSubsets => {
                    if let Ok(mesh) = self.parse_mesh_chunk(&mut reader, chunk_header) {
                        model.meshes.push(mesh);
                    }
                }
                ChunkType::Node => {
                    // Parse node chunk
                }
                ChunkType::Material => {
                    // Parse material chunk
                }
                ChunkType::BoneAnim | ChunkType::BoneNameList => {
                    // Parse bone data
                }
                _ => {
                    if !options.skip_unknown_chunks {
                        return Err(ParseError::UnknownChunkType {
                            chunk_type: chunk_header.chunk_type.to_u32(),
                        });
                    }
                }
            }
        }

        // Report completion
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::Complete,
                bytes_processed: reader.stream_position()?,
                total_bytes: None,
                current_item: None,
                items_processed: chunk_headers.len() as u64,
                total_items: Some(chunk_headers.len() as u64),
            });
        }

        Ok(model)
    }
}

/// CGF file header
#[derive(Debug)]
struct CgfHeader {
    chunk_count: u32,
    chunk_table_offset: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgf_version() {
        assert_ne!(CgfVersion::Legacy(1), CgfVersion::Ivo(1));
    }

    #[test]
    fn test_material_texture_paths() {
        let mat = MaterialRef {
            name: "test".into(),
            index: 0,
            shader: "illum".into(),
            textures: MaterialTextures {
                diffuse: Some("textures/diffuse.dds".into()),
                normal: Some("textures/normal.dds".into()),
                ..Default::default()
            },
            params: HashMap::new(),
            sub_materials: Vec::new(),
        };

        let paths = mat.texture_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"textures/diffuse.dds"));
    }
}