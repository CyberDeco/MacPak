//! Image, texture, and material methods for `GltfBuilder`

use super::super::materials::{
    GltfImage, GltfMaterial, GltfNormalTextureInfo, GltfOcclusionTextureInfo,
    GltfPbrMetallicRoughness, GltfSampler, GltfTexture, GltfTextureInfo,
};
use super::super::types::GltfBufferView;
use super::GltfBuilder;

impl GltfBuilder {
    /// Add an embedded image (PNG bytes) to the GLB.
    /// Returns the image index.
    pub fn add_embedded_image(&mut self, png_data: &[u8], name: Option<String>) -> usize {
        // Align to 4 for consistency (not necessary but it's whatev)
        self.align(4);
        let byte_offset = self.buffer.len();
        self.buffer.extend_from_slice(png_data);

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(GltfBufferView {
            buffer: 0,
            byte_offset,
            byte_length: png_data.len(),
            target: None, // No target for images
        });

        let img_idx = self.images.len();
        self.images.push(GltfImage {
            buffer_view: bv_idx,
            mime_type: "image/png".to_string(),
            name,
        });

        img_idx
    }

    /// Add a texture sampler with default settings (linear filtering, repeat wrap).
    /// Returns the sampler index.
    pub fn add_sampler(&mut self) -> usize {
        let sampler_idx = self.samplers.len();
        self.samplers.push(GltfSampler::default());
        sampler_idx
    }

    /// Add a custom texture sampler.
    /// Returns the sampler index.
    pub fn add_sampler_custom(&mut self, sampler: GltfSampler) -> usize {
        let sampler_idx = self.samplers.len();
        self.samplers.push(sampler);
        sampler_idx
    }

    /// Add a texture referencing an image and optionally a sampler.
    /// Returns the texture index.
    pub fn add_texture(
        &mut self,
        image_index: usize,
        sampler_index: Option<usize>,
        name: Option<String>,
    ) -> usize {
        let tex_idx = self.textures.len();
        self.textures.push(GltfTexture {
            source: image_index,
            sampler: sampler_index,
            name,
        });
        tex_idx
    }

    /// Add a PBR material with optional textures.
    ///
    /// # Arguments
    /// * `name` - Optional material name
    /// * `base_color_texture` - Albedo/diffuse texture index
    /// * `normal_texture` - Normal map texture index
    /// * `metallic_roughness_texture` - Combined metallic-roughness texture index
    /// * `occlusion_texture` - Ambient occlusion texture index
    ///
    /// Returns the material index.
    pub fn add_material(
        &mut self,
        name: Option<String>,
        base_color_texture: Option<usize>,
        normal_texture: Option<usize>,
        metallic_roughness_texture: Option<usize>,
        occlusion_texture: Option<usize>,
    ) -> usize {
        let pbr = GltfPbrMetallicRoughness {
            base_color_factor: Some([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: base_color_texture.map(|idx| GltfTextureInfo {
                index: idx,
                tex_coord: None,
            }),
            metallic_factor: Some(1.0),
            roughness_factor: Some(1.0),
            metallic_roughness_texture: metallic_roughness_texture.map(|idx| GltfTextureInfo {
                index: idx,
                tex_coord: None,
            }),
        };

        let mat_idx = self.materials.len();
        self.materials.push(GltfMaterial {
            name,
            pbr_metallic_roughness: Some(pbr),
            normal_texture: normal_texture.map(|idx| GltfNormalTextureInfo {
                index: idx,
                scale: Some(1.0),
                tex_coord: None,
            }),
            occlusion_texture: occlusion_texture.map(|idx| GltfOcclusionTextureInfo {
                index: idx,
                strength: Some(1.0),
                tex_coord: None,
            }),
            emissive_texture: None,
            emissive_factor: None,
            alpha_mode: None,
            alpha_cutoff: None,
            double_sided: None,
        });

        mat_idx
    }

    /// Add a simple material with just a base color texture.
    /// Returns the material index.
    pub fn add_simple_material(
        &mut self,
        name: Option<String>,
        base_color_texture: usize,
    ) -> usize {
        self.add_material(name, Some(base_color_texture), None, None, None)
    }

    /// Convenience method to add an image, create a texture for it, and return the texture index.
    /// Also creates a default sampler if none exists.
    pub fn add_image_as_texture(&mut self, png_data: &[u8], name: Option<String>) -> usize {
        // Ensure there's at least one sampler
        let sampler_idx = if self.samplers.is_empty() {
            Some(self.add_sampler())
        } else {
            Some(0)
        };

        let image_idx = self.add_embedded_image(png_data, name.clone());
        self.add_texture(image_idx, sampler_idx, name)
    }
}
