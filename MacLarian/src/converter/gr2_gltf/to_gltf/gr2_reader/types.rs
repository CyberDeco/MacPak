//! Public data structures for GR2 parsing.

/// A parsed vertex with all attributes.
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub bone_weights: [u8; 4],
    pub bone_indices: [u8; 4],
    pub qtangent: [i16; 4],
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

/// Mesh data containing vertices and indices.
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub is_32bit_indices: bool,
}

/// Transform data (translation, rotation, scale/shear).
#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale_shear: [f32; 9],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale_shear: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// A bone in a skeleton.
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent_index: i32,
    pub transform: Transform,
    pub inverse_world_transform: [f32; 16],
}

/// A skeleton containing bones.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
}

/// Information about what data a GR2 file contains.
#[derive(Debug, Clone)]
pub struct Gr2ContentInfo {
    pub material_count: usize,
    pub skeleton_count: usize,
    pub mesh_count: usize,
    pub model_count: usize,
}

impl Gr2ContentInfo {
    /// Returns a human-readable description of the file contents.
    #[must_use]
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if self.skeleton_count > 0 {
            parts.push(format!("{} skeleton(s)", self.skeleton_count));
        }
        if self.mesh_count > 0 {
            parts.push(format!("{} mesh(es)", self.mesh_count));
        }
        if self.model_count > 0 {
            parts.push(format!("{} model(s)", self.model_count));
        }
        if self.material_count > 0 {
            parts.push(format!("{} material(s)", self.material_count));
        }
        if parts.is_empty() {
            "empty".to_string()
        } else {
            parts.join(", ")
        }
    }
}
