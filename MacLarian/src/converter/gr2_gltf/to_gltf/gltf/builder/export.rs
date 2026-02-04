//! Export methods for `GltfBuilder`

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::{Error, Result};

use super::super::types::{GltfAsset, GltfBuffer, GltfDocument, GltfScene};
use super::GltfBuilder;

impl GltfBuilder {
    pub(super) fn build_document(
        self,
        root_bone_idx: Option<usize>,
        buffer_uri: Option<String>,
    ) -> (GltfDocument, Vec<u8>) {
        let mut scene_nodes = Vec::new();

        if let Some(root_idx) = root_bone_idx {
            scene_nodes.push(root_idx);
        }

        for (i, node) in self.nodes.iter().enumerate() {
            if node.mesh.is_some() {
                scene_nodes.push(i);
            }
        }

        let has_profiles = self.meshes.iter().any(|m| m.extensions.is_some())
            || self.skins.iter().any(|s| s.extensions.is_some());

        let doc = GltfDocument {
            asset: GltfAsset {
                version: "2.0".to_string(),
                generator: Some("MacLarian GR2 to glTF converter".to_string()),
            },
            scene: 0,
            scenes: vec![GltfScene {
                name: Some("Scene".to_string()),
                nodes: scene_nodes,
            }],
            nodes: self.nodes,
            meshes: self.meshes,
            skins: self.skins,
            materials: self.materials,
            textures: self.textures,
            images: self.images,
            samplers: self.samplers,
            accessors: self.accessors,
            buffer_views: self.buffer_views,
            buffers: vec![GltfBuffer {
                byte_length: self.buffer.len(),
                uri: buffer_uri,
            }],
            extensions_used: if has_profiles {
                vec!["MACLARIAN_glTF_extensions".into()]
            } else {
                vec![]
            },
        };

        (doc, self.buffer)
    }

    /// Build GLB data and return as bytes.
    ///
    /// # Errors
    /// Returns an error if JSON serialization fails.
    pub fn build_glb(self, root_bone_idx: Option<usize>) -> Result<Vec<u8>> {
        let (doc, buffer) = self.build_document(root_bone_idx, None);
        let json = serde_json::to_string(&doc)
            .map_err(|e| Error::ConversionError(format!("JSON serialization error: {e}")))?;
        let json_bytes = json.as_bytes();

        let json_padding = (4 - (json_bytes.len() % 4)) % 4;
        let json_chunk_len = json_bytes.len() + json_padding;

        let bin_padding = (4 - (buffer.len() % 4)) % 4;
        let bin_chunk_len = buffer.len() + bin_padding;

        let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

        let mut output = Vec::with_capacity(total_len);

        // GLB header
        output.extend_from_slice(b"glTF");
        output.extend_from_slice(&2u32.to_le_bytes());
        output.extend_from_slice(&(total_len as u32).to_le_bytes());

        // JSON chunk
        output.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
        output.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
        output.extend_from_slice(json_bytes);
        for _ in 0..json_padding {
            output.push(b' ');
        }

        // Binary chunk
        output.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
        output.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
        output.extend_from_slice(&buffer);
        for _ in 0..bin_padding {
            output.push(0u8);
        }

        Ok(output)
    }

    /// Export as a GLB file.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn export_glb(self, path: &Path, root_bone_idx: Option<usize>) -> Result<()> {
        let glb_data = self.build_glb(root_bone_idx)?;
        let mut file = File::create(path)?;
        file.write_all(&glb_data)?;
        Ok(())
    }

    /// Export as separate .gltf (JSON) and .bin (binary buffer) files.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn export_gltf(self, path: &Path, root_bone_idx: Option<usize>) -> Result<()> {
        // Determine the .bin file name (same base name, .bin extension)
        let bin_filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("{s}.bin"))
            .ok_or_else(|| Error::ConversionError("Invalid output path".to_string()))?;

        let bin_path = path.with_file_name(&bin_filename);

        // Build document with URI pointing to the .bin file
        let (doc, buffer) = self.build_document(root_bone_idx, Some(bin_filename));

        // Write JSON to .gltf file
        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| Error::ConversionError(format!("JSON serialization error: {e}")))?;
        let mut gltf_file = File::create(path)?;
        gltf_file.write_all(json.as_bytes())?;

        // Write binary buffer to .bin file
        let mut bin_file = File::create(&bin_path)?;
        bin_file.write_all(&buffer)?;

        Ok(())
    }
}
