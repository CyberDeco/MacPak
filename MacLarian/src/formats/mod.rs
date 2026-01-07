//! File format handlers for Larian Studios formats

pub mod common;
pub mod lsf;
pub mod lsx;
pub mod lsj;
pub mod loca;
pub mod gr2;
pub mod meta;
pub mod virtual_texture;
pub mod dialog;
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

// Re-export virtual texture types
pub use virtual_texture::{VirtualTextureExtractor, GtsFile, GtpFile};

// Re-export dialog types
pub use dialog::{
    Dialog, DialogNode, NodeConstructor, DialogEditorData,
    TaggedText, TagTextEntry, RuleGroup, Rule,
    FlagGroup, FlagType, Flag, SpeakerInfo, GameData,
    parse_dialog, parse_dialog_bytes, parse_dialog_file, parse_dialog_lsf,
    DialogParseError, LocalizationCache, LocalizedEntry, LocalizationError,
    get_available_languages,
};

// Re-export WEM/audio types
pub use wem::{WemError, WemHeader, DecodedAudio, WwiseVorbisHeader, parse_wem_header, parse_wwise_vorbis_header};

#[cfg(feature = "audio")]
pub use wem::{load_wem_file_vgmstream, decode_wwise_vorbis_fallback};
