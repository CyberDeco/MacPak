//! PAK archive operations

use crate::error::{Error, Result};
use larian_formats::lspk::{DecompressedLspk, Reader, Writer};
use larian_formats::format::Write as LarianWrite;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct PakOperations;

impl PakOperations {
    /// Extract a PAK file to a directory
    pub fn extract<P: AsRef<Path>>(pak_path: P, output_dir: P) -> Result<()> {
        let file = File::open(pak_path.as_ref())?;
        let reader = BufReader::new(file);
        
        // Use the Reader API from larian-formats
        let mut pak_reader = Reader::new(reader)
            .map_err(|e| Error::ConversionError(format!("Failed to open PAK: {:?}", e)))?;
        
        let decompressed = pak_reader.read()
            .map_err(|e| Error::ConversionError(format!("Failed to read PAK: {:?}", e)))?;
        
        std::fs::create_dir_all(&output_dir)?;
        
        for file in decompressed.files {
            // Skip .DS_Store files
            if file.path.file_name() == Some(std::ffi::OsStr::new(".DS_Store")) {
                tracing::debug!("Skipping .DS_Store file: {:?}", file.path);
                continue;
            }
            
            let output_path = output_dir.as_ref().join(&file.path);
            
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            std::fs::write(&output_path, &file.decompressed_bytes)?;
        }
        
        Ok(())
    }
    
    /// Create a PAK file from a directory
    pub fn create<P: AsRef<Path>>(source_dir: P, output_pak: P) -> Result<()> {
        // Remove .DS_Store files before packing
        Self::remove_ds_store_files(source_dir.as_ref())?;
        
        let writer = Writer::with_mod_root_path(source_dir.as_ref())?;
        writer.write(output_pak.as_ref().to_path_buf())?;
        
        Ok(())
    }
    
    /// Remove .DS_Store files from a directory tree
    fn remove_ds_store_files(dir: &Path) -> Result<()> {
        use walkdir::WalkDir;
        
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_name() == ".DS_Store" {
                std::fs::remove_file(entry.path())?;
                tracing::debug!("Removed .DS_Store file before packing: {:?}", entry.path());
            }
        }
        
        Ok(())
    }
    
    /// List contents of a PAK file
    pub fn list<P: AsRef<Path>>(pak_path: P) -> Result<Vec<String>> {
        let file = File::open(pak_path)?;
        let reader = BufReader::new(file);
        
        let mut pak_reader = Reader::new(reader)
            .map_err(|e| Error::ConversionError(format!("Failed to open PAK: {:?}", e)))?;
        
        let decompressed = pak_reader.read()
            .map_err(|e| Error::ConversionError(format!("Failed to read PAK: {:?}", e)))?;
        
        Ok(decompressed.files.iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect())
    }
    
    /// Extract meta.lsx from a PAK
    pub fn extract_meta<P: AsRef<Path>>(pak_path: P) -> Result<String> {
        let file = File::open(pak_path)?;
        let reader = BufReader::new(file);
        
        let mut pak_reader = Reader::new(reader)
            .map_err(|e| Error::ConversionError(format!("Failed to open PAK: {:?}", e)))?;
        
        let decompressed = pak_reader.read()
            .map_err(|e| Error::ConversionError(format!("Failed to read PAK: {:?}", e)))?;
        
        let meta_file = decompressed.extract_meta_lsx()
            .map_err(|e| Error::FileNotFoundInPak(format!("meta.lsx: {:?}", e)))?;
        
        String::from_utf8(meta_file.decompressed_bytes)
            .map_err(|e| Error::ConversionError(format!("Invalid UTF-8 in meta.lsx: {}", e)))
    }
}