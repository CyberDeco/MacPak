//! LSF type utilities - centralized type handling for LSF/LSX conversions

use crate::error::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

pub type TypeId = u32;

// Type constants
pub const TYPE_NONE: TypeId = 0;
pub const TYPE_UINT8: TypeId = 1;
pub const TYPE_INT16: TypeId = 2;
pub const TYPE_UINT16: TypeId = 3;
pub const TYPE_INT32: TypeId = 4;
pub const TYPE_UINT32: TypeId = 5;
pub const TYPE_FLOAT: TypeId = 6;
pub const TYPE_DOUBLE: TypeId = 7;
pub const TYPE_IVEC2: TypeId = 8;
pub const TYPE_IVEC3: TypeId = 9;
pub const TYPE_IVEC4: TypeId = 10;
pub const TYPE_FVEC2: TypeId = 11;
pub const TYPE_FVEC3: TypeId = 12;
pub const TYPE_FVEC4: TypeId = 13;
pub const TYPE_MAT2X2: TypeId = 14;
pub const TYPE_MAT3X3: TypeId = 15;
pub const TYPE_MAT3X4: TypeId = 16;
pub const TYPE_MAT4X3: TypeId = 17;
pub const TYPE_MAT4X4: TypeId = 18;
pub const TYPE_BOOL: TypeId = 19;
pub const TYPE_STRING: TypeId = 20;
pub const TYPE_PATH: TypeId = 21;
pub const TYPE_FIXEDSTRING: TypeId = 22;
pub const TYPE_LSSTRING: TypeId = 23;
pub const TYPE_UINT64: TypeId = 24;
pub const TYPE_SCRATCHBUFFER: TypeId = 25;
pub const TYPE_OLD_INT64: TypeId = 26;
pub const TYPE_INT8: TypeId = 27;
pub const TYPE_TRANSLATEDSTRING: TypeId = 28;
pub const TYPE_WSTRING: TypeId = 29;
pub const TYPE_LSWSTRING: TypeId = 30;
pub const TYPE_GUID: TypeId = 31;
pub const TYPE_INT64: TypeId = 32;
pub const TYPE_TRANSLATEDFSSTRING: TypeId = 33;

/// Get the human-readable name for an LSF type ID
pub fn get_type_name(type_id: TypeId) -> &'static str {
    match type_id {
        0 => "None",
        1 => "uint8",
        2 => "int16",
        3 => "uint16",
        4 => "int32",
        5 => "uint32",
        6 => "float",
        7 => "double",
        8 => "ivec2",
        9 => "ivec3",
        10 => "ivec4",
        11 => "fvec2",
        12 => "fvec3",
        13 => "fvec4",
        14 => "mat2x2",
        15 => "mat3x3",
        16 => "mat3x4",
        17 => "mat4x3",
        18 => "mat4x4",
        19 => "bool",
        20 => "string",
        21 => "path",
        22 => "FixedString",
        23 => "LSString",
        24 => "uint64",
        25 => "ScratchBuffer",
        26 => "old_int64",
        27 => "int8",
        28 => "TranslatedString",
        29 => "WString",
        30 => "LSWString",
        31 => "guid",
        32 => "int64",
        33 => "TranslatedFSString",
        _ => "Unknown",
    }
}

/// Convert type name string to type ID
pub fn type_name_to_id(type_name: &str) -> TypeId {
    match type_name {
        "None" => 0,
        "uint8" | "Byte" => 1,
        "int16" | "Short" => 2,
        "uint16" | "UShort" => 3,
        "int32" | "Int" => 4,
        "uint32" | "UInt" => 5,
        "float" | "Float" => 6,
        "double" | "Double" => 7,
        "ivec2" | "IVec2" => 8,
        "ivec3" | "IVec3" => 9,
        "ivec4" | "IVec4" => 10,
        "fvec2" | "Vec2" => 11,
        "fvec3" | "Vec3" => 12,
        "fvec4" | "Vec4" => 13,
        "mat2x2" | "Mat2" => 14,
        "mat3x3" | "Mat3" => 15,
        "mat3x4" | "Mat3x4" => 16,
        "mat4x3" | "Mat4x3" => 17,
        "mat4x4" | "Mat4" => 18,
        "bool" | "Bool" => 19,
        "string" | "String" => 20,
        "path" | "Path" => 21,
        "FixedString" => 22,
        "LSString" => 23,
        "uint64" | "ULongLong" => 24,
        "ScratchBuffer" => 25,
        "old_int64" | "Long" => 26,
        "int8" | "Int8" => 27,
        "TranslatedString" => 28,
        "WString" => 29,
        "LSWString" => 30,
        "guid" | "UUID" => 31,
        "int64" | "Int64" => 32,
        "TranslatedFSString" => 33,
        _ => 0,
    }
}

