//! Tab views for MacPak

pub mod editor;
pub mod browser;
pub mod pak_ops;
pub mod gr2;
pub mod virtual_textures;
pub mod dyes;
pub mod search;

pub use browser::browser_tab;
pub use browser::kill_preview_process;
pub use editor::load_file_in_tab;
pub use editor::editor_tab;
pub use pak_ops::pak_ops_tab;
pub use gr2::gr2_tab;
pub use virtual_textures::virtual_textures_tab;
pub use dyes::dyes_tab;
pub use search::search_tab;
