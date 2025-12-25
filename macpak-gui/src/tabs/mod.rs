//! Tab views for MacPak

pub mod editor;
pub mod browser;
pub mod pak_ops;
pub mod search;
pub mod uuid_gen;

pub use editor::editor_tab;
pub use editor::load_file;
pub use browser::browser_tab;
pub use pak_ops::pak_ops_tab;
pub use search::search_tab;
pub use uuid_gen::uuid_gen_tab;
