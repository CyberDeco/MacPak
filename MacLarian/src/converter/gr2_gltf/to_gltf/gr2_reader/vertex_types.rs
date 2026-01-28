//! Internal types for vertex parsing.

/// Section header from GR2 file.
#[derive(Debug, Clone, Copy)]
pub(super) struct SectionHeader {
    pub compression: u32,
    pub offset_in_file: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub relocations_offset: u32,
    pub num_relocations: u32,
}

/// Member type enumeration for vertex attributes.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum MemberType {
    None = 0,
    Real32 = 10,
    UInt8 = 12,
    NormalUInt8 = 14,
    BinormalInt16 = 17,
    Real16 = 21,
    Unknown(u32),
}

impl MemberType {
    pub(super) fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::None,
            10 => Self::Real32,
            12 => Self::UInt8,
            14 => Self::NormalUInt8,
            17 => Self::BinormalInt16,
            21 => Self::Real16,
            _ => Self::Unknown(v),
        }
    }

    pub(super) fn element_size(&self) -> usize {
        match self {
            Self::Real32 => 4,
            Self::Real16 | Self::BinormalInt16 => 2,
            Self::UInt8 | Self::NormalUInt8 => 1,
            _ => 4,
        }
    }
}

/// Definition of a vertex member.
#[derive(Debug, Clone)]
pub(super) struct MemberDef {
    pub name: String,
    pub member_type: MemberType,
    pub array_size: u32,
}

impl MemberDef {
    pub fn total_size(&self) -> usize {
        self.member_type.element_size() * self.array_size.max(1) as usize
    }
}

/// Vertex type definition containing member definitions.
#[derive(Debug, Clone)]
pub(super) struct VertexType {
    pub members: Vec<MemberDef>,
}

impl VertexType {
    pub fn stride(&self) -> usize {
        self.members.iter().map(MemberDef::total_size).sum()
    }
}
