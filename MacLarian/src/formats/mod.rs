//! File format handlers for Larian Studios formats
//!
//! Note: `dialog` and `virtual_texture` have been promoted to top-level modules.
//! They are re-exported here for backwards compatibility.

pub mod common;
pub mod lsf;
pub mod lsx;
pub mod lsj;
pub mod loca;
pub mod gr2;
pub mod meta;
pub mod voice_meta;
#[cfg(feature = "audio")]
pub mod wem;

// Re-export common types for convenience
pub use common::{TypeId, get_type_name, type_name_to_id};

// Re-export main document types
pub use lsf::{LsfDocument, LsfNode, LsfAttribute};
pub use lsx::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
pub use lsj::{LsjDocument, LsjNode, LsjAttribute};
pub use loca::{LocaResource, LocalizedText, read_loca, write_loca};
pub use meta::{ModMetadata, parse_meta_lsx};

// Re-export GR2 decompression utilities
pub use gr2::decompress_gr2;

// Re-export virtual texture types (from top-level module for backwards compatibility)
pub use crate::virtual_texture::{VirtualTextureExtractor, GtsFile, GtpFile};

// Re-export dialog types (from top-level module for backwards compatibility)
pub use crate::dialog::{
    Dialog, DialogNode, NodeConstructor, DialogEditorData,
    TaggedText, TagTextEntry, RuleGroup, Rule,
    FlagGroup, FlagType, Flag, SpeakerInfo, GameData,
    parse_dialog, parse_dialog_bytes, parse_dialog_file, parse_dialog_lsf,
    DialogParseError, LocalizationCache, LocalizedEntry, LocalizationError,
    get_available_languages,
};

// Re-export voice meta types
pub use voice_meta::{
    VoiceMetaEntry, VoiceMetaCache,
    load_voice_meta_from_pak, load_voice_meta_from_folder,
    find_voice_files_path, find_voice_meta_path,
};

// Re-export WEM/audio types
#[cfg(feature = "audio")]
pub use wem::{WemError, WemHeader, DecodedAudio, WwiseVorbisHeader, parse_wem_header, parse_wwise_vorbis_header};

#[cfg(feature = "audio")]
pub use wem::{load_wem_file_vgmstream, decode_wwise_vorbis_fallback};
