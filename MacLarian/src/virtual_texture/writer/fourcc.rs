//! `FourCC` metadata tree builder and serializer
//!
//!
//!
//! The `FourCC` metadata in GTS files uses a hierarchical tree structure
//! with format codes indicating node types:
//! - 1: Container node (has children)
//! - 2: String (UTF-16LE, null-terminated)
//! - 3: Int (u32)
//! - 8: Binary data
//! - 0x0D: GUID (16 bytes)

use crate::error::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Seek, SeekFrom, Write};

/// Format codes for `FourCC` nodes
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FourCCFormat {
    /// Container node with children
    Node = 1,
    /// String value (UTF-16LE)
    String = 2,
    /// Integer value (u32)
    Int = 3,
    /// Binary data
    Binary = 8,
    /// GUID (16 bytes)
    Guid = 0x0D,
}

/// A node in the `FourCC` metadata tree
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum FourCCNode {
    /// Container node with children
    Container {
        fourcc: [u8; 4],
        children: Vec<FourCCNode>,
    },
    /// String value
    String { fourcc: [u8; 4], value: String },
    /// Integer value
    Int { fourcc: [u8; 4], value: u32 },
    /// Binary data
    Binary { fourcc: [u8; 4], data: Vec<u8> },
    /// GUID value
    Guid { fourcc: [u8; 4], guid: [u8; 16] },
}

impl FourCCNode {
    /// Create a container node
    #[must_use]
    pub fn container(fourcc: [u8; 4]) -> Self {
        Self::Container {
            fourcc,
            children: Vec::new(),
        }
    }

    /// Create a string node
    #[must_use]
    pub fn string(fourcc: [u8; 4], value: impl Into<String>) -> Self {
        Self::String {
            fourcc,
            value: value.into(),
        }
    }

    /// Create an integer node
    #[must_use]
    pub fn int(fourcc: [u8; 4], value: u32) -> Self {
        Self::Int { fourcc, value }
    }

    /// Create a binary node
    #[must_use]
    pub fn binary(fourcc: [u8; 4], data: Vec<u8>) -> Self {
        Self::Binary { fourcc, data }
    }

    /// Create a GUID node
    #[must_use]
    pub fn guid(fourcc: [u8; 4], guid: [u8; 16]) -> Self {
        Self::Guid { fourcc, guid }
    }

    /// Add a child to a container node
    pub fn add_child(&mut self, child: FourCCNode) {
        if let Self::Container { children, .. } = self {
            children.push(child);
        }
    }
}

/// `FourCC` metadata tree
#[derive(Debug, Clone, Default)]
pub struct FourCCTree {
    root: Option<FourCCNode>,
}

impl FourCCTree {
    /// Create a new empty tree
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Set the root node
    pub fn set_root(&mut self, node: FourCCNode) {
        self.root = Some(node);
    }

    /// Write the tree to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<u32> {
        let start_pos = writer.stream_position()?;

        if let Some(ref root) = self.root {
            Self::write_node(writer, root)?;
        }

