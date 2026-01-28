//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LOCA and XML localization format conversions
//!
//! Handles conversions between Larian's localization formats:
//! - LOCA (binary) - Compact binary localization format used in PAK files
//! - XML - Human-readable XML format for editing

mod loca_to_xml;
mod xml_to_loca;

pub use loca_to_xml::{
    convert_loca_to_xml, convert_loca_to_xml_with_progress, to_xml as loca_to_xml_string,
};
pub use xml_to_loca::{
    convert_xml_to_loca, convert_xml_to_loca_with_progress, from_xml as loca_from_xml,
};
