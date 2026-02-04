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
mod workspace;

// Re-export all state types
pub use app::AppState;
pub use browser::{BrowserState, FileEntry, RawImageData, SortColumn};
pub use config::{
    ConfigState, PersistedBrowserState, PersistedConfig, PersistedDialogueState,
    PersistedEditorState, PersistedSearchState, PersistedWindowState,
};
pub use dialogue::{
    DialogEntry, DialogSource, DialogueState, DisplayFlag, DisplayNode, NODE_TYPE_OPTIONS,
    VoiceMetaCache, VoiceMetaEntry,
};
pub use dyes::{
    DyeColorEntry, DyesState, GeneratedDyeEntry, ImportedDyeEntry, VENDOR_DEFS, VendorDef,
};
pub use editor::{EditorState, EditorTab, EditorTabsState};
pub use gr2::Gr2State;
pub use pak_ops::{ActiveDialog, PakCompression, PakOpsState};
pub use search::{IndexStatus, SearchResult, SearchSortColumn, SearchState, SortDirection};
pub use virtual_textures::VirtualTexturesState;
pub use workspace::{PersistedWorkspaceState, WorkspaceState};
