#![allow(non_snake_case)]
// Pedantic lints that are stylistic preferences, not code quality issues
#![allow(clippy::needless_pass_by_value)] // API convenience over micro-optimization
#![allow(clippy::must_use_candidate)] // Not critical for internal code
#![allow(clippy::missing_errors_doc)] // Not a public API crate
#![allow(clippy::missing_panics_doc)] // Not a public API crate
#![allow(clippy::doc_markdown)] // Not critical for internal docs
#![allow(clippy::module_name_repetitions)] // Often clearer with full names
#![allow(clippy::items_after_statements)] // Valid Rust pattern
#![allow(clippy::wildcard_imports)] // Common in test modules and prelude re-exports
#![allow(clippy::cast_precision_loss)] // Acceptable for UI/graphics code
#![allow(clippy::cast_possible_truncation)] // Often intentional
#![allow(clippy::cast_sign_loss)] // Often intentional with checked bounds
#![allow(clippy::similar_names)] // Context makes them distinct
#![allow(clippy::too_many_lines)] // Complex functions are sometimes necessary
#![allow(clippy::struct_excessive_bools)] // Domain models may need many booleans
#![allow(clippy::implicit_hasher)] // HashMap is fine for internal code
#![allow(clippy::uninlined_format_args)] // Both styles are valid
#![allow(clippy::collapsible_if)] // Often clearer to keep separate
#![allow(clippy::collapsible_else_if)] // Often clearer to keep separate
#![allow(clippy::single_match_else)] // Often clearer than if-let
#![allow(clippy::match_same_arms)] // Often clearer with explicit arms
#![allow(clippy::manual_let_else)] // Both patterns are valid
#![allow(clippy::redundant_else)] // Explicit else can be clearer
#![allow(clippy::if_not_else)] // Explicit negation can be clearer
#![allow(clippy::match_bool)] // Sometimes clearer than if-else
#![allow(clippy::trivially_copy_pass_by_ref)] // Micro-optimization not needed
#![allow(clippy::borrow_as_ptr)] // Internal code patterns
#![allow(clippy::used_underscore_binding)] // Intentional usage
#![allow(clippy::match_wildcard_for_single_variants)] // Often clearer
#![allow(clippy::semicolon_if_nothing_returned)] // Style preference
#![allow(clippy::redundant_closure_for_method_calls)] // Clearer in GUI method chains
#![allow(clippy::map_unwrap_or)] // Both patterns are valid
#![allow(clippy::option_if_let_else)] // Often clearer with if-let
#![allow(clippy::indexing_slicing)] // Bounds already checked
#![allow(clippy::explicit_iter_loop)] // .iter() is often clearer
#![allow(clippy::clone_on_copy)] // Often clearer for types that look non-Copy
#![allow(clippy::cloned_instead_of_copied)] // Both are valid
#![allow(clippy::useless_conversion)] // Sometimes needed for type inference
#![allow(clippy::unnecessary_wraps)] // Sometimes needed for trait compatibility
#![allow(clippy::let_unit_value)] // Intentional for clarity
#![allow(clippy::assigning_clones)] // Micro-optimization not needed
#![allow(clippy::needless_borrowed_reference)] // Both patterns valid
#![allow(clippy::borrow_deref_ref)] // Both patterns valid
#![allow(clippy::map_err_ignore)] // Style preference
#![allow(clippy::single_match)] // Often clearer
#![allow(clippy::double_ended_iterator_last)] // Intentional usage
#![allow(clippy::format_push_string)] // Clearer in some cases
#![allow(clippy::type_complexity)] // Complex types are sometimes necessary
#![allow(clippy::derivable_impls)] // Sometimes explicit is clearer
#![allow(clippy::redundant_closure)] // Sometimes clearer
#![allow(clippy::cast_lossless)] // Style preference
#![allow(clippy::needless_borrow)] // Often clearer to be explicit
#![allow(clippy::manual_is_ascii_check)] // Direct comparison is fine
#![allow(clippy::io_other_error)] // Compatibility with older code
#![allow(clippy::useless_format)] // Sometimes needed for type compatibility
#![allow(clippy::redundant_else)] // Explicit else can be clearer
#![allow(clippy::manual_clamp)] // Sometimes clearer
#![allow(clippy::op_ref)] // Reference on left operand is fine
#![allow(clippy::needless_return)] // Explicit return can be clearer
#![allow(clippy::use_debug)] // Intentional debug formatting
#![allow(clippy::manual_while_let_some)] // Style preference
#![allow(clippy::let_and_return)] // Explicit binding can be clearer
#![allow(clippy::fn_params_excessive_bools)] // Domain requirements
#![allow(clippy::return_self_not_must_use)] // Not critical for internal code
#![allow(clippy::cast_possible_wrap)] // Intentional
#![allow(clippy::to_string_in_format_args)] // Fine for correctness
#![allow(clippy::only_used_in_recursion)] // False positive sometimes
#![allow(clippy::ignored_unit_patterns)] // Ok(_) is fine for unit
#![allow(clippy::shadow_unrelated)] // Rebinding in reactive code is common
#![allow(clippy::option_map_or_none)] // Both patterns valid
#![allow(clippy::single_char_add_str)] // Clear for character appending
#![allow(clippy::manual_string_new)] // Both patterns valid
#![allow(clippy::single_char_pattern)] // Explicit pattern is fine
#![allow(clippy::if_then_some_else_none)] // Both patterns valid
#![allow(clippy::manual_range_patterns)] // Both patterns valid
#![allow(clippy::needless_borrows_for_generic_args)] // Often clearer
#![allow(clippy::redundant_locals)] // Reactive code rebinding is common
#![allow(clippy::unnecessary_to_owned)] // Sometimes needed for lifetimes
#![allow(clippy::should_implement_trait)] // Not always appropriate
#![allow(clippy::manual_map)] // Both patterns valid
#![allow(clippy::zombie_processes)] // Intentional for subprocesses
#![allow(clippy::while_let_loop)] // Both patterns valid
#![allow(clippy::manual_is_variant_and)] // Both patterns valid
#![allow(clippy::unnecessary_debug_formatting)] // Intentional
#![allow(clippy::manual_pattern_char_comparison)] // Both patterns valid
#![allow(clippy::while_let_on_iterator)] // Both patterns valid
#![allow(clippy::non_std_lazy_statics)] // lazy_static still works
#![allow(clippy::inefficient_to_string)] // Needed for types
#![allow(unknown_lints)] // Suppress warnings about unknown lints
#![allow(clippy::unnecessary_map_or)] // map_or is clearer sometimes
//! `MacPak` - High-level BG3 modding toolkit
use std::path::Path;