/// Check if a type is numeric
pub fn is_numeric(type_id: TypeId) -> bool {
    matches!(type_id, 1 | 2 | 3 | 4 | 5 | 6 | 7 | 24 | 26 | 27 | 32)
}

/// Get column count for vector/matrix types
pub fn get_columns(type_id: TypeId) -> Option<usize> {
    match type_id {
        8 | 11 | 14 => Some(2),  // ivec2, fvec2, mat2x2
        9 | 12 | 15 | 17 => Some(3),  // ivec3, fvec3, mat3x3, mat4x3
        10 | 13 | 16 | 18 => Some(4),  // ivec4, fvec4, mat3x4, mat4x4
        _ => None,
    }
}

/// Get row count for matrix types
pub fn get_rows(type_id: TypeId) -> Option<usize> {
    match type_id {
        8 | 9 | 10 | 11 | 12 | 13 => Some(1),  // vectors
        14 => Some(2),  // mat2x2
        15 | 16 => Some(3),  // mat3x3, mat3x4
        17 | 18 => Some(4),  // mat4x3, mat4x4
        _ => None,
    }
}

// ============================================================================
// SERIALIZATION: String → Bytes (for LSX → LSF)
// ============================================================================

/// Serialize a value to bytes based on type ID
pub fn serialize_value(buffer: &mut Vec<u8>, type_id: TypeId, value_str: &str) -> Result<usize> {
    let start = buffer.len();
    
    match type_id {
        // String types (null-terminated)
        20 | 21 | 22 | 23 | 29 | 30 => {
            buffer.extend_from_slice(value_str.as_bytes());
            buffer.push(0); // null terminator
        }
        // Bool
        19 => {
            let val = match value_str {
                "True" | "true" | "1" => 1u8,
                _ => 0u8,
            };
            buffer.push(val);
        }
        // Integer types
        0 => {} // None type, no value
        1 | 27 => buffer.push(value_str.parse().unwrap_or(0)),
        2 => buffer.write_i16::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        3 => buffer.write_u16::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        4 => buffer.write_i32::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        5 => buffer.write_u32::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        24 => buffer.write_u64::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        26 | 32 => buffer.write_i64::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        // Float types
        6 => buffer.write_f32::<LittleEndian>(value_str.parse().unwrap_or(0.0))?,
        7 => buffer.write_f64::<LittleEndian>(value_str.parse().unwrap_or(0.0))?,
        // Vector types (space-separated values)
        8 => serialize_ivec(buffer, value_str, 2)?,
        9 => serialize_ivec(buffer, value_str, 3)?,
        10 => serialize_ivec(buffer, value_str, 4)?,
        11 => serialize_fvec(buffer, value_str, 2)?,
        12 => serialize_fvec(buffer, value_str, 3)?,
        13 => serialize_fvec(buffer, value_str, 4)?,
        // Matrix types
        14 | 15 | 16 | 17 | 18 => serialize_matrix(buffer, value_str)?,
        // UUID
        31 => serialize_uuid(buffer, value_str)?,
        // Binary types
        25 => {
            let decoded = BASE64.decode(value_str).unwrap_or_default();
            buffer.extend_from_slice(&decoded);
        }
        _ => {}
    }
    
    Ok(buffer.len() - start)
}

fn serialize_ivec(buffer: &mut Vec<u8>, value_str: &str, count: usize) -> Result<()> {
    let values: Vec<i32> = value_str
        .split_whitespace()
        .take(count)
        .filter_map(|s| s.parse().ok())
        .collect();
    
    for val in values {
        buffer.write_i32::<LittleEndian>(val)?;
    }
    
    Ok(())
}

fn serialize_fvec(buffer: &mut Vec<u8>, value_str: &str, count: usize) -> Result<()> {
    let values: Vec<f32> = value_str
        .split_whitespace()
        .take(count)
        .filter_map(|s| s.parse().ok())
        .collect();
    
    for val in values {
        buffer.write_f32::<LittleEndian>(val)?;
    }
    
    Ok(())
}

fn serialize_matrix(buffer: &mut Vec<u8>, value_str: &str) -> Result<()> {
    let values: Vec<f32> = value_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    
    for val in values {
        buffer.write_f32::<LittleEndian>(val)?;
    }
    
    Ok(())
}

fn serialize_uuid(buffer: &mut Vec<u8>, value_str: &str) -> Result<()> {
    // Parse UUID string and write with byte swapping
    let clean = value_str.replace('-', "");
    if clean.len() != 32 {
        // Write empty GUID
        buffer.extend_from_slice(&[0u8; 16]);
        return Ok(());
    }
    
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        if let Ok(byte) = u8::from_str_radix(&clean[i*2..i*2+2], 16) {
            bytes[i] = byte;
        }
    }
    
    // Apply byte swapping (reverse of format_uuid)
    buffer.push(bytes[3]);
    buffer.push(bytes[2]);
    buffer.push(bytes[1]);
    buffer.push(bytes[0]);
    buffer.push(bytes[5]);
    buffer.push(bytes[4]);
    buffer.push(bytes[7]);
    buffer.push(bytes[6]);
    buffer.push(bytes[9]);
    buffer.push(bytes[8]);
    buffer.push(bytes[11]);
    buffer.push(bytes[10]);
    buffer.push(bytes[13]);
    buffer.push(bytes[12]);
    buffer.push(bytes[15]);
    buffer.push(bytes[14]);
    
    Ok(())
}

