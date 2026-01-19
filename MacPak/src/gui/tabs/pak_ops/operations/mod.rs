//! PAK file operations
//!
//! Split into logical groups:
//! - `extract`: Single file extraction operations
//! - `list`: PAK content listing operations
//! - `create`: PAK creation and rebuild operations
//! - `batch`: Batch extract/create operations
//! - `validate`: Mod structure validation

mod batch;
mod create;
mod extract;
mod list;
mod validate;

pub use batch::{batch_create_paks, batch_extract_paks};
pub use create::{create_pak_file, create_pak_from_dropped_folder, execute_create_pak, rebuild_pak_file, rebuild_pak_from_dropped_folder};
pub use extract::{
    execute_individual_extract, extract_dropped_file, extract_individual_files, extract_pak_file,
};
pub use list::{list_dropped_file, list_pak_contents};
pub use validate::{validate_dropped_folder, validate_mod_structure};
