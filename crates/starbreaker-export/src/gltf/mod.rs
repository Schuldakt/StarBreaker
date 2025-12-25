//! glTF 2.0 exporter
//!
//! Exports CGF models to glTF 2.0 format (JSON + BIN)

mod exporter;

pub use exporter::{GltfExporter, GltfExportOptions, GltfExportError};

use serde::{Deserialize, Serialize};

/// glTF 2.0 root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gltf {
    pub asset: Asset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub scenes: Vec<Scene>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nodes: Vec<Node>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub meshes: Vec<Mesh>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub materials: Vec<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accessors: Vec<Accessor>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub buffer_views: Vec<BufferView>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub buffers: Vec<Buffer>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skins: Vec<Skin>,
}

/// glTF asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

/// glTF scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub nodes: Vec<usize>,
}

/// glTF node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<usize>,
}

/// glTF mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

/// glTF mesh primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    pub attributes: std::collections::HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
}

/// glTF material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pbrMetallicRoughness")]
    pub pbr_metallic_roughness: Option<PbrMetallicRoughness>,
}

/// PBR metallic roughness material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMetallicRoughness {
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseColorFactor")]
    pub base_color_factor: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "metallicFactor")]
    pub metallic_factor: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "roughnessFactor")]
    pub roughness_factor: Option<f32>,
}

/// glTF accessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accessor {
    #[serde(skip_serializing_if = "Option::is_none", rename = "bufferView")]
    pub buffer_view: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "byteOffset")]
    pub byte_offset: Option<usize>,
    #[serde(rename = "componentType")]
    pub component_type: u32,
    pub count: usize,
    #[serde(rename = "type")]
    pub accessor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f32>>,
}

/// glTF buffer view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferView {
    pub buffer: usize,
    #[serde(skip_serializing_if = "Option::is_none", rename = "byteOffset")]
    pub byte_offset: Option<usize>,
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none", rename = "byteStride")]
    pub byte_stride: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

/// glTF buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buffer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
}

/// glTF skin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "inverseBindMatrices")]
    pub inverse_bind_matrices: usize,
    pub joints: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<usize>,
}

// glTF component type constants
pub const COMPONENT_TYPE_BYTE: u32 = 5120;
pub const COMPONENT_TYPE_UNSIGNED_BYTE: u32 = 5121;
pub const COMPONENT_TYPE_SHORT: u32 = 5122;
pub const COMPONENT_TYPE_UNSIGNED_SHORT: u32 = 5123;
pub const COMPONENT_TYPE_UNSIGNED_INT: u32 = 5125;
pub const COMPONENT_TYPE_FLOAT: u32 = 5126;

// glTF buffer view target constants
pub const TARGET_ARRAY_BUFFER: u32 = 34962;
pub const TARGET_ELEMENT_ARRAY_BUFFER: u32 = 34963;

// glTF primitive mode constants
pub const MODE_TRIANGLES: u32 = 4;
