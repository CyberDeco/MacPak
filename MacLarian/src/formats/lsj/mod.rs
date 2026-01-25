//! LSJ (JSON) format module
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod reader;
mod writer;
mod document;

pub use document::{LsjDocument, LsjNode, LsjAttribute, LsjHeader, LsjSave, TranslatedFSStringArgument, TranslatedFSStringValue};
pub use reader::{read_lsj, parse_lsj};
pub use writer::{write_lsj, serialize_lsj};