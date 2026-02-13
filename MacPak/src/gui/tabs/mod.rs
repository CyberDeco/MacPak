//! Tab views for MacPak

pub mod browser;
pub mod convert;
pub mod dialogue;
pub mod dyes;
pub mod editor;
pub mod pak_ops;
pub mod search;
pub mod workbench;

pub use browser::browser_tab;
pub use browser::kill_preview_process;
pub use convert::{convert_tab, subtab_bar};
pub use dialogue::dialogue_tab;
pub use dyes::dyes_tab;
pub use editor::editor_tab;
pub use editor::load_file_in_tab;
pub use pak_ops::pak_ops_tab;
pub use search::search_tab;
pub use workbench::workbench_tab;
