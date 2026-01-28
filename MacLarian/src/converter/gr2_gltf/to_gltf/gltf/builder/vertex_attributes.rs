//! Vertex attribute methods for `GltfBuilder`

use super::super::types::{GltfBufferView, GltfAccessor};
use super::GltfBuilder;

impl GltfBuilder {
    pub(super) fn add_positions(&mut self, positions: &[[f32; 3]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for pos in positions {
            for i in 0..3 {
                min[i] = min[i].min(pos[i]);
                max[i] = max[i].max(pos[i]);
            }
            for &v in pos {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: positions.len() * 12,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126, // FLOAT
            count: positions.len(),
            accessor_type: "VEC3".to_string(),
            min: Some(min.to_vec()),
            max: Some(max.to_vec()),
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_normals(&mut self, normals: &[[f32; 3]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for n in normals {
            for &v in n {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: normals.len() * 12,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: normals.len(),
            accessor_type: "VEC3".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_tangents(&mut self, tangents: &[[f32; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for t in tangents {
            for &v in t {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: tangents.len() * 16,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: tangents.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_texcoords(&mut self, uvs: &[[f32; 2]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for uv in uvs {
            for &v in uv {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: uvs.len() * 8,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126,
            count: uvs.len(),
            accessor_type: "VEC2".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_colors(&mut self, colors: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for c in colors {
            self.buffer.extend_from_slice(c);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: colors.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121, // UNSIGNED_BYTE
            count: colors.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: Some(true),
        });

        acc_idx
    }

    pub(super) fn add_joints(&mut self, joints: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for j in joints {
            self.buffer.extend_from_slice(j);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: joints.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121, // UNSIGNED_BYTE
            count: joints.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_weights(&mut self, weights: &[[u8; 4]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for w in weights {
            self.buffer.extend_from_slice(w);
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: weights.len() * 4,
            target: Some(34962),
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5121,
            count: weights.len(),
            accessor_type: "VEC4".to_string(),
            min: None,
            max: None,
            normalized: Some(true),
        });

        acc_idx
    }

    pub(super) fn add_indices(&mut self, indices: &[u32], use_32bit: bool) -> usize {
        let byte_offset;
        let byte_length;
        let component_type;

        if use_32bit || indices.iter().any(|&i| i > 65535) {
            self.align(4);
            byte_offset = self.buffer.len();
            for &idx in indices {
                self.buffer.extend_from_slice(&idx.to_le_bytes());
            }
            byte_length = indices.len() * 4;
            component_type = 5125; // UNSIGNED_INT
        } else {
            self.align(2);
            byte_offset = self.buffer.len();
            for &idx in indices {
                self.buffer.extend_from_slice(&(idx as u16).to_le_bytes());
            }
            byte_length = indices.len() * 2;
            component_type = 5123; // UNSIGNED_SHORT
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length,
            target: Some(34963), // ELEMENT_ARRAY_BUFFER
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type,
            count: indices.len(),
            accessor_type: "SCALAR".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }

    pub(super) fn add_inverse_bind_matrices(&mut self, matrices: &[[f32; 16]]) -> usize {
        self.align(4);
        let byte_offset = self.buffer.len();

        for mat in matrices {
            for &v in mat {
                self.buffer.extend_from_slice(&v.to_le_bytes());
            }
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: matrices.len() * 64,
            target: None,
        });

        let acc_idx = self.accessors.len();
        self.accessors.push(GltfAccessor {
            buffer_view: bv_idx,
            component_type: 5126, // FLOAT
            count: matrices.len(),
            accessor_type: "MAT4".to_string(),
            min: None,
            max: None,
            normalized: None,
        });

        acc_idx
    }
}
