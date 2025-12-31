//! Shared application state for MacPak

mod app;
mod browser;
mod dyes;
mod editor;
mod gr2;
mod pak_ops;
mod search;
mod tools;
mod virtual_textures;

// Re-export all state types
pub use app::AppState;
pub use browser::{BrowserState, FileEntry, RawImageData, SortColumn};
pub use dyes::{DyeColorEntry, DyesState};
pub use editor::{EditorState, EditorTab, EditorTabsState};
pub use gr2::{Gr2ConversionDirection, Gr2OutputFormat, Gr2State};
pub use pak_ops::{PakCompression, PakOpsState};
pub use search::{SearchResult, SearchState};
pub use tools::{ToolsState, UuidFormat};
pub use virtual_textures::VirtualTexturesState;
