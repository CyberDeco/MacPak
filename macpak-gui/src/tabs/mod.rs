//! Tab views for MacPak

pub mod editor;
pub mod browser;
pub mod gr2;
pub mod pak_ops;
pub mod search;
pub mod tools;

pub use editor::editor_tab;
pub use editor::load_file;
pub use editor::load_file_in_tab;
pub use browser::browser_tab;
pub use gr2::gr2_tab;
pub use pak_ops::pak_ops_tab;
pub use search::search_tab;
pub use tools::tools_tab;
