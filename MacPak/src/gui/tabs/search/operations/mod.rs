//! Search operations and background processing

mod cache;
mod extraction;
mod indexing;
mod overlays;
mod progress;
mod search;

pub use cache::auto_load_cached_index;
pub use extraction::{execute_extraction, extract_selected_results, extract_single_result};
pub use indexing::{build_index, find_pak_files};
pub use overlays::{progress_overlay, search_overlay};
pub use search::{copy_to_clipboard, perform_search};
