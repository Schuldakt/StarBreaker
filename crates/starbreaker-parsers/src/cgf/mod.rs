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

use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;

use rayon::prelude::*;

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
    fn parse_chunks_parallel(&self, chunks: &[ChunkHeader], data: &[u8]) -> ParseResult<Vec<Chunk>> {
        chunks.par_iter()
            .map(|header| self.parse_chunk(header, data))
            .collect()
    }
    /// Create a new CGF parser
    pub fn new() -> Self {
        Self
    }

    /// Parse file header
    fn parse_header<R: Read + Seek>(&self, reader: &mut R) -> ParseResult<(CgfVersion, CgfHeader)> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        let (version, _header_size) = if &magic[0..8] == CRYTEK_MAGIC {
            // Legacy CryTek format
            let mut header_data = [0u8; 8];
            reader.read_exact(&mut header_data)?;

            let _file_type = u32::from_le_bytes([header_data[0], header_data[1], header_data[2], header_data[3]]);
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

        let _flags = u32::from_le_bytes([mesh_header[0], mesh_header[1], mesh_header[2], mesh_header[3]]);
        let vert_count = u32::from_le_bytes([mesh_header[4], mesh_header[5], mesh_header[6], mesh_header[7]]) as usize;
        let face_count = u32::from_le_bytes([mesh_header[8], mesh_header[9], mesh_header[10], mesh_header[11]]) as usize;
        let _uv_count = u32::from_le_bytes([mesh_header[12], mesh_header[13], mesh_header[14], mesh_header[15]]) as usize;

        // Read vertices
        let mut vertices = Vec::with_capacity(vert_count);
        for _ in 0..vert_count {
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
                uv: vec![[0.0, 0.0]],
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

            vertex.uv = vec![[
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

    /// Parse node chunk data
    fn parse_node_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<Node> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read name length and name
        let mut name_len_buf = [0u8; 4];
        reader.read_exact(&mut name_len_buf)?;
        let name_len = u32::from_le_bytes(name_len_buf) as usize;

        let mut name_bytes = vec![0u8; name_len];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read node header
        let mut node_data = [0u8; 128];
        reader.read_exact(&mut node_data)?;

        let id = u32::from_le_bytes([node_data[0], node_data[1], node_data[2], node_data[3]]);
        let parent_id = u32::from_le_bytes([node_data[4], node_data[5], node_data[6], node_data[7]]);

        // Read transform matrix (4x4, 64 bytes)
        let mut transform = [[0.0f32; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                let offset = 8 + (row * 4 + col) * 4;
                transform[row][col] = f32::from_le_bytes([
                    node_data[offset],
                    node_data[offset + 1],
                    node_data[offset + 2],
                    node_data[offset + 3],
                ]);
            }
        }

        // Extract position from matrix (last column)
        let position = [transform[0][3], transform[1][3], transform[2][3]];

        // Default rotation and scale
        let rotation = [0.0, 0.0, 0.0, 1.0]; // Identity quaternion
        let scale = [1.0, 1.0, 1.0];

        // Read mesh and material indices
        let mesh_index_raw = u32::from_le_bytes([
            node_data[72], node_data[73], node_data[74], node_data[75]
        ]);
        let material_index_raw = u32::from_le_bytes([
            node_data[76], node_data[77], node_data[78], node_data[79]
        ]);

        let mesh_index = if mesh_index_raw == 0xFFFFFFFF {
            None
        } else {
            Some(mesh_index_raw as usize)
        };

        let material_index = if material_index_raw == 0xFFFFFFFF {
            None
        } else {
            Some(material_index_raw)
        };

        Ok(Node {
            name,
            id,
            parent_id,
            transform,
            position,
            rotation,
            scale,
            mesh_index,
            material_index,
            properties: HashMap::new(),
        })
    }

    /// Parse material chunk data
    fn parse_material_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<MaterialRef> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read material name length
        let mut name_len_buf = [0u8; 4];
        reader.read_exact(&mut name_len_buf)?;
        let name_len = u32::from_le_bytes(name_len_buf) as usize;

        let mut name_bytes = vec![0u8; name_len];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read shader name length
        let mut shader_len_buf = [0u8; 4];
        reader.read_exact(&mut shader_len_buf)?;
        let shader_len = u32::from_le_bytes(shader_len_buf) as usize;

        let mut shader_bytes = vec![0u8; shader_len];
        reader.read_exact(&mut shader_bytes)?;
        let shader = String::from_utf8_lossy(&shader_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read material index
        let mut index_buf = [0u8; 4];
        reader.read_exact(&mut index_buf)?;
        let index = u32::from_le_bytes(index_buf);

        // Read texture count
        let mut tex_count_buf = [0u8; 4];
        reader.read_exact(&mut tex_count_buf)?;
        let tex_count = u32::from_le_bytes(tex_count_buf) as usize;

        // Read textures
        let mut textures = MaterialTextures::default();
        for i in 0..tex_count.min(4) {
            let mut tex_len_buf = [0u8; 4];
            reader.read_exact(&mut tex_len_buf)?;
            let tex_len = u32::from_le_bytes(tex_len_buf) as usize;

            if tex_len > 0 {
                let mut tex_bytes = vec![0u8; tex_len];
                reader.read_exact(&mut tex_bytes)?;
                let tex_path = String::from_utf8_lossy(&tex_bytes)
                    .trim_end_matches('\0')
                    .to_string();

                match i {
                    0 => textures.diffuse = Some(tex_path),
                    1 => textures.normal = Some(tex_path),
                    2 => textures.specular = Some(tex_path),
                    3 => textures.emissive = Some(tex_path),
                    _ => {}
                }
            }
        }

        Ok(MaterialRef {
            name,
            index,
            shader,
            textures,
            params: HashMap::new(),
            sub_materials: Vec::new(),
        })
    }

    /// Parse CompiledBones chunk data (0xACDC0000)
    fn parse_compiled_bones_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<Skeleton> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read bone count
        let mut bone_count_buf = [0u8; 4];
        reader.read_exact(&mut bone_count_buf)?;
        let bone_count = u32::from_le_bytes(bone_count_buf) as usize;

        let mut skeleton = Skeleton::new();

        // Read bone names
        let mut bone_names = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let mut name_len_buf = [0u8; 4];
            reader.read_exact(&mut name_len_buf)?;
            let name_len = u32::from_le_bytes(name_len_buf) as usize;

            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8_lossy(&name_bytes)
                .trim_end_matches('\0')
                .to_string();
            
            bone_names.push(name);
        }

        // Read bone data
        for (_idx, name) in bone_names.into_iter().enumerate() {
            // Read parent index
            let mut parent_buf = [0u8; 4];
            reader.read_exact(&mut parent_buf)?;
            let parent_raw = i32::from_le_bytes(parent_buf);
            let parent_index = if parent_raw < 0 {
                None
            } else {
                Some(parent_raw as usize)
            };

            // Read controller ID
            let mut controller_buf = [0u8; 4];
            reader.read_exact(&mut controller_buf)?;
            let controller_id = u32::from_le_bytes(controller_buf);

            // Read local transform matrix (4x4 = 64 bytes)
            let mut local_transform = [[0.0f32; 4]; 4];
            for row in 0..4 {
                for col in 0..4 {
                    let mut val_buf = [0u8; 4];
                    reader.read_exact(&mut val_buf)?;
                    local_transform[row][col] = f32::from_le_bytes(val_buf);
                }
            }

            // Read bind pose matrix (4x4 = 64 bytes)
            let mut bind_pose = [[0.0f32; 4]; 4];
            for row in 0..4 {
                for col in 0..4 {
                    let mut val_buf = [0u8; 4];
                    reader.read_exact(&mut val_buf)?;
                    bind_pose[row][col] = f32::from_le_bytes(val_buf);
                }
            }

            // Calculate inverse bind pose (transpose for simplified inverse)
            let mut inverse_bind_pose = [[0.0f32; 4]; 4];
            for row in 0..4 {
                for col in 0..4 {
                    inverse_bind_pose[row][col] = bind_pose[col][row];
                }
            }

            let bone = Bone {
                name,
                parent_index,
                controller_id,
                local_transform,
                bind_pose,
                inverse_bind_pose,
                physics: None,
                limits: None,
            };

            skeleton.add_bone(bone);
        }

        skeleton.build_hierarchy();

        Ok(skeleton)
    }

    /// Parse CompiledMesh chunk data (0xCCCC0000)
    /// This is the optimized runtime mesh format
    fn parse_compiled_mesh_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<Mesh> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read compiled mesh header
        let mut mesh_header = [0u8; 32];
        reader.read_exact(&mut mesh_header)?;

        let _flags = u32::from_le_bytes([mesh_header[0], mesh_header[1], mesh_header[2], mesh_header[3]]);
        let vert_count = u32::from_le_bytes([mesh_header[4], mesh_header[5], mesh_header[6], mesh_header[7]]) as usize;
        let index_count = u32::from_le_bytes([mesh_header[8], mesh_header[9], mesh_header[10], mesh_header[11]]) as usize;
        let subset_count = u32::from_le_bytes([mesh_header[12], mesh_header[13], mesh_header[14], mesh_header[15]]) as usize;
        
        // Read vertex stream count
        let stream_count = u32::from_le_bytes([mesh_header[16], mesh_header[17], mesh_header[18], mesh_header[19]]) as usize;

        // Parse vertex streams
        let mut positions = Vec::with_capacity(vert_count);
        let mut normals = Vec::with_capacity(vert_count);
        let mut uvs = Vec::with_capacity(vert_count);
        let mut colors = Vec::new();
        let mut bone_weights_list = Vec::new();
        let mut bone_indices_list = Vec::new();

        for _ in 0..stream_count {
            // Read stream type
            let mut stream_type_buf = [0u8; 4];
            reader.read_exact(&mut stream_type_buf)?;
            let stream_type = u32::from_le_bytes(stream_type_buf);

            // Read stream size
            let mut stream_size_buf = [0u8; 4];
            reader.read_exact(&mut stream_size_buf)?;
            let stream_size = u32::from_le_bytes(stream_size_buf) as usize;

            match stream_type {
                0 => {
                    // Position stream
                    for _ in 0..vert_count {
                        let mut pos_buf = [0u8; 12];
                        reader.read_exact(&mut pos_buf)?;
                        positions.push([
                            f32::from_le_bytes([pos_buf[0], pos_buf[1], pos_buf[2], pos_buf[3]]),
                            f32::from_le_bytes([pos_buf[4], pos_buf[5], pos_buf[6], pos_buf[7]]),
                            f32::from_le_bytes([pos_buf[8], pos_buf[9], pos_buf[10], pos_buf[11]]),
                        ]);
                    }
                }
                1 => {
                    // Normal stream
                    for _ in 0..vert_count {
                        let mut norm_buf = [0u8; 12];
                        reader.read_exact(&mut norm_buf)?;
                        normals.push([
                            f32::from_le_bytes([norm_buf[0], norm_buf[1], norm_buf[2], norm_buf[3]]),
                            f32::from_le_bytes([norm_buf[4], norm_buf[5], norm_buf[6], norm_buf[7]]),
                            f32::from_le_bytes([norm_buf[8], norm_buf[9], norm_buf[10], norm_buf[11]]),
                        ]);
                    }
                }
                2 => {
                    // UV stream
                    for _ in 0..vert_count {
                        let mut uv_buf = [0u8; 8];
                        reader.read_exact(&mut uv_buf)?;
                        uvs.push([
                            f32::from_le_bytes([uv_buf[0], uv_buf[1], uv_buf[2], uv_buf[3]]),
                            f32::from_le_bytes([uv_buf[4], uv_buf[5], uv_buf[6], uv_buf[7]]),
                        ]);
                    }
                }
                3 => {
                    // Color stream (already in u8 RGBA format)
                    colors.reserve(vert_count);
                    for _ in 0..vert_count {
                        let mut color_buf = [0u8; 4];
                        reader.read_exact(&mut color_buf)?;
                        colors.push(color_buf);
                    }
                }
                12 => {
                    // Skin data (bone weights and indices)
                    bone_weights_list.reserve(vert_count);
                    bone_indices_list.reserve(vert_count);
                    
                    for _ in 0..vert_count {
                        let mut skin_buf = [0u8; 32]; // 4 weights + 4 indices
                        reader.read_exact(&mut skin_buf)?;
                        
                        let weights = [
                            f32::from_le_bytes([skin_buf[0], skin_buf[1], skin_buf[2], skin_buf[3]]),
                            f32::from_le_bytes([skin_buf[4], skin_buf[5], skin_buf[6], skin_buf[7]]),
                            f32::from_le_bytes([skin_buf[8], skin_buf[9], skin_buf[10], skin_buf[11]]),
                            f32::from_le_bytes([skin_buf[12], skin_buf[13], skin_buf[14], skin_buf[15]]),
                        ];
                        
                        // Convert u16 bone indices to u8 (clamped to 255 max)
                        let indices = [
                            u16::from_le_bytes([skin_buf[16], skin_buf[17]]).min(255) as u8,
                            u16::from_le_bytes([skin_buf[18], skin_buf[19]]).min(255) as u8,
                            u16::from_le_bytes([skin_buf[20], skin_buf[21]]).min(255) as u8,
                            u16::from_le_bytes([skin_buf[22], skin_buf[23]]).min(255) as u8,
                        ];
                        
                        bone_weights_list.push(weights);
                        bone_indices_list.push(indices);
                    }
                }
                _ => {
                    // Unknown stream - skip
                    reader.seek(SeekFrom::Current(stream_size as i64))?;
                }
            }
        }

        // Fill in default values if streams were missing
        if normals.is_empty() {
            normals.resize(vert_count, [0.0, 1.0, 0.0]);
        }
        if uvs.is_empty() {
            uvs.resize(vert_count, [0.0, 0.0]);
        }

        // Build vertices
        let mut vertices = Vec::with_capacity(vert_count);
        for i in 0..vert_count {
            vertices.push(Vertex {
                position: positions.get(i).copied().unwrap_or([0.0, 0.0, 0.0]),
                normal: normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                uv: vec![uvs.get(i).copied().unwrap_or([0.0, 0.0])],
                color: colors.get(i).copied(),
                tangent: None,
                bone_weights: bone_weights_list.get(i).copied(),
                bone_indices: bone_indices_list.get(i).copied(),
            });
        }

        // Read indices
        let face_count = index_count / 3;
        let mut faces = Vec::with_capacity(face_count);
        
        for _ in 0..face_count {
            let mut index_buf = [0u8; 12];
            reader.read_exact(&mut index_buf)?;
            
            faces.push(Face {
                indices: [
                    u32::from_le_bytes([index_buf[0], index_buf[1], index_buf[2], index_buf[3]]),
                    u32::from_le_bytes([index_buf[4], index_buf[5], index_buf[6], index_buf[7]]),
                    u32::from_le_bytes([index_buf[8], index_buf[9], index_buf[10], index_buf[11]]),
                ],
                material_id: 0,
                smoothing_group: 0,
            });
        }

        // Read subsets
        let mut subsets = Vec::with_capacity(subset_count);
        for _ in 0..subset_count {
            let mut subset_buf = [0u8; 16];
            reader.read_exact(&mut subset_buf)?;
            
            let material_id = u32::from_le_bytes([subset_buf[0], subset_buf[1], subset_buf[2], subset_buf[3]]);
            let first_index = u32::from_le_bytes([subset_buf[4], subset_buf[5], subset_buf[6], subset_buf[7]]);
            let num_indices = u32::from_le_bytes([subset_buf[8], subset_buf[9], subset_buf[10], subset_buf[11]]);
            let first_vertex = u32::from_le_bytes([subset_buf[12], subset_buf[13], subset_buf[14], subset_buf[15]]);
            
            subsets.push(MeshSubset {
                material_id,
                first_index,
                num_indices,
                first_vertex,
                num_vertices: 0,
                bounding_box: None,
            });
        }

        Ok(Mesh {
            name: format!("CompiledMesh_{}", header.id),
            vertices,
            faces,
            subsets,
            bounding_box: None,
        })
    }

    /// Parse CompiledMorphTargets chunk data (0xACDC0002)
    /// Contains blend shape/morph target data for facial animation
    fn parse_compiled_morph_targets_chunk<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &ChunkHeader,
    ) -> ParseResult<Vec<MorphTarget>> {
        reader.seek(SeekFrom::Start(header.offset as u64))?;

        // Read header
        let mut morph_header = [0u8; 8];
        reader.read_exact(&mut morph_header)?;
        
        let target_count = u32::from_le_bytes([morph_header[0], morph_header[1], morph_header[2], morph_header[3]]) as usize;
        let _flags = u32::from_le_bytes([morph_header[4], morph_header[5], morph_header[6], morph_header[7]]);

        let mut morph_targets = Vec::with_capacity(target_count);

        for _ in 0..target_count {
            // Read morph target name (length-prefixed)
            let mut name_len_buf = [0u8; 4];
            reader.read_exact(&mut name_len_buf)?;
            let name_len = u32::from_le_bytes(name_len_buf) as usize;
            
            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8_lossy(&name_bytes).to_string();

            // Read weight range
            let mut range_buf = [0u8; 8];
            reader.read_exact(&mut range_buf)?;
            let _min_weight = f32::from_le_bytes([range_buf[0], range_buf[1], range_buf[2], range_buf[3]]);
            let _max_weight = f32::from_le_bytes([range_buf[4], range_buf[5], range_buf[6], range_buf[7]]);

            // Read delta count
            let mut delta_count_buf = [0u8; 4];
            reader.read_exact(&mut delta_count_buf)?;
            let delta_count = u32::from_le_bytes(delta_count_buf) as usize;

            // Read vertex deltas
            let mut vertex_deltas = Vec::with_capacity(delta_count);
            let mut normal_deltas = Vec::new();
            
            for _ in 0..delta_count {
                let mut delta_buf = [0u8; 28]; // 4 (index) + 12 (position) + 12 (normal)
                reader.read_exact(&mut delta_buf)?;
                
                let vertex_index = u32::from_le_bytes([delta_buf[0], delta_buf[1], delta_buf[2], delta_buf[3]]);
                
                let position_delta = [
                    f32::from_le_bytes([delta_buf[4], delta_buf[5], delta_buf[6], delta_buf[7]]),
                    f32::from_le_bytes([delta_buf[8], delta_buf[9], delta_buf[10], delta_buf[11]]),
                    f32::from_le_bytes([delta_buf[12], delta_buf[13], delta_buf[14], delta_buf[15]]),
                ];
                
                let normal_delta = [
                    f32::from_le_bytes([delta_buf[16], delta_buf[17], delta_buf[18], delta_buf[19]]),
                    f32::from_le_bytes([delta_buf[20], delta_buf[21], delta_buf[22], delta_buf[23]]),
                    f32::from_le_bytes([delta_buf[24], delta_buf[25], delta_buf[26], delta_buf[27]]),
                ];

                vertex_deltas.push((vertex_index, position_delta));

                // Check if normal is non-zero
                if normal_delta[0].abs() > 0.0001 
                    || normal_delta[1].abs() > 0.0001 
                    || normal_delta[2].abs() > 0.0001 {
                    normal_deltas.push((vertex_index, normal_delta));
                }
            }

            morph_targets.push(MorphTarget {
                name,
                mesh_index: 0, // Will be set when linking to mesh
                vertex_deltas,
                normal_deltas,
            });
        }

        Ok(morph_targets)
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
                    if let Ok(node) = self.parse_node_chunk(&mut reader, chunk_header) {
                        model.nodes.push(node);
                    }
                }
                ChunkType::Material => {
                    if let Ok(material) = self.parse_material_chunk(&mut reader, chunk_header) {
                        model.materials.push(material);
                    }
                }
                ChunkType::CompiledBones => {
                    if let Ok(skeleton) = self.parse_compiled_bones_chunk(&mut reader, chunk_header) {
                        model.skeleton = Some(skeleton);
                    }
                }
                ChunkType::CompiledMesh => {
                    if let Ok(mesh) = self.parse_compiled_mesh_chunk(&mut reader, chunk_header) {
                        model.meshes.push(mesh);
                    }
                }
                ChunkType::CompiledMorphTargets => {
                    if let Ok(morph_targets) = self.parse_compiled_morph_targets_chunk(&mut reader, chunk_header) {
                        model.morph_targets.extend(morph_targets);
                    }
                }
                ChunkType::BoneAnim | ChunkType::BoneNameList => {
                    // Legacy bone data - skip for now
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