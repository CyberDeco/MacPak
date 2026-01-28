//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! LSX to LSJ conversion
//!
//! Key insight: In LSJ, the region IS the root node.
//! LSX: <region id="dialog"><node id="dialog">...</node></region>
//! LSJ: "regions": { "dialog": { ... } }

use crate::error::Result;
use crate::formats::common::{TypeId, get_type_name, type_name_to_id};
use crate::formats::lsj::{self, LsjAttribute, LsjDocument, LsjHeader, LsjNode, LsjSave};
use crate::formats::lsx::{LsxAttribute as LsxAttrType, LsxDocument, LsxNode};
use std::collections::HashMap;
use std::path::Path;

/// Convert LSX file to LSJ format
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsx_to_lsj<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    convert_lsx_to_lsj_with_progress(source, dest, &|_| {})
}

/// Convert LSX file to LSJ format with progress callback
///
/// # Errors
/// Returns an error if reading or conversion fails.
pub fn convert_lsx_to_lsj_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    progress: crate::converter::ConvertProgressCallback,
) -> Result<()> {
    use crate::converter::{ConvertPhase, ConvertProgress};
    tracing::info!(
        "Converting LSX→LSJ: {:?} → {:?}",
        source.as_ref(),
        dest.as_ref()
    );

    progress(&ConvertProgress::with_file(
        ConvertPhase::ReadingSource,
        1,
        3,
        "Reading LSX file...",
    ));
    let lsx_doc = crate::formats::lsx::read_lsx(&source)?;

    let region_count = lsx_doc.regions.len();
    progress(&ConvertProgress::with_file(
        ConvertPhase::Converting,
        2,
        3,
        format!("Converting {region_count} regions to JSON..."),
    ));
    let lsj_doc = to_lsj(&lsx_doc)?;

    progress(&ConvertProgress::with_file(
        ConvertPhase::WritingOutput,
        3,
        3,
        "Writing LSJ file...",
    ));
    lsj::write_lsj(&lsj_doc, dest)?;

    tracing::info!("Conversion complete");
    Ok(())
}

/// Convert LSX document to LSJ document
///
/// # Errors
/// Returns an error if conversion fails.
pub fn to_lsj(lsx: &LsxDocument) -> Result<LsjDocument> {
    let mut regions = HashMap::new();

    for region in &lsx.regions {
        // The first node in the region IS the region content in LSJ
        // Don't merge - just convert it directly
        if let Some(root_node) = region.nodes.first() {
            let lsj_region = convert_node_to_lsj(root_node)?;
            regions.insert(region.id.clone(), lsj_region);
        }
    }

    Ok(LsjDocument {
        save: LsjSave {
            header: LsjHeader {
                version: format!("{}.{}.{}.{}", lsx.major, lsx.minor, lsx.revision, lsx.build),
            },
            regions,
        },
    })
}

/// Convert an LSX node to an LSJ node
fn convert_node_to_lsj(lsx_node: &LsxNode) -> Result<LsjNode> {
    let mut lsj_node = LsjNode::new();

    // Convert attributes
    for attr in &lsx_node.attributes {
        let lsj_attr = convert_attribute(attr)?;
        lsj_node.attributes.insert(attr.id.clone(), lsj_attr);
    }

    // Convert children
    for child in &lsx_node.children {
        convert_child_node(&mut lsj_node, child)?;
    }

    Ok(lsj_node)
}

/// Convert a child node - becomes an array entry in LSJ
fn convert_child_node(parent: &mut LsjNode, child: &LsxNode) -> Result<()> {
    // Create a node for this child
    let mut child_node = LsjNode::new();

    // Add attributes
    for attr in &child.attributes {
        let lsj_attr = convert_attribute(attr)?;
        child_node.attributes.insert(attr.id.clone(), lsj_attr);
    }

    // Recursively add children
    for grandchild in &child.children {
        convert_child_node(&mut child_node, grandchild)?;
    }

    // Add this child to parent's children array
    parent
        .children
        .entry(child.id.clone())
        .or_default()
        .push(child_node);

    Ok(())
}

fn convert_attribute(attr: &LsxAttrType) -> Result<LsjAttribute> {
    let type_id = type_name_to_id(&attr.type_name);
    let type_name = get_type_name(type_id);

    // Handle TranslatedString (type 28)
    if type_id == 28 {
        return Ok(LsjAttribute::TranslatedString {
            type_name: type_name.to_string(),
            value: if attr.value.is_empty() {
                None
            } else {
                Some(attr.value.clone())
            },
            handle: attr.handle.clone().unwrap_or_default(),
            version: attr.version,
        });
    }

    // Handle TranslatedFSString (type 33)
    if type_id == 33 {
        return Ok(LsjAttribute::TranslatedFSString {
            type_name: type_name.to_string(),
            value: if attr.value.is_empty() {
                None
            } else {
                Some(attr.value.clone())
            },
            handle: attr.handle.clone().unwrap_or_default(),
            arguments: Vec::new(),
        });
    }

    // Convert value based on type
    let json_value = convert_value_to_json(type_id, &attr.value)?;

    Ok(LsjAttribute::Simple {
        type_name: type_name.to_string(),
        value: json_value,
    })
}

fn convert_value_to_json(type_id: TypeId, value_str: &str) -> Result<serde_json::Value> {
    use serde_json::Value;

    Ok(match type_id {
        // Integers
        1 => Value::Number(value_str.parse::<u8>().unwrap_or(0).into()), // uint8
        27 => Value::Number(value_str.parse::<i8>().unwrap_or(0).into()), // int8
        2 => Value::Number(value_str.parse::<i16>().unwrap_or(0).into()),
        3 => Value::Number(value_str.parse::<u16>().unwrap_or(0).into()),
        4 => Value::Number(value_str.parse::<i32>().unwrap_or(0).into()),
        5 => Value::Number(value_str.parse::<u32>().unwrap_or(0).into()),
        24 => Value::Number(value_str.parse::<u64>().unwrap_or(0).into()),
        26 | 32 => Value::Number(value_str.parse::<i64>().unwrap_or(0).into()),

        // Floats
        6 => {
            let f = value_str.parse::<f32>().unwrap_or(0.0);
            Value::Number(
                serde_json::Number::from_f64(f64::from(f)).unwrap_or(serde_json::Number::from(0)),
            )
        }
        7 => {
            let f = value_str.parse::<f64>().unwrap_or(0.0);
            Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)))
        }

        // Bool
        19 => Value::Bool(value_str == "True" || value_str == "true" || value_str == "1"),

        // Vectors and matrices - keep as strings (space-separated)
        8..=18 => Value::String(value_str.to_string()),

        // All other types (strings, UUIDs, paths, etc.)
        _ => Value::String(value_str.to_string()),
    })
}
