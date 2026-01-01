//! Format conversion utilities
//!
//! This module handles conversions between different Larian file formats:
//! - LSF (binary) ↔ LSX (XML) - Direct conversion
//! - LSX (XML) ↔ LSJ (JSON) - Direct conversion
//! - LSF (binary) ↔ LSJ (JSON) - via LSX intermediate
//! - GR2 (Granny2) ↔ glTF - 3D model conversion
//! - LSBC, LSBX, LSBS conversions - Future

mod lsf_to_lsx;
mod lsx_to_lsf;
mod lsx_to_lsj;
mod lsj_to_lsx;
mod lsf_to_lsj;
mod lsj_to_lsf;
mod loca_to_xml;
mod xml_to_loca;

pub mod gr2_to_gltf;
pub mod gltf_to_gr2;

// Re-export conversion functions
pub use lsf_to_lsx::{convert_lsf_to_lsx, to_lsx};
pub use lsf_to_lsj::convert_lsf_to_lsj;
pub use lsx_to_lsf::{convert_lsx_to_lsf, from_lsx};
pub use lsx_to_lsj::{convert_lsx_to_lsj, to_lsj};
pub use lsj_to_lsf::convert_lsj_to_lsf;
pub use lsj_to_lsx::{convert_lsj_to_lsx, to_lsx as lsj_to_lsx_doc};

// GR2/glTF conversion exports
pub use gr2_to_gltf::{convert_gr2_to_glb, convert_gr2_to_gltf, convert_gr2_bytes_to_glb};
pub use gltf_to_gr2::{convert_gltf_to_gr2, convert_gltf_bytes_to_gr2};

// LOCA conversion exports
pub use loca_to_xml::{convert_loca_to_xml, to_xml as loca_to_xml_string};
pub use xml_to_loca::convert_xml_to_loca;

// Convenience aliases
pub use lsf_to_lsx::convert_lsf_to_lsx as lsf_to_lsx;
pub use lsf_to_lsj::convert_lsf_to_lsj as lsf_to_lsj;
pub use lsx_to_lsf::convert_lsx_to_lsf as lsx_to_lsf;
pub use lsx_to_lsj::convert_lsx_to_lsj as lsx_to_lsj;
pub use lsj_to_lsf::convert_lsj_to_lsf as lsj_to_lsf;
pub use lsj_to_lsx::convert_lsj_to_lsx as lsj_to_lsx;
