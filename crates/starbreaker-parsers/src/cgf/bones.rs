// starbreaker-parsers/src/cgf/bones.rs
//! CGF skeleton and bone structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skeleton structure for skinned meshes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    /// All bones in the skeleton
    pub bones: Vec<Bone>,
    /// Bone name to index mapping
    pub bone_map: HashMap<String, usize>,
    /// Root bone indices
    pub root_bones: Vec<usize>,
}

impl Skeleton {
    /// Create a new empty skeleton
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            bone_map: HashMap::new(),
            root_bones: Vec::new(),
        }
    }

    /// Add a bone to the skeleton
    pub fn add_bone(&mut self, bone: Bone) -> usize {
        let idx = self.bones.len();
        self.bone_map.insert(bone.name.clone(), idx);
        
        if bone.parent_index.is_none() {
            self.root_bones.push(idx);
        }
        
        self.bones.push(bone);
        idx
    }

    /// Get bone count
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Find bone by name
    pub fn find_bone(&self, name: &str) -> Option<&Bone> {
        self.bone_map.get(name).map(|&idx| &self.bones[idx])
    }

    /// Find bone index by name
    pub fn find_bone_index(&self, name: &str) -> Option<usize> {
        self.bone_map.get(name).copied()
    }

    /// Get bone by index
    pub fn get_bone(&self, index: usize) -> Option<&Bone> {
        self.bones.get(index)
    }

    /// Get children of a bone
    pub fn children(&self, bone_index: usize) -> Vec<usize> {
        self.bones.iter()
            .enumerate()
            .filter(|(_, b)| b.parent_index == Some(bone_index))
            .map(|(i, _)| i)
            .collect()
    }

    /// Build hierarchy from parent indices
    pub fn build_hierarchy(&mut self) {
        self.root_bones.clear();
        
        for (idx, bone) in self.bones.iter().enumerate() {
            if bone.parent_index.is_none() {
                self.root_bones.push(idx);
            }
        }
    }

    /// Get bone chain from a bone to root
    pub fn bone_chain_to_root(&self, bone_index: usize) -> Vec<usize> {
        let mut chain = vec![bone_index];
        let mut current = bone_index;
        
        while let Some(parent) = self.bones.get(current).and_then(|b| b.parent_index) {
            chain.push(parent);
            current = parent;
        }
        
        chain
    }

    /// Calculate world transform for a bone
    pub fn world_transform(&self, bone_index: usize) -> [[f32; 4]; 4] {
        let chain = self.bone_chain_to_root(bone_index);
        
        let mut result = IDENTITY_MATRIX;
        
        // Multiply from root to bone
        for &idx in chain.iter().rev() {
            if let Some(bone) = self.bones.get(idx) {
                result = multiply_matrices(result, bone.local_transform);
            }
        }
        
        result
    }

    /// Get all bone names
    pub fn bone_names(&self) -> Vec<&str> {
        self.bones.iter().map(|b| b.name.as_str()).collect()
    }

    /// Validate skeleton structure
    pub fn validate(&self) -> Result<(), String> {
        for (idx, bone) in self.bones.iter().enumerate() {
            if let Some(parent) = bone.parent_index {
                if parent >= self.bones.len() {
                    return Err(format!(
                        "Bone {} has invalid parent index {}",
                        bone.name, parent
                    ));
                }
                if parent == idx {
                    return Err(format!(
                        "Bone {} references itself as parent",
                        bone.name
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

/// A single bone in the skeleton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    /// Bone name
    pub name: String,
    /// Parent bone index (None for root bones)
    pub parent_index: Option<usize>,
    /// Controller ID (for animation)
    pub controller_id: u32,
    /// Local transform matrix (relative to parent)
    pub local_transform: [[f32; 4]; 4],
    /// Bind pose matrix (world space)
    pub bind_pose: [[f32; 4]; 4],
    /// Inverse bind pose (for skinning)
    pub inverse_bind_pose: [[f32; 4]; 4],
    /// Physics properties
    pub physics: Option<BonePhysics>,
    /// Bone limits (for IK)
    pub limits: Option<BoneLimits>,
}

impl Bone {
    /// Create a new bone
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_index: None,
            controller_id: 0,
            local_transform: IDENTITY_MATRIX,
            bind_pose: IDENTITY_MATRIX,
            inverse_bind_pose: IDENTITY_MATRIX,
            physics: None,
            limits: None,
        }
    }

    /// Get the bone position from transform
    pub fn position(&self) -> [f32; 3] {
        [
            self.local_transform[3][0],
            self.local_transform[3][1],
            self.local_transform[3][2],
        ]
    }

    /// Check if this is a root bone
    pub fn is_root(&self) -> bool {
        self.parent_index.is_none()
    }

    /// Set position in transform
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.local_transform[3][0] = position[0];
        self.local_transform[3][1] = position[1];
        self.local_transform[3][2] = position[2];
    }

    /// Calculate inverse bind pose from bind pose
    pub fn calculate_inverse_bind_pose(&mut self) {
        self.inverse_bind_pose = invert_matrix(self.bind_pose);
    }
}

/// Physics properties for a bone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonePhysics {
    /// Physics type
    pub physics_type: PhysicsType,
    /// Mass
    pub mass: f32,
    /// Damping
    pub damping: f32,
    /// Stiffness
    pub stiffness: f32,
    /// Collision proxy geometry index
    pub proxy_index: Option<u32>,
}

/// Physics type for bones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsType {
    /// No physics
    None,
    /// Rigid body
    Rigid,
    /// Ragdoll/physical
    Ragdoll,
    /// Rope/chain
    Rope,
    /// Cloth
    Cloth,
}

