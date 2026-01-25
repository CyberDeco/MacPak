//! LSX (XML) format module
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod document;
mod reader;
mod writer;

pub use document::{LsxDocument, LsxRegion, LsxNode, LsxAttribute};
pub use reader::{read_lsx, parse_lsx};
pub use writer::{write_lsx, serialize_lsx};