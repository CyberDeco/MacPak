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

/// BG3 model type flags bitmask (stored in GR2 Flags[0]).
///
/// Fields document the flag layout; individual bits are not currently
/// read back after parsing but are kept for format reference.
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct ModelFlags {
    pub mesh_proxy: bool,
    pub cloth: bool,
    pub has_proxy_geometry: bool,
    pub has_color: bool,
    pub skinned: bool,
    pub rigid: bool,
    pub spring: bool,
    pub occluder: bool,
}

impl ModelFlags {
    pub fn from_u32(v: u32) -> Self {
        Self {
            mesh_proxy: v & 0x01 != 0,
            cloth: v & 0x02 != 0,
            has_proxy_geometry: v & 0x04 != 0,
            has_color: v & 0x08 != 0,
            skinned: v & 0x10 != 0,
            rigid: v & 0x20 != 0,
            spring: v & 0x40 != 0,
            occluder: v & 0x80 != 0,
        }
    }
}

/// BG3 cloth simulation flags bitmask (stored in GR2 Flags[2]).
#[derive(Debug, Clone, Copy, Default)]
pub struct ClothFlags {
    pub cloth_01: bool,
    pub cloth_02: bool,
    pub cloth_04: bool,
    pub cloth_physics: bool,
}

impl ClothFlags {
    pub fn from_u32(v: u32) -> Self {
        Self {
            cloth_01: v & 0x01 != 0,
            cloth_02: v & 0x02 != 0,
            cloth_04: v & 0x04 != 0,
            cloth_physics: v & 0x100 != 0,
        }
    }
}

/// Per-mesh rendering properties (GR2 `UserMeshProperties` pointer target).
#[derive(Debug, Clone, Default)]
pub struct MeshPropertySet {
    pub model_flags: ModelFlags,
    pub cloth_flags: ClothFlags,
    pub lod_distance: f32,
    pub is_impostor: bool,
}

/// Per-mesh extended metadata (GR2 Mesh.ExtendedData pointer target).
#[derive(Debug, Clone, Default)]
pub struct MeshExtendedData {
    pub mesh_proxy: i32,
    pub rigid: i32,
    pub cloth: i32,
    pub spring: i32,
    pub occluder: i32,
    pub lod: i32,
    pub user_defined_properties: Option<String>,
    pub mesh_properties: Option<MeshPropertySet>,
}

/// A bone binding from a GR2 mesh.
#[derive(Debug, Clone)]
pub struct BoneBinding {
    pub bone_name: String,
    pub obb_min: [f32; 3],
    pub obb_max: [f32; 3],
    pub tri_count: i32,
    pub tri_indices: Vec<i32>,
}

/// A topology group from a GR2 mesh.
#[derive(Debug, Clone)]
pub struct TopologyGroup {
    pub material_index: i32,
    pub tri_first: i32,
    pub tri_count: i32,
}

/// A model from a GR2 file.
#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub mesh_binding_names: Vec<String>,
    pub initial_placement: Transform,
}

/// Mesh data containing vertices and indices.
#[derive(Clone)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub is_32bit_indices: bool,
    pub extended_data: Option<MeshExtendedData>,
    pub bone_bindings: Vec<BoneBinding>,
    pub material_binding_names: Vec<String>,
    pub topology_groups: Vec<TopologyGroup>,
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
    pub lod_error: f32,
}

/// A skeleton containing bones.
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
    pub lod_type: i32,
}

/// Information about what data a GR2 file contains.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
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