// Re-export maclarian
pub use maclarian;

pub mod error;
pub mod index;
pub mod operations;
pub mod workspace;

// GUI-specific modules (moved from MacLarian)
pub mod dialog;
pub mod dyes;
pub mod formats;

// Feature-gated modules
#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "viewer")]
pub mod viewer;

pub use error::{Error, Result};

/// Main toolkit interface
pub struct Toolkit {
    workspace: workspace::Workspace,
    #[allow(dead_code)] // Future: file indexing functionality
    index: index::FileIndex,
}

impl Toolkit {
    /// Creates a new toolkit instance.
    ///
    /// # Errors
    ///
    /// Returns an error if workspace or file index initialization fails.
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: workspace::Workspace::new()?,
            index: index::FileIndex::new()?,
        })
    }

    /// Open or create a workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be opened.
    pub fn open_workspace(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.workspace.open(path)?;
        Ok(())
    }

    /// Extracts a PAK file to a destination directory.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails.
    pub fn extract_pak(&self, pak: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
        operations::extraction::extract_pak(pak, dest)
    }

    /// Converts an LSF file to LSX format.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_lsf_to_lsx(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::lsf_to_lsx(source, dest)
    }

    /// Converts a LOCA file to XML format.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_loca_to_xml(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::loca_to_xml(source, dest)
    }

    /// Converts an XML file to LOCA format.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn convert_xml_to_loca(
        &self,
        source: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> Result<()> {
        operations::conversion::xml_to_loca(source, dest)
    }

    // Virtual texture operations

    /// List information about a GTS file.
    ///
    /// # Errors
    ///
    /// Returns an error if the GTS file cannot be read or parsed.
    pub fn list_gts(
        &self,
        gts_path: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::GtsInfo> {
        operations::virtual_texture::list_gts(gts_path)
    }

    /// Get information about a GTP file.
    ///
    /// # Errors
    ///
    /// Returns an error if the GTP or GTS file cannot be read or parsed.
    pub fn gtp_info(
        &self,
        gtp_path: impl AsRef<Path>,
        gts_path: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::GtpInfo> {
        operations::virtual_texture::gtp_info(gtp_path, gts_path)
    }

    /// Extract a single GTP file to DDS textures.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails.
    pub fn extract_gtp(
        &self,
        gtp_path: impl AsRef<Path>,
        gts_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<()> {
        operations::virtual_texture::extract_gtp(gtp_path, gts_path, output_dir)
    }

    /// Extract all GTP files referenced by a GTS file.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails.
    pub fn extract_virtual_textures(
        &self,
        gts_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<operations::virtual_texture::ExtractResult> {
        operations::virtual_texture::extract_all(gts_path, output_dir)
    }
}
