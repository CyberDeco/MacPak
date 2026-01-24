//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0
//!
//! LSF (Larian Story Format) binary format module

mod document;
mod reader;
mod writer;
pub mod string_table;  // Public because it's used in utils and tests

// Public API
pub use document::{LsfDocument, LsfNode, LsfAttribute, LsfMetadataFormat};
pub use reader::{read_lsf, parse_lsf_bytes};
pub use writer::{write_lsf, write_lsf_with_format, serialize_lsf, serialize_lsf_with_format, LsfFormat};
pub use string_table::StringTable;