//! Format conversion utilities
//! 
//! This module handles conversions between different Larian file formats:
//! - LSF (binary) ↔ LSX (XML)
//! - LSX (XML) ↔ LSJ (JSON)
//! - LSF (binary) ↔ LSJ (JSON)
//! - LSBC, LSBX, LSBS conversions - Future

mod lsf_to_lsx;
mod lsx_to_lsf;
mod lsx_to_lsj;
mod lsj_to_lsx;
mod lsf_to_lsj;
mod lsj_to_lsf;

// Re-export conversion functions
pub use lsf_to_lsx::{convert_lsf_to_lsx, to_lsx};
pub use lsx_to_lsf::{convert_lsx_to_lsf, from_lsx};
pub use lsx_to_lsj::{convert_lsx_to_lsj, to_lsj};
pub use lsj_to_lsx::{convert_lsj_to_lsx, to_lsx as lsj_to_lsx_doc};
pub use lsf_to_lsj::convert_lsf_to_lsj;
pub use lsj_to_lsf::convert_lsj_to_lsf;

// Convenience aliases
pub use lsf_to_lsx::convert_lsf_to_lsx as lsf_to_lsx;
pub use lsx_to_lsf::convert_lsx_to_lsf as lsx_to_lsf;
pub use lsx_to_lsj::convert_lsx_to_lsj as lsx_to_lsj;
pub use lsj_to_lsx::convert_lsj_to_lsx as lsj_to_lsx;
pub use lsf_to_lsj::convert_lsf_to_lsj as lsf_to_lsj;
pub use lsj_to_lsf::convert_lsj_to_lsf as lsj_to_lsf;