/// Bone rotation limits for IK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneLimits {
    /// Minimum rotation (euler angles in radians)
    pub min_rotation: [f32; 3],
    /// Maximum rotation (euler angles in radians)
    pub max_rotation: [f32; 3],
    /// Spring tension
    pub spring_tension: f32,
    /// Spring angle
    pub spring_angle: f32,
}

impl Default for BoneLimits {
    fn default() -> Self {
        Self {
            min_rotation: [-std::f32::consts::PI; 3],
            max_rotation: [std::f32::consts::PI; 3],
            spring_tension: 0.0,
            spring_angle: 0.0,
        }
    }
}

// Matrix utilities

/// Identity matrix
const IDENTITY_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Multiply two 4x4 matrices
fn multiply_matrices(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0f32; 4]; 4];
    
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = 
                a[i][0] * b[0][j] +
                a[i][1] * b[1][j] +
                a[i][2] * b[2][j] +
                a[i][3] * b[3][j];
        }
    }
    
    result
}

/// Invert a 4x4 matrix (assuming it's a valid transform matrix)
fn invert_matrix(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    // For transform matrices, we can use a simplified approach:
    // The upper 3x3 is rotation (orthogonal), the last row is [0,0,0,1]
    // and translation is in column 3
    
    // Transpose the 3x3 rotation part
    let mut result = IDENTITY_MATRIX;
    
    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = m[j][i];
        }
    }
    
    // Negate and transform the translation
    let tx = m[3][0];
    let ty = m[3][1];
    let tz = m[3][2];
    
    result[3][0] = -(result[0][0] * tx + result[1][0] * ty + result[2][0] * tz);
    result[3][1] = -(result[0][1] * tx + result[1][1] * ty + result[2][1] * tz);
    result[3][2] = -(result[0][2] * tx + result[1][2] * ty + result[2][2] * tz);
    
    result
}

/// Convert quaternion to rotation matrix
pub fn quaternion_to_matrix(q: [f32; 4]) -> [[f32; 4]; 4] {
    let [x, y, z, w] = q;
    
    let xx = x * x;
    let xy = x * y;
    let xz = x * z;
    let xw = x * w;
    let yy = y * y;
    let yz = y * z;
    let yw = y * w;
    let zz = z * z;
    let zw = z * w;
    
    [
        [1.0 - 2.0 * (yy + zz), 2.0 * (xy - zw), 2.0 * (xz + yw), 0.0],
        [2.0 * (xy + zw), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - xw), 0.0],
        [2.0 * (xz - yw), 2.0 * (yz + xw), 1.0 - 2.0 * (xx + yy), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Convert rotation matrix to quaternion
pub fn matrix_to_quaternion(m: [[f32; 4]; 4]) -> [f32; 4] {
    let trace = m[0][0] + m[1][1] + m[2][2];
    
    if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        [
            (m[2][1] - m[1][2]) * s,
            (m[0][2] - m[2][0]) * s,
            (m[1][0] - m[0][1]) * s,
            0.25 / s,
        ]
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = 2.0 * (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt();
        [
            0.25 * s,
            (m[0][1] + m[1][0]) / s,
            (m[0][2] + m[2][0]) / s,
            (m[2][1] - m[1][2]) / s,
        ]
    } else if m[1][1] > m[2][2] {
        let s = 2.0 * (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt();
        [
            (m[0][1] + m[1][0]) / s,
            0.25 * s,
            (m[1][2] + m[2][1]) / s,
            (m[0][2] - m[2][0]) / s,
        ]
    } else {
        let s = 2.0 * (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt();
        [
            (m[0][2] + m[2][0]) / s,
            (m[1][2] + m[2][1]) / s,
            0.25 * s,
            (m[1][0] - m[0][1]) / s,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_add_bone() {
        let mut skeleton = Skeleton::new();
        
        let root = Bone::new("root");
        let root_idx = skeleton.add_bone(root);
        
        let mut child = Bone::new("child");
        child.parent_index = Some(root_idx);
        let child_idx = skeleton.add_bone(child);
        
        assert_eq!(skeleton.bone_count(), 2);
        assert_eq!(skeleton.root_bones, vec![0]);
        assert_eq!(skeleton.children(root_idx), vec![child_idx]);
    }

    #[test]
    fn test_skeleton_find_bone() {
        let mut skeleton = Skeleton::new();
        skeleton.add_bone(Bone::new("test_bone"));
        
        assert!(skeleton.find_bone("test_bone").is_some());
        assert!(skeleton.find_bone("nonexistent").is_none());
    }

    #[test]
    fn test_bone_chain_to_root() {
        let mut skeleton = Skeleton::new();
        
        let root_idx = skeleton.add_bone(Bone::new("root"));
        
        let mut child = Bone::new("child");
        child.parent_index = Some(root_idx);
        let child_idx = skeleton.add_bone(child);
        
        let mut grandchild = Bone::new("grandchild");
        grandchild.parent_index = Some(child_idx);
        let grandchild_idx = skeleton.add_bone(grandchild);
        
        let chain = skeleton.bone_chain_to_root(grandchild_idx);
        assert_eq!(chain, vec![grandchild_idx, child_idx, root_idx]);
    }

    #[test]
    fn test_identity_matrix_multiply() {
        let m = IDENTITY_MATRIX;
        let result = multiply_matrices(m, m);
        
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((result[i][j] - expected).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_quaternion_to_matrix_identity() {
        let q = [0.0, 0.0, 0.0, 1.0]; // Identity quaternion
        let m = quaternion_to_matrix(q);
        
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m[i][j] - expected).abs() < 0.001);
            }
        }
    }
}