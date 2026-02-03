//! GR2 file writer.
//!
//! Writes meshes and skeletons to GR2 format compatible with
//! Baldur's Gate 3 and Divinity: Original Sin 2.
//!
//! Note: Compression is currently disabled. Files are written uncompressed.

#![allow(dead_code, clippy::vec_init_then_push)]

mod build_sections;
mod constants;
mod file_bytes;
mod section;
mod types;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::gltf_loader::{MeshData, Skeleton};
use crate::error::Result;

// Section is used internally by build_sections and file_bytes

pub struct Gr2Writer {
    meshes: Vec<MeshData>,
    skeleton: Option<Skeleton>,
}

impl Gr2Writer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            skeleton: None,
        }
    }

    pub fn add_mesh(&mut self, mesh: &MeshData) {
        self.meshes.push(MeshData {
            name: mesh.name.clone(),
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
        });
    }

    pub fn add_skeleton(&mut self, skeleton: &Skeleton) {
        self.skeleton = Some(skeleton.clone());
    }

    /// Build the GR2 file and return as bytes.
    ///
    /// # Errors
    /// Returns an error if building the GR2 data fails.
    pub fn build(&self) -> Result<Vec<u8>> {
        let sections = self.build_sections()?;
        self.build_file_bytes(&sections)
    }

    /// Write the GR2 file to disk.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write(&self, path: &Path) -> Result<()> {
        let data = self.build()?;
        let mut file = File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl Default for Gr2Writer {
    fn default() -> Self {
        Self::new()
    }
}