        let end_pos = writer.stream_position()?;
        Ok((end_pos - start_pos) as u32)
    }

    /// Write a single node
    fn write_node<W: Write + Seek>(writer: &mut W, node: &FourCCNode) -> Result<()> {
        match node {
            FourCCNode::Container { fourcc, children } => {
                // Write header with placeholder length
                let header_pos = writer.stream_position()?;
                writer.write_all(fourcc)?;
                writer.write_u8(FourCCFormat::Node as u8)?;
                writer.write_u8(0)?; // Extended length flag (0 for now)
                writer.write_u16::<LittleEndian>(0)?; // Placeholder length

                // Write children
                for child in children {
                    Self::write_node(writer, child)?;
                }

                // Calculate and update length
                let end_pos = writer.stream_position()?;
                let content_len = end_pos - header_pos - 8; // Exclude header

                // Seek back and update length
                writer.seek(SeekFrom::Start(header_pos + 6))?;
                if content_len > 0xFFFF {
                    // Need extended length
                    writer.seek(SeekFrom::Start(header_pos + 5))?;
                    writer.write_u8(1)?; // Extended length flag
                    writer.write_u16::<LittleEndian>((content_len >> 16) as u16)?;
                    // Write lower 16 bits after seek
                    writer.seek(SeekFrom::Start(header_pos + 6))?;
                    writer.write_u16::<LittleEndian>(content_len as u16)?;
                } else {
                    writer.write_u16::<LittleEndian>(content_len as u16)?;
                }

                // Seek back to end
                writer.seek(SeekFrom::Start(end_pos))?;

                // Align to 4 bytes
                Self::align_to_4(writer)?;
            }

            FourCCNode::String { fourcc, value } => {
                // Convert to UTF-16LE with null terminator
                let utf16: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes: Vec<u8> = utf16.iter().flat_map(|&c| c.to_le_bytes()).collect();

                writer.write_all(fourcc)?;
                writer.write_u8(FourCCFormat::String as u8)?;
                writer.write_u8(0)?;
                writer.write_u16::<LittleEndian>(bytes.len() as u16)?;
                writer.write_all(&bytes)?;

                Self::align_to_4(writer)?;
            }

            FourCCNode::Int { fourcc, value } => {
                writer.write_all(fourcc)?;
                writer.write_u8(FourCCFormat::Int as u8)?;
                writer.write_u8(0)?;
                writer.write_u16::<LittleEndian>(4)?;
                writer.write_u32::<LittleEndian>(*value)?;

                // Already aligned (4 bytes of data)
            }

            FourCCNode::Binary { fourcc, data } => {
                writer.write_all(fourcc)?;
                writer.write_u8(FourCCFormat::Binary as u8)?;
                writer.write_u8(0)?;
                writer.write_u16::<LittleEndian>(data.len() as u16)?;
                writer.write_all(data)?;

                Self::align_to_4(writer)?;
            }

            FourCCNode::Guid { fourcc, guid } => {
                writer.write_all(fourcc)?;
                writer.write_u8(FourCCFormat::Guid as u8)?;
                writer.write_u8(0)?;
                writer.write_u16::<LittleEndian>(16)?;
                writer.write_all(guid)?;

                // Already aligned (16 bytes of data)
            }
        }

        Ok(())
    }

    /// Align writer position to 4-byte boundary
    fn align_to_4<W: Write + Seek>(writer: &mut W) -> Result<()> {
        let pos = writer.stream_position()?;
        let padding = (4 - (pos % 4)) % 4;
        for _ in 0..padding {
            writer.write_u8(0)?;
        }
        Ok(())
    }
}

/// Build the standard `FourCC` metadata tree for a virtual texture
pub fn build_metadata_tree(
    texture_name: &str,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    layers: &[(&str, &str)], // (name, type) pairs
    guid: &[u8; 16],
) -> FourCCTree {
    let mut tree = FourCCTree::new();

    // Build META root
    let mut meta = FourCCNode::container(*b"META");

    // ATLS (Atlas)
    let mut atls = FourCCNode::container(*b"ATLS");
    let mut txts = FourCCNode::container(*b"TXTS");

    // TXTR (Texture entry)
    let mut txtr = FourCCNode::container(*b"TXTR");
    txtr.add_child(FourCCNode::string(*b"NAME", texture_name));
    txtr.add_child(FourCCNode::int(*b"WDTH", width));
    txtr.add_child(FourCCNode::int(*b"HGHT", height));
    txtr.add_child(FourCCNode::int(*b"XXXX", x));
    txtr.add_child(FourCCNode::int(*b"YYYY", y));
    txtr.add_child(FourCCNode::string(*b"ADDR", ""));
    txtr.add_child(FourCCNode::binary(*b"SRGB", vec![1, 0, 0, 0]));
    txtr.add_child(FourCCNode::guid(*b"THMB", *guid));

    txts.add_child(txtr);
    atls.add_child(txts);
    meta.add_child(atls);

    // PROJ (Project) - empty
    meta.add_child(FourCCNode::container(*b"PROJ"));

    // LINF (Layer info)
    let mut linf = FourCCNode::container(*b"LINF");
    for (i, (name, layer_type)) in layers.iter().enumerate() {
        let mut layr = FourCCNode::container(*b"LAYR");
        layr.add_child(FourCCNode::int(*b"INDX", i as u32));
        layr.add_child(FourCCNode::string(*b"TYPE", *layer_type));
        layr.add_child(FourCCNode::string(*b"NAME", *name));
        linf.add_child(layr);
    }
    meta.add_child(linf);

    // INFO (Build info)
    let mut info = FourCCNode::container(*b"INFO");

    // COMP (Compiler)
    let mut comp = FourCCNode::container(*b"COMP");
    comp.add_child(FourCCNode::binary(*b"CMPW", vec![1, 0])); // Version 1.0
    comp.add_child(FourCCNode::binary(*b"BLDV", vec![1, 0, 0, 0, 0, 0]));
    info.add_child(comp);

    info.add_child(FourCCNode::string(*b"DATE", ""));
    info.add_child(FourCCNode::string(*b"BLKS", ""));
    info.add_child(FourCCNode::string(*b"TILE", ""));
    info.add_child(FourCCNode::string(*b"BDPR", ""));
    info.add_child(FourCCNode::int(*b"LTMP", 0));

    meta.add_child(info);

    tree.set_root(meta);
    tree
}
