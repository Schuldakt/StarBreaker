// starbreaker-parsers/src/cgf/chunks.rs
//! CGF chunk types and headers

use serde::{Deserialize, Serialize};

/// Chunk types found in CGF files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkType {
    /// Source information
    SourceInfo,
    /// Timing information
    Timing,
    /// MTL Name chunk (materials)
    MtlName,
    /// Export flags
    ExportFlags,
    /// Mesh data
    Mesh,
    /// Mesh subsets
    MeshSubsets,
    /// Node data
    Node,
    /// Material chunk
    Material,
    /// Bone animation data
    BoneAnim,
    /// Bone name list
    BoneNameList,
    /// Bone initial positions
    BoneInitialPos,
    /// Bone mesh (physics)
    BoneMesh,
    /// Helper (dummy/locator)
    Helper,
    /// Morph targets
    MorphTargets,
    /// Controller data (animation)
    Controller,
    /// Compiled bones
    CompiledBones,
    /// Compiled physical bones
    CompiledPhysicalBones,
    /// Compiled morph targets
    CompiledMorphTargets,
    /// Compiled mesh
    CompiledMesh,
    /// Compiled physics geometry
    CompiledPhysicsGeometry,
    /// Compiled int skinning data
    CompiledIntSkinVertices,
    /// Compiled ext to int map
    CompiledExtToIntMap,
    /// Data stream
    DataStream,
    /// Breakable physics
    BreakablePhysics,
    /// Face map
    FaceMap,
    /// Vertex animation
    VertAnim,
    /// Scene properties
    SceneProps,
    /// Footplant info
    FootPlantInfo,
    /// Bone mesh (unknown variant)
    BoneMeshUnknown,
    /// Unknown chunk type
    Unknown(u32),
}

impl ChunkType {
    /// Convert from raw u32 chunk type ID
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x0000 => ChunkType::SourceInfo,
            0x0001 => ChunkType::Timing,
            0x0002 | 0x0014 => ChunkType::MtlName,
            0x0003 => ChunkType::ExportFlags,
            0x1000 => ChunkType::Mesh,
            0x1001 => ChunkType::MeshSubsets,
            0x100B => ChunkType::Node,
            0x100C => ChunkType::Material,
            0x1016 => ChunkType::BoneAnim,
            0x1017 => ChunkType::BoneNameList,
            0x1018 => ChunkType::BoneInitialPos,
            0x1019 => ChunkType::BoneMesh,
            0x101A => ChunkType::Helper,
            0x101B => ChunkType::MorphTargets,
            0x101C => ChunkType::Controller,
            0x1021 | 0xACDC0000 => ChunkType::CompiledBones,
            0x1022 | 0xACDC0001 => ChunkType::CompiledPhysicalBones,
            0x1023 | 0xACDC0002 => ChunkType::CompiledMorphTargets,
            0x1024 | 0xCCCC0000 => ChunkType::CompiledMesh,
            0x1025 => ChunkType::CompiledPhysicsGeometry,
            0x1026 => ChunkType::CompiledIntSkinVertices,
            0x1027 => ChunkType::CompiledExtToIntMap,
            0x1028 => ChunkType::DataStream,
            0x1029 => ChunkType::BreakablePhysics,
            0x102A => ChunkType::FaceMap,
            0x102B => ChunkType::VertAnim,
            0x102C => ChunkType::SceneProps,
            0x102D => ChunkType::FootPlantInfo,
            0x102E => ChunkType::BoneMeshUnknown,
            other => ChunkType::Unknown(other),
        }
    }

    /// Convert to raw u32 chunk type ID
    pub fn to_u32(&self) -> u32 {
        match self {
            ChunkType::SourceInfo => 0x0000,
            ChunkType::Timing => 0x0001,
            ChunkType::MtlName => 0x0014,
            ChunkType::ExportFlags => 0x0003,
            ChunkType::Mesh => 0x1000,
            ChunkType::MeshSubsets => 0x1001,
            ChunkType::Node => 0x100B,
            ChunkType::Material => 0x100C,
            ChunkType::BoneAnim => 0x1016,
            ChunkType::BoneNameList => 0x1017,
            ChunkType::BoneInitialPos => 0x1018,
            ChunkType::BoneMesh => 0x1019,
            ChunkType::Helper => 0x101A,
            ChunkType::MorphTargets => 0x101B,
            ChunkType::Controller => 0x101C,
            ChunkType::CompiledBones => 0xACDC0000,
            ChunkType::CompiledPhysicalBones => 0xACDC0001,
            ChunkType::CompiledMorphTargets => 0xACDC0002,
            ChunkType::CompiledMesh => 0xCCCC0000,
            ChunkType::CompiledPhysicsGeometry => 0x1025,
            ChunkType::CompiledIntSkinVertices => 0x1026,
            ChunkType::CompiledExtToIntMap => 0x1027,
            ChunkType::DataStream => 0x1028,
            ChunkType::BreakablePhysics => 0x1029,
            ChunkType::FaceMap => 0x102A,
            ChunkType::VertAnim => 0x102B,
            ChunkType::SceneProps => 0x102C,
            ChunkType::FootPlantInfo => 0x102D,
            ChunkType::BoneMeshUnknown => 0x102E,
            ChunkType::Unknown(v) => *v,
        }
    }

    /// Check if this chunk contains mesh geometry
    pub fn is_mesh(&self) -> bool {
        matches!(self, ChunkType::Mesh | ChunkType::MeshSubsets | ChunkType::CompiledMesh)
    }

    /// Check if this chunk contains bone/animation data
    pub fn is_bone(&self) -> bool {
        matches!(self, 
            ChunkType::BoneAnim | ChunkType::BoneNameList | ChunkType::BoneInitialPos |
            ChunkType::BoneMesh | ChunkType::CompiledBones | ChunkType::CompiledPhysicalBones
        )
    }
}

