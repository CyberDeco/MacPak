//! LSJ (JSON) format module
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

mod document;
mod reader;
mod writer;

pub use document::{
    LsjAttribute, LsjDocument, LsjHeader, LsjNode, LsjSave, TranslatedFSStringArgument,
    TranslatedFSStringValue,
};
pub use reader::{parse_lsj, read_lsj};
pub use writer::{serialize_lsj, write_lsj};
