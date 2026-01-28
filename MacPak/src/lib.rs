#![allow(non_snake_case)]
//! MacPak - High-level BG3 modding toolkit
use std::path::Path;

// Re-export maclarian
pub use maclarian;

pub mod error;
pub mod index;
pub mod operations;
pub mod workspace;

// GUI-specific modules (moved from MacLarian)
pub mod dialog;
pub mod dyes;
pub mod formats;

// Feature-gated modules
#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "viewer")]
pub mod viewer;

pub use error::{Error, Result};

/// Main toolkit interface
pub struct Toolkit {
    workspace: workspace::Workspace,
    #[allow(dead_code)] // Future: file indexing functionality
    index: index::FileIndex,
}

impl Toolkit {
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: workspace::Workspace::new()?,
            index: index::FileIndex::new()?,
        })
    }

    /// Open or create a workspace
    pub fn open_workspace(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.workspace.open(path)?;
        Ok(())
    }

    // High-level operations
    pub fn extract_pak(&self, pak: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
        operations::extraction::extract_pak(pak, dest)
    }

    pub fn convert_lsf_to_lsx(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::lsf_to_lsx(source, dest)
    }

    pub fn convert_loca_to_xml(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::loca_to_xml(source, dest)
    }

    pub fn convert_xml_to_loca(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::xml_to_loca(source, dest)
    }

    // Virtual texture operations

    /// List information about a GTS file
    pub fn list_gts(
        &self,
        gts_path: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::GtsInfo> {
        operations::virtual_texture::list_gts(gts_path)
    }

    /// Get information about a GTP file
    pub fn gtp_info(
        &self,
        gtp_path: impl AsRef<Path>,
        gts_path: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::GtpInfo> {
        operations::virtual_texture::gtp_info(gtp_path, gts_path)
    }

    /// Extract a single GTP file to DDS textures
    pub fn extract_gtp(
        &self,
        gtp_path: impl AsRef<Path>,
        gts_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<()> {
        operations::virtual_texture::extract_gtp(gtp_path, gts_path, output_dir)
    }

    /// Extract all GTP files referenced by a GTS file
    pub fn extract_virtual_textures(
        &self,
        gts_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::ExtractResult> {
        operations::virtual_texture::extract_all(gts_path, output_dir)
    }
}
