//! Shared application state for MacPak

mod app;
mod browser;
mod config;
mod dialogue;
mod dyes;
mod editor;
pub mod gr2;
mod pak_ops;
mod search;
pub mod virtual_textures;

// Re-export all state types
pub use app::AppState;
pub use browser::{BrowserState, FileEntry, RawImageData, SortColumn};
pub use config::ConfigState;
pub use dialogue::{DialogueState, DialogEntry, DialogSource, DisplayNode, DisplayFlag, NODE_TYPE_OPTIONS};
pub use dyes::{DyeColorEntry, DyesState, GeneratedDyeEntry, ImportedDyeEntry, VendorDef, VENDOR_DEFS};
pub use editor::{EditorState, EditorTab, EditorTabsState};
pub use gr2::Gr2State;
pub use pak_ops::{PakCompression, PakOpsState};
pub use search::{SearchResult, SearchState};
pub use virtual_textures::VirtualTexturesState;