/// Chunk header
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// Chunk type
    pub chunk_type: ChunkType,
    /// Chunk version
    pub version: u32,
    /// Offset to chunk data
    pub offset: u32,
    /// Chunk ID
    pub id: u32,
    /// Chunk size (for Ivo format)
    pub size: u32,
}

/// Parsed chunk with data
#[derive(Debug)]
pub enum CgfChunk {
    /// Source information
    SourceInfo {
        source_file: String,
        date: String,
        author: String,
    },
    /// Material name reference
    MtlName {
        name: String,
        material_type: u32,
        physics_type: u32,
    },
    /// Mesh data (see mesh.rs)
    Mesh(super::Mesh),
    /// Node data
    Node(super::Node),
    /// Bone animation
    BoneAnim {
        num_keys: u32,
        keys: Vec<BoneKey>,
    },
    /// Bone name list
    BoneNames(Vec<String>),
    /// Controller data
    Controller {
        controller_type: u32,
        data: Vec<u8>,
    },
    /// Unknown chunk (preserved as raw data)
    Unknown {
        chunk_type: u32,
        version: u32,
        data: Vec<u8>,
    },
}

/// Animation key for bones
#[derive(Debug, Clone)]
pub struct BoneKey {
    /// Time in ticks
    pub time: f32,
    /// Position (for position keys)
    pub position: Option<[f32; 3]>,
    /// Rotation (for rotation keys)
    pub rotation: Option<[f32; 4]>,
    /// Scale (for scale keys)
    pub scale: Option<[f32; 3]>,
}

/// Data stream types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DataStreamType {
    Positions = 0,
    Normals = 1,
    UVs = 2,
    Colors = 3,
    Colors2 = 4,
    Indices = 5,
    Tangents = 6,
    ShCoeffs = 7,
    ShapeDeformation = 8,
    BoneMap = 9,
    FaceArea = 10,
    Qtangents = 11,
    SkinData = 12,
    VertsUV = 13,
    PS3EdgeData = 14,
    ExtraBonesMapping = 15,
    P3S_C = 16,
    Unknown(u32),
}

impl From<u32> for DataStreamType {
    fn from(value: u32) -> Self {
        match value {
            0 => DataStreamType::Positions,
            1 => DataStreamType::Normals,
            2 => DataStreamType::UVs,
            3 => DataStreamType::Colors,
            4 => DataStreamType::Colors2,
            5 => DataStreamType::Indices,
            6 => DataStreamType::Tangents,
            7 => DataStreamType::ShCoeffs,
            8 => DataStreamType::ShapeDeformation,
            9 => DataStreamType::BoneMap,
            10 => DataStreamType::FaceArea,
            11 => DataStreamType::Qtangents,
            12 => DataStreamType::SkinData,
            13 => DataStreamType::VertsUV,
            14 => DataStreamType::PS3EdgeData,
            15 => DataStreamType::ExtraBonesMapping,
            16 => DataStreamType::P3S_C,
            other => DataStreamType::Unknown(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_type_roundtrip() {
        let types = [
            ChunkType::Mesh,
            ChunkType::Node,
            ChunkType::CompiledBones,
            ChunkType::Unknown(0xDEAD),
        ];

        for ct in types {
            let value = ct.to_u32();
            let restored = ChunkType::from_u32(value);
            assert_eq!(ct, restored);
        }
    }

    #[test]
    fn test_chunk_type_is_mesh() {
        assert!(ChunkType::Mesh.is_mesh());
        assert!(ChunkType::CompiledMesh.is_mesh());
        assert!(!ChunkType::Node.is_mesh());
    }

    #[test]
    fn test_chunk_type_is_bone() {
        assert!(ChunkType::BoneAnim.is_bone());
        assert!(ChunkType::CompiledBones.is_bone());
        assert!(!ChunkType::Mesh.is_bone());
    }
}