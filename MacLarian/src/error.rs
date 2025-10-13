//! Error types for MacLarian

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid LSF magic: expected LSOF, found {0:?}")]
    InvalidLsfMagic([u8; 4]),
    
    #[error("Unsupported LSF version: {0} (supported: 2-7)")]
    UnsupportedLsfVersion(u32),
    
    #[error("Invalid PAK magic: expected LSPK")]
    InvalidPakMagic,
    
    #[error("Decompression failed: {0}")]
    DecompressionError(String),
    
    #[error("Compression failed: {0}")]
    CompressionError(String),
    
    #[error("XML parse error: {0}")]
    XmlError(#[from] quick_xml::Error),
    
    #[error("XML attribute error: {0}")]
    XmlAttrError(String),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    
    #[error("Invalid string index: {0}")]
    InvalidStringIndex(i32),
    
    #[error("Invalid node index: {0}")]
    InvalidNodeIndex(i32),
    
    #[error("Invalid attribute type: {0}")]
    InvalidAttributeType(u32),
    
    #[error("Format conversion error: {0}")]
    ConversionError(String),
    
    #[error("File not found in PAK: {0}")]
    FileNotFoundInPak(String),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    
    #[error("Walk directory error: {0}")]
    WalkDirError(String),
}

// Add conversion from quick_xml::events::attributes::AttrError
impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Error::XmlAttrError(err.to_string())
    }
}

// Add conversion from walkdir::Error
impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        Error::WalkDirError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;