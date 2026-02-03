//! LSF, LSX, and LSJ format conversions
//!
//! Handles conversions between Larian's document formats:
//! - LSF (binary) - Compact binary format used in PAK files
//! - LSX (XML) - Human-readable XML format
//! - LSJ (JSON) - JSON format used for dialogs and some configs
//!
//! Conversion paths:
//! - LSF ↔ LSX: Direct conversion
//! - LSX ↔ LSJ: Direct conversion
//! - LSF ↔ LSJ: Via LSX intermediate
//!
//!

mod lsf_to_lsj;
mod lsf_to_lsx;
mod lsj_to_lsf;
mod lsj_to_lsx;
mod lsx_to_lsf;
mod lsx_to_lsj;

// Re-export conversion functions
pub use lsf_to_lsj::{convert_lsf_to_lsj, convert_lsf_to_lsj_with_progress};
pub use lsf_to_lsx::{convert_lsf_to_lsx, convert_lsf_to_lsx_with_progress, to_lsx};
pub use lsj_to_lsf::{convert_lsj_to_lsf, convert_lsj_to_lsf_with_progress};
pub use lsj_to_lsx::{convert_lsj_to_lsx, convert_lsj_to_lsx_with_progress};
pub use lsx_to_lsf::{convert_lsx_to_lsf, convert_lsx_to_lsf_with_progress, from_lsx};
pub use lsx_to_lsj::{convert_lsx_to_lsj, convert_lsx_to_lsj_with_progress, to_lsj};

// Convenience aliases matching the module names
pub use lsf_to_lsj::convert_lsf_to_lsj as lsf_to_lsj;
pub use lsf_to_lsj::convert_lsf_to_lsj_with_progress as lsf_to_lsj_with_progress;
pub use lsf_to_lsx::convert_lsf_to_lsx as lsf_to_lsx;
pub use lsf_to_lsx::convert_lsf_to_lsx_with_progress as lsf_to_lsx_with_progress;
pub use lsj_to_lsf::convert_lsj_to_lsf as lsj_to_lsf;
pub use lsj_to_lsf::convert_lsj_to_lsf_with_progress as lsj_to_lsf_with_progress;
pub use lsj_to_lsx::convert_lsj_to_lsx as lsj_to_lsx;
pub use lsj_to_lsx::convert_lsj_to_lsx_with_progress as lsj_to_lsx_with_progress;
pub use lsx_to_lsf::convert_lsx_to_lsf as lsx_to_lsf;
pub use lsx_to_lsf::convert_lsx_to_lsf_with_progress as lsx_to_lsf_with_progress;
pub use lsx_to_lsj::convert_lsx_to_lsj as lsx_to_lsj;
pub use lsx_to_lsj::convert_lsx_to_lsj_with_progress as lsx_to_lsj_with_progress;
