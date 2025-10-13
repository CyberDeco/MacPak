//! PAK creation functionality

use crate::error::{Error, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use walkdir::WalkDir;

const LSPK_SIGNATURE: u32 = 0x4B53504C; // "LSPK"
const LSPK_VERSION: u32 = 18; // BG3 uses version 18

#[derive(Debug)]
struct FileEntry {
    path: String,
    data: Vec<u8>,
    compressed_data: Vec<u8>,
    offset_in_data: u64,
}

pub fn create_pak<P: AsRef<Path>>(source_dir: P, output_pak: P) -> Result<()> {
    tracing::info!("Scanning directory: {:?}", source_dir.as_ref());
    
    let mut entries = Vec::new();
    let base_path = source_dir.as_ref();
    
    for entry in WalkDir::new(base_path) {
        let entry = entry?;
        
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative_path = path.strip_prefix(base_path)
                .map_err(|e| Error::InvalidPath(format!("{}", e)))?
                .to_string_lossy()
                .replace('\\', "/");
            
            let data = std::fs::read(path)?;
            let compressed_data = compress_lz4(&data)?;
            
            entries.push(FileEntry {
                path: relative_path,
                data,
                compressed_data,
                offset_in_data: 0,
            });
        }
    }
    
    tracing::info!("Found {} files, creating PAK", entries.len());
    
    let file = File::create(output_pak)?;
    let mut writer = BufWriter::new(file);
    
    write_header(&mut writer, &entries)?;
    write_file_list(&mut writer, &mut entries)?;
    write_file_data(&mut writer, &entries)?;
    
    writer.flush()?;
    
    tracing::info!("PAK created successfully");
    Ok(())
}

fn write_header<W: Write>(writer: &mut W, entries: &[FileEntry]) -> Result<()> {
    writer.write_u32::<LittleEndian>(LSPK_SIGNATURE)?;
    writer.write_u32::<LittleEndian>(LSPK_VERSION)?;
    writer.write_u64::<LittleEndian>(28)?; // File list offset
    
    let file_list_size = calculate_file_list_size(entries);
    writer.write_u32::<LittleEndian>(file_list_size)?;
    writer.write_u8(0x0F)?; // Compression flags
    writer.write_u8(0)?;
    writer.write(&[0u8; 16])?; // MD5 placeholder
    writer.write_u16::<LittleEndian>(1)?; // Number of parts
    
    Ok(())
}

fn calculate_file_list_size(entries: &[FileEntry]) -> u32 {
    let mut size = 4;
    for entry in entries {
        size += entry.path.len() + 1 + 8 + 8 + 8 + 4 + 4 + 4;
    }
    size as u32
}

fn write_file_list<W: Write>(writer: &mut W, entries: &mut [FileEntry]) -> Result<()> {
    writer.write_u32::<LittleEndian>(entries.len() as u32)?;
    
    let file_list_size = calculate_file_list_size(entries);
    let mut current_offset = 28 + file_list_size as u64;
    
    for entry in entries.iter_mut() {
        writer.write_all(entry.path.as_bytes())?;
        writer.write_u8(0)?;
        
        entry.offset_in_data = current_offset;
        writer.write_u64::<LittleEndian>(current_offset)?;
        writer.write_u64::<LittleEndian>(entry.compressed_data.len() as u64)?;
        writer.write_u64::<LittleEndian>(entry.data.len() as u64)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(0x0F)?;
        
        let crc = crc32fast::hash(&entry.data);
        writer.write_u32::<LittleEndian>(crc)?;
        
        current_offset += entry.compressed_data.len() as u64;
    }
    
    Ok(())
}

fn write_file_data<W: Write>(writer: &mut W, entries: &[FileEntry]) -> Result<()> {
    for entry in entries {
        writer.write_all(&entry.compressed_data)?;
    }
    Ok(())
}

fn compress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    Ok(lz4_flex::compress_prepend_size(data))
}
