//! Diff and merge tools for LSF/LSX files
//!
//! Compare mod versions and perform three-way merges for collaborative modding.
//!
//! # Diffing Files
//!
//! ```no_run
//! use maclarian::diff::{diff_files, DiffOptions};
//!
//! // Compare two files (LSF or LSX)
//! let result = diff_files("old/meta.lsx", "new/meta.lsx", &DiffOptions::default())?;
//!
//! println!("Regions changed: {}", result.regions_changed());
//! for change in &result.changes {
//!     println!("{}", change);
//! }
//! # Ok::<(), maclarian::Error>(())
//! ```
//!
//! # Three-Way Merge
//!
//! ```no_run
//! use maclarian::diff::{merge_files, MergeOptions};
//!
//! // Merge changes from two branches
//! let result = merge_files(
//!     "base/meta.lsx",   // Common ancestor
//!     "ours/meta.lsx",   // Our changes
//!     "theirs/meta.lsx", // Their changes
//!     &MergeOptions::default(),
//! )?;
//!
//! if result.has_conflicts() {
//!     println!("Conflicts found:");
//!     for conflict in &result.conflicts {
//!         println!("  {}", conflict);
//!     }
//! } else {
//!     // Save merged result
//!     result.write("merged/meta.lsx")?;
//! }
//! # Ok::<(), maclarian::Error>(())
//! ```
//!

mod lsx_diff;
mod merge;
mod types;

pub use lsx_diff::{diff_documents, diff_files};
pub use merge::{merge_documents, merge_files};
pub use types::{
    AttributeChange, Change, ChangeType, Conflict, ConflictType, DiffOptions, DiffResult,
    MergeOptions, MergeResult, NodeChange, NodePath, RegionChange,
};
