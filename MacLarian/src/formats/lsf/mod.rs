//! LSF (Larian Story Format) binary format module
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

mod document;
mod reader;
mod writer;

// Public API
pub use document::{LsfDocument, LsfNode, LsfAttribute};
pub use reader::{read_lsf, parse_lsf_bytes};
pub use writer::{write_lsf, write_lsf_with_format, serialize_lsf, serialize_lsf_with_format};

// Internal API (used by converter module)
pub(crate) use document::LsfMetadataFormat;

