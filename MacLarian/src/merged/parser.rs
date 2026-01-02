//! LSX parsing for merged databases
//!
//! Parses VisualBank, MaterialBank, TextureBank, and VirtualTextureBank regions
//! from _merged.lsx files.

use crate::formats::lsx::{LsxNode, LsxRegion};

use super::types::*;

use std::path::Path;

/// Parse a VisualBank region into the database
pub fn parse_visual_bank(region: &LsxRegion, db: &mut MergedDatabase) {
    for node in &region.nodes {
        if node.id != "VisualBank" {
            continue;
        }
        for resource in &node.children {
            if resource.id != "Resource" {
                continue;
            }
            if let Some(visual) = parse_visual_resource(resource) {
                // Index by visual name
                db.visuals_by_name
                    .insert(visual.name.clone(), visual.id.clone());

                // Index by GR2 filename (multiple visuals can share the same GR2)
                let gr2_filename = Path::new(&visual.gr2_path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                if !gr2_filename.is_empty() {
                    db.visuals_by_gr2
                        .entry(gr2_filename)
                        .or_default()
                        .push(visual.id.clone());
                }

                db.visuals_by_id.insert(visual.id.clone(), visual);
            }
        }
    }
}

/// Parse a single visual Resource node
fn parse_visual_resource(node: &LsxNode) -> Option<VisualAsset> {
    let mut id = String::new();
    let mut name = String::new();
    let mut gr2_path = String::new();
    let mut material_ids = Vec::new();

    for attr in &node.attributes {
        match attr.id.as_str() {
            "ID" => id = attr.value.clone(),
            "Name" => name = attr.value.clone(),
            "SourceFile" => gr2_path = attr.value.clone(),
            _ => {}
        }
    }

    // Extract MaterialIDs from Objects children
    for child in &node.children {
        if child.id == "Objects" {
            for attr in &child.attributes {
                if attr.id == "MaterialID" && !attr.value.is_empty() {
                    if !material_ids.contains(&attr.value) {
                        material_ids.push(attr.value.clone());
                    }
                }
            }
        }
    }

    if id.is_empty() || gr2_path.is_empty() {
        return None;
    }

    Some(VisualAsset {
        id,
        name,
        gr2_path,
        source_pak: String::new(),
        material_ids,
        textures: Vec::new(),
        virtual_textures: Vec::new(),
    })
}

/// Parse a MaterialBank region into the database
pub fn parse_material_bank(region: &LsxRegion, db: &mut MergedDatabase) {
    for node in &region.nodes {
        if node.id != "MaterialBank" {
            continue;
        }
        for resource in &node.children {
            if resource.id != "Resource" {
                continue;
            }
            if let Some(material) = parse_material_resource(resource) {
                db.materials.insert(material.id.clone(), material);
            }
        }
    }
}

/// Parse a single material Resource node
fn parse_material_resource(node: &LsxNode) -> Option<MaterialDef> {
    let mut id = String::new();
    let mut name = String::new();
    let mut source_file = String::new();
    let mut texture_ids = Vec::new();

    for attr in &node.attributes {
        match attr.id.as_str() {
            "ID" => id = attr.value.clone(),
            "Name" => name = attr.value.clone(),
            "SourceFile" => source_file = attr.value.clone(),
            _ => {}
        }
    }

    // Extract texture references from Texture2DParameters children
    for child in &node.children {
        if child.id == "Texture2DParameters" {
            let mut param_name = String::new();
            let mut tex_id = String::new();

            for attr in &child.attributes {
                match attr.id.as_str() {
                    "ParameterName" => param_name = attr.value.clone(),
                    "ID" => tex_id = attr.value.clone(),
                    _ => {}
                }
            }

            if !tex_id.is_empty() {
                texture_ids.push(TextureParam {
                    name: param_name,
                    texture_id: tex_id,
                });
            }
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(MaterialDef {
        id,
        name,
        source_file,
        source_pak: String::new(),
        texture_ids,
    })
}

/// Parse a TextureBank region into the database
pub fn parse_texture_bank(region: &LsxRegion, db: &mut MergedDatabase) {
    for node in &region.nodes {
        if node.id != "TextureBank" {
            continue;
        }
        for resource in &node.children {
            if resource.id != "Resource" {
                continue;
            }
            if let Some(texture) = parse_texture_resource(resource) {
                db.textures.insert(texture.id.clone(), texture);
            }
        }
    }
}

/// Parse a single texture Resource node
fn parse_texture_resource(node: &LsxNode) -> Option<TextureRef> {
    let mut id = String::new();
    let mut name = String::new();
    let mut dds_path = String::new();
    let mut width = 0u32;
    let mut height = 0u32;

    for attr in &node.attributes {
        match attr.id.as_str() {
            "ID" => id = attr.value.clone(),
            "Name" => name = attr.value.clone(),
            "SourceFile" => dds_path = attr.value.clone(),
            "Width" => width = attr.value.parse().unwrap_or(0),
            "Height" => height = attr.value.parse().unwrap_or(0),
            _ => {}
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(TextureRef {
        id,
        name,
        dds_path,
        source_pak: String::new(),
        width,
        height,
        parameter_name: None,
    })
}

/// Parse a VirtualTextureBank region into the database
pub fn parse_virtual_texture_bank(region: &LsxRegion, db: &mut MergedDatabase) {
    for node in &region.nodes {
        if node.id != "VirtualTextureBank" {
            continue;
        }
        for resource in &node.children {
            if resource.id != "Resource" {
                continue;
            }
            if let Some(vt) = parse_virtual_texture_resource(resource) {
                db.virtual_textures.insert(vt.id.clone(), vt);
            }
        }
    }
}

/// Parse a single virtual texture Resource node
fn parse_virtual_texture_resource(node: &LsxNode) -> Option<VirtualTextureRef> {
    let mut id = String::new();
    let mut name = String::new();
    let mut gtex_hash = String::new();

    for attr in &node.attributes {
        match attr.id.as_str() {
            "ID" => id = attr.value.clone(),
            "Name" => name = attr.value.clone(),
            "GTexFileName" => gtex_hash = attr.value.clone(),
            _ => {}
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(VirtualTextureRef {
        id,
        name,
        gtex_hash,
    })
}

/// Merge one database into another
pub fn merge_databases(target: &mut MergedDatabase, source: MergedDatabase) {
    target.visuals_by_id.extend(source.visuals_by_id);
    target.visuals_by_name.extend(source.visuals_by_name);

    // Merge visuals_by_gr2 (append to existing vecs)
    for (gr2, ids) in source.visuals_by_gr2 {
        target.visuals_by_gr2.entry(gr2).or_default().extend(ids);
    }

    target.materials.extend(source.materials);
    target.textures.extend(source.textures);
    target.virtual_textures.extend(source.virtual_textures);
}

/// Resolve cross-references between visuals, materials, and textures
pub fn resolve_references(db: &mut MergedDatabase) {
    let materials = db.materials.clone();
    let textures = db.textures.clone();
    let virtual_textures = db.virtual_textures.clone();

    for visual in db.visuals_by_id.values_mut() {
        let mut resolved_textures = Vec::new();
        let mut resolved_vts = Vec::new();

        for mat_id in &visual.material_ids {
            if let Some(material) = materials.get(mat_id) {
                for tex_param in &material.texture_ids {
                    if let Some(texture) = textures.get(&tex_param.texture_id) {
                        let mut tex_ref = texture.clone();
                        tex_ref.parameter_name = Some(tex_param.name.clone());
                        if !resolved_textures.iter().any(|t: &TextureRef| t.id == tex_ref.id) {
                            resolved_textures.push(tex_ref);
                        }
                    }
                }
            }
        }

        // Find virtual texture by EXACT name match
        if let Some(vt) = virtual_textures.values().find(|vt| vt.name == visual.name) {
            resolved_vts.push(vt.clone());
        }

        visual.textures = resolved_textures;
        visual.virtual_textures = resolved_vts;
    }
}
