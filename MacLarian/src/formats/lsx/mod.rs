//! LSX (XML) format module
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod document;
mod reader;
mod writer;

pub use document::{LsxAttribute, LsxDocument, LsxNode, LsxRegion};
pub use reader::{parse_lsx, read_lsx};
pub use writer::{serialize_lsx, write_lsx};
