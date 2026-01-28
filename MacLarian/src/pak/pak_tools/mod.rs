//! PAK archive operations

#![allow(
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::cast_possible_truncation
)]

mod cache;
mod decompression;
mod helpers;
mod operations;

pub use cache::PakReaderCache;
pub use operations::PakOperations;

use super::lspk::PakProgress;

/// Progress callback for PAK operations.
///
/// Receives a [`PakProgress`] struct with phase, current/total counts, and optional filename.
/// Must be `Sync + Send` to support parallel decompression/compression.
///
/// # Example
/// ```ignore
/// use maclarian::pak::{PakOperations, PakPhase};
///
/// PakOperations::extract_with_progress(pak, dest, &|progress| {
///     match progress.phase {
///         PakPhase::ReadingTable => println!("Reading file table..."),
///         PakPhase::DecompressingFiles => {
///             println!("{}/{}: {:?}", progress.current, progress.total, progress.current_file);
///         }
///         _ => {}
///     }
/// })?;
/// ```
pub type ProgressCallback<'a> = &'a (dyn Fn(&PakProgress) + Sync + Send);
