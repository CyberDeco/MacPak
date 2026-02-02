//! Section management for GR2 file building

use super::super::utils::f32_to_half;

/// A fixup (relocation) to be applied.
#[derive(Debug, Clone)]
pub struct Fixup {
    /// Offset in the source section where the pointer is written
    pub offset_in_section: u32,
    /// Target section index
    pub target_section: u32,
    /// Offset within target section
    pub target_offset: u32,
}

/// A section of data being built.
pub struct Section {
    pub data: Vec<u8>,
    pub fixups: Vec<Fixup>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            fixups: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn pos(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.data.len() % alignment)) % alignment;
        self.data.extend(std::iter::repeat_n(0u8, padding));
    }

    pub fn write_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    pub fn write_u16(&mut self, v: u16) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i16(&mut self, v: i16) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u64(&mut self, v: u64) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_f32(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_f16(&mut self, v: f32) {
        self.data.extend_from_slice(&f32_to_half(v).to_le_bytes());
    }

    /// Write a pointer (64-bit) and record a fixup.
    pub fn write_ptr(&mut self, target_section: u32, target_offset: u32) {
        let offset = self.pos();
        self.fixups.push(Fixup {
            offset_in_section: offset,
            target_section,
            target_offset,
        });
        // Write placeholder (will be resolved later)
        self.write_u64(0);
    }

    /// Write an array reference (count + pointer).
    pub fn write_array_ref(&mut self, count: u32, target_section: u32, target_offset: u32) {
        self.write_u32(count);
        self.write_ptr(target_section, target_offset);
    }

    /// Write a null pointer.
    pub fn write_null_ptr(&mut self) {
        self.write_u64(0);
    }

    /// Write a null array reference.
    pub fn write_null_array(&mut self) {
        self.write_u32(0);
        self.write_u64(0);
    }

    /// Write a string and return its offset.
    pub fn write_string(&mut self, s: &str) -> u32 {
        let offset = self.pos();
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0); // Null terminator
        offset
    }
}
