//! Error types for `MacLarian`

use std::path::PathBuf;

use lz4_flex::frame::Error as Lz4FrameError;
use thiserror::Error;

/// The error type for `MacLarian` operations.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    // ==================== IO Errors ====================
    /// IO error from file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // ==================== PAK Archive Errors ====================
    /// The file is not a valid PAK archive (missing LSPK magic).
    #[error("invalid PAK magic: expected LSPK")]
    InvalidPakMagic,

    /// A multi-part archive part file could not be found.
    #[error("archive part {part} not found")]
    ArchivePartNotFound {
        /// The archive part number (0 = main, 1+ = additional parts).
        part: u8,
    },

    /// A multi-part archive part file does not exist on disk.
    #[error("archive part file not found: {path}")]
    ArchivePartMissing {
        /// The expected path to the archive part.
        path: PathBuf,
    },

    /// PAK extraction completed but some files failed.
    #[error("extraction failed for {failed} of {total} files: {first_error}")]
    PakExtractionPartialFailure {
        /// Number of successfully extracted files.
        total: usize,
        /// Number of failed files.
        failed: usize,
        /// The first error message encountered.
        first_error: String,
    },

    /// None of the requested files were found in the PAK archive.
    #[error("none of the requested files were found in the PAK")]
    RequestedFilesNotFound,

    /// The requested file was not found in the PAK archive.
    #[error("file not found in PAK: {0}")]
    FileNotFoundInPak(String),

    /// PAK header has not been read yet (internal state error).
    #[error("PAK header not read")]
    PakHeaderNotRead,

    /// PAK footer has not been read yet (internal state error).
    #[error("PAK footer not read")]
    PakFooterNotRead,

    /// PAK contains too many files to process.
    #[error("PAK contains too many files: {count}")]
    PakTooManyFiles {
        /// The number of files in the PAK.
        count: usize,
    },

    /// PAK file table exceeds size limits.
    #[error("PAK file table too large: {size} bytes")]
    PakFileTableTooLarge {
        /// The size of the file table in bytes.
        size: usize,
    },

    // ==================== LSF Format Errors ====================
    /// The file is not a valid LSF file (missing LSOF magic).
    #[error("invalid LSF magic: expected LSOF, found {0:?}")]
    InvalidLsfMagic([u8; 4]),

    /// The LSF version is not supported.
    #[error("unsupported LSF version: {version} (supported: 1-7)")]
    UnsupportedLsfVersion {
        /// The version number found in the file.
        version: u32,
    },

    /// Invalid string table index in LSF file.
    #[error("invalid string index: {0}")]
    InvalidStringIndex(String),

    /// Invalid node index in LSF file.
    #[error("invalid node index: {0}")]
    InvalidNodeIndex(i32),

    /// Invalid attribute type in LSF file.
    #[error("invalid attribute type: {0}")]
    InvalidAttributeType(u32),

    // ==================== LOCA Format Errors ====================
    /// The file is not a valid LOCA file.
    #[error("invalid LOCA magic: expected LOCA, found {0:?}")]
    InvalidLocaMagic([u8; 4]),

    // ==================== GR2 Format Errors ====================
    /// The GR2 file is invalid or corrupted.
    #[error("invalid GR2 file: {message}")]
    InvalidGr2 {
        /// Description of what is invalid.
        message: String,
    },

    /// The GR2 file contains no meshes.
    #[error("GR2 file contains no meshes")]
    Gr2NoMeshes,

    /// The GR2 file contains no skinned meshes (required for skeletal export).
    #[error("GR2 file contains no skinned meshes")]
    Gr2NoSkinnedMeshes,

    /// The GR2 file contains no skeleton.
    #[error("GR2 file contains no skeleton")]
    Gr2NoSkeleton,

    // ==================== glTF Conversion Errors ====================
    /// Failed to load or parse a glTF file.
    #[error("failed to load glTF: {message}")]
    GltfLoadFailed {
        /// The error message from the glTF parser.
        message: String,
    },

    /// The glTF mesh is missing required position data.
    #[error("glTF mesh missing position attribute")]
    GltfMissingPositions,

    /// The glTF accessor references a missing buffer view.
    #[error("glTF accessor missing buffer view")]
    GltfMissingBufferView,

    /// Failed to serialize glTF JSON.
    #[error("glTF JSON serialization failed: {message}")]
    GltfSerializationFailed {
        /// The serialization error message.
        message: String,
    },

    // ==================== DDS/PNG Texture Errors ====================
    /// Failed to parse a DDS texture file.
    #[error("failed to parse DDS: {message}")]
    DdsParseFailed {
        /// The parse error message.
        message: String,
    },

    /// The DDS format is not supported.
    #[error("unsupported DDS format: {format}")]
    DdsUnsupportedFormat {
        /// The format identifier or description.
        format: String,
    },

    /// Failed to create an image buffer from texture data.
    #[error("failed to create image buffer")]
    ImageBufferFailed,

    /// Failed to encode PNG image.
    #[error("failed to encode PNG: {message}")]
    PngEncodeFailed {
        /// The encoding error message.
        message: String,
    },

    /// Failed to open or read a PNG file.
    #[error("failed to open PNG: {message}")]
    PngOpenFailed {
        /// The error message.
        message: String,
    },

    /// Failed to create a DDS texture.
    #[error("failed to create DDS: {message}")]
    DdsCreateFailed {
        /// The error message.
        message: String,
    },

    /// Failed to write DDS texture data.
    #[error("failed to write DDS: {message}")]
    DdsWriteFailed {
        /// The error message.
        message: String,
    },

    // ==================== Virtual Texture (GTS/GTP) Errors ====================
    /// The file is not a valid GTS file.
    #[error("invalid GTS magic: expected IVTX")]
    InvalidGtsMagic,

    /// The file is not a valid GTP file.
    #[error("invalid GTP magic")]
    InvalidGtpMagic,

    /// Could not find the corresponding GTS file for a GTP file.
    #[error("GTS file not found for: {gtp_name}")]
    GtsNotFoundForGtp {
        /// The GTP filename.
        gtp_name: String,
    },

    /// The hash was not found in the GTS metadata.
    #[error("hash '{hash}' not found in GTS metadata")]
    GtsHashNotFound {
        /// The hash that was not found.
        hash: String,
    },

    /// The GTP file was not found in the GTS metadata.
    #[error("GTP file '{gtp_name}' not found in GTS metadata")]
    GtpNotInGtsMetadata {
        /// The GTP filename.
        gtp_name: String,
    },

    /// Invalid virtual texture layer index.
    #[error("invalid layer index: {index} (must be 0=BaseMap, 1=NormalMap, or 2=PhysicalMap)")]
    InvalidLayerIndex {
        /// The invalid index.
        index: usize,
    },

    /// Invalid page index in virtual texture.
    #[error("invalid page index: {index}")]
    InvalidPageIndex {
        /// The invalid index.
        index: usize,
    },

    /// Invalid chunk index in virtual texture.
    #[error("invalid chunk index: {index}")]
    InvalidChunkIndex {
        /// The invalid index.
        index: usize,
    },

    /// `GTex` hash not found in any search path.
    #[error("GTex hash '{hash}' not found in search paths")]
    GtexHashNotFound {
        /// The `GTex` hash.
        hash: String,
    },

    /// Virtual texture builder has no textures added.
    #[error("no textures added to virtual texture builder")]
    VirtualTextureNoTextures,

    /// Virtual texture configuration is invalid.
    #[error("virtual texture config invalid: {message}")]
    VirtualTextureConfigInvalid {
        /// The validation error message.
        message: String,
    },

    /// Virtual texture builder output directory not set.
    #[error("virtual texture output directory not set")]
    VirtualTextureOutputNotSet,

    /// Virtual texture source file not found.
    #[error("virtual texture source not found: {path}")]
    VirtualTextureSourceNotFound {
        /// The missing source file path.
        path: PathBuf,
    },

    /// Requested mip level not available in texture.
    #[error("mip level {level} not available in texture")]
    VirtualTextureMipNotAvailable {
        /// The requested mip level.
        level: usize,
    },

    // ==================== Compression/Decompression Errors ====================
    /// LZ4 decompression failed.
    #[error("LZ4 decompression failed: {message}")]
    Lz4DecompressionFailed {
        /// The error message.
        message: String,
    },

    /// LZ4 frame error.
    #[error("LZ4 frame error: {0}")]
    Lz4FrameError(#[from] Lz4FrameError),

    /// Zlib decompression failed.
    #[error("Zlib decompression failed: {message}")]
    ZlibDecompressionFailed {
        /// The error message.
        message: String,
    },

    /// `FastLZ` decompression failed.
    #[error("FastLZ decompression failed: {message}")]
    FastLzDecompressionFailed {
        /// The error message.
        message: String,
    },

    /// `FastLZ` compression failed.
    #[error("FastLZ compression failed: {message}")]
    FastLzCompressionFailed {
        /// The error message.
        message: String,
    },

    /// `BitKnit` decompression failed (GR2 files).
    #[error("BitKnit decompression failed: {message}")]
    BitKnitDecompressionFailed {
        /// The error message.
        message: String,
    },

    /// Unsupported compression method.
    #[error("unsupported compression method: {method}")]
    UnsupportedCompressionMethod {
        /// The compression method identifier.
        method: u8,
    },

    // ==================== Game Data / Path Errors ====================
    /// Could not automatically detect BG3 installation path.
    #[error("could not determine BG3 install path")]
    Bg3PathNotFound,

    /// Could not determine the path to VirtualTextures.pak.
    #[error("could not determine VirtualTextures.pak path")]
    VirtualTexturesPakPathNotFound,

    // ==================== Parsing Errors ====================
    /// XML parsing error.
    #[error("XML parse error: {0}")]
    XmlError(#[from] quick_xml::Error),

    /// XML attribute error.
    #[error("XML attribute error: {0}")]
    XmlAttrError(String),

    /// JSON parsing or serialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// UTF-8 conversion error.
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    // ==================== File System Errors ====================
    /// Invalid file path.
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Directory traversal error.
    #[error("directory walk error: {0}")]
    WalkDirError(String),

    /// Unexpected end of file.
    #[error("unexpected end of file")]
    UnexpectedEof,

    // ==================== Generic/Fallback Errors ====================
    // These are kept for edge cases but should be used sparingly.

    /// Generic decompression error (use specific variants when possible).
    #[error("decompression failed: {0}")]
    DecompressionError(String),

    /// Generic compression error (use specific variants when possible).
    #[error("compression failed: {0}")]
    CompressionError(String),

    /// Invalid format error (use specific variants when possible).
    #[error("invalid format: {0}")]
    InvalidFormat(String),

    /// Invalid index error.
    #[error("invalid index: {0}")]
    InvalidIndex(String),

    /// Search operation error.
    #[error("search error: {0}")]
    SearchError(String),

    /// Generic conversion error (use specific variants when possible).
    ///
    /// This variant is deprecated - prefer using specific error variants.
    #[error("conversion error: {0}")]
    ConversionError(String),

    /// Generic virtual texture error (use specific variants when possible).
    ///
    /// This variant is deprecated - prefer using specific error variants.
    #[error("virtual texture error: {0}")]
    VirtualTexture(String),

    /// Generic DDS error (use specific variants when possible).
    ///
    /// This variant is deprecated - prefer using specific error variants.
    #[error("DDS error: {0}")]
    DdsError(String),
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

/// A specialized Result type for `MacLarian` operations.
pub type Result<T> = std::result::Result<T, Error>;