/// Serialize TranslatedString
pub fn serialize_translated_string(
    buffer: &mut Vec<u8>,
    handle: &str,
    version: u16,
    _value: &str,
) -> Result<usize> {
    let start = buffer.len();
    
    // Write version
    buffer.write_u16::<LittleEndian>(version)?;
    
    // Write handle length (including null terminator)
    let handle_len = if handle.is_empty() { 0 } else { handle.len() + 1 };
    buffer.write_i32::<LittleEndian>(handle_len as i32)?;
    
    // Write handle string
    if !handle.is_empty() {
        buffer.extend_from_slice(handle.as_bytes());
        buffer.push(0); // null terminator
    }
    
    // BG3 doesn't typically use the value field, so we leave it empty
    
    Ok(buffer.len() - start)
}

// ============================================================================
// DESERIALIZATION: Bytes → String (for LSF → LSX)
// ============================================================================

/// Extract value from bytes and convert to string representation
pub fn extract_value(values: &[u8], offset: usize, length: usize, type_id: TypeId) -> Result<String> {
    if offset + length > values.len() {
        return Ok(String::new());
    }
    
    let bytes = &values[offset..offset + length];
    
    Ok(match type_id {
        // String types (null-terminated)
        20 | 21 | 22 | 23 | 29 | 30 => {
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..end]).into_owned()
        }
        // Bool
        19 => if bytes.first() == Some(&1) { "True" } else { "False" }.to_string(),
        // Integer types
        0 => String::new(),
        1 | 27 => bytes.first().map(|v| v.to_string()).unwrap_or_default(),
        2 => i16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        3 => u16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        4 => i32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        5 => u32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        24 => u64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        26 | 32 => i64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Float types
        6 => f32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        7 => f64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Vector types
        8 => format_ivec(bytes, 2),
        9 => format_ivec(bytes, 3),
        10 => format_ivec(bytes, 4),
        11 => format_fvec(bytes, 2),
        12 => format_fvec(bytes, 3),
        13 => format_fvec(bytes, 4),
        // Matrix types
        14 | 15 | 16 | 17 | 18 => format_matrix(bytes),
        // UUID
        31 => format_uuid(bytes),
        // Binary types
        25 => BASE64.encode(bytes),
        // Unknown
        _ => {
            let byte_list: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
            format!("[{}]", byte_list.join(", "))
        }
    })
}

/// Format integer vector (space-separated)
pub fn format_ivec(bytes: &[u8], count: usize) -> String {
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(i32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

/// Format float vector (space-separated)
pub fn format_fvec(bytes: &[u8], count: usize) -> String {
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(f32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

/// Format matrix (space-separated floats)
pub fn format_matrix(bytes: &[u8]) -> String {
    let count = bytes.len() / 4;
    let values: Vec<String> = (0..count)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                Some(f32::from_le_bytes(bytes[offset..offset+4].try_into().ok()?).to_string())
            } else {
                None
            }
        })
        .collect();
    values.join(" ")
}

/// Format UUID with byte swapping (Windows GUID format)
pub fn format_uuid(bytes: &[u8]) -> String {
    if bytes.len() >= 16 {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[3], bytes[2], bytes[1], bytes[0],
            bytes[5], bytes[4],
            bytes[7], bytes[6],
            bytes[9], bytes[8],
            bytes[11], bytes[10],
            bytes[13], bytes[12],
            bytes[15], bytes[14]
        )
    } else {
        String::new()
    }
}

/// Extract TranslatedString (handle, version, optional value)
pub fn extract_translated_string(values: &[u8], offset: usize, length: usize) -> Result<(String, u16, Option<String>)> {
    if offset + length > values.len() {
        return Ok((String::new(), 0, None));
    }
    
    let bytes = &values[offset..offset + length];
    let mut cursor = Cursor::new(bytes);
    
    let version = cursor.read_u16::<LittleEndian>()?;
    let handle_length = cursor.read_i32::<LittleEndian>()? as usize;
    
    if handle_length == 0 {
        return Ok((String::new(), version, None));
    }
    
    let mut handle_bytes = vec![0u8; handle_length.saturating_sub(1)];
    cursor.read_exact(&mut handle_bytes)?;
    let _ = cursor.read_u8()?; // null terminator
    
    let handle = String::from_utf8_lossy(&handle_bytes).into_owned();
    
    Ok((handle, version, None))
}