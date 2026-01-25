//! Common type utilities - centralized type handling for LSF/LSX/LSJ conversions

use crate::error::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

pub type TypeId = u32;

// Type constants used internally for type identification
pub(crate) const TYPE_NONE: TypeId = 0;
pub(crate) const TYPE_UINT8: TypeId = 1;
pub(crate) const TYPE_INT16: TypeId = 2;
pub(crate) const TYPE_UINT16: TypeId = 3;
pub(crate) const TYPE_INT32: TypeId = 4;
pub(crate) const TYPE_UINT32: TypeId = 5;
pub(crate) const TYPE_FLOAT: TypeId = 6;
pub(crate) const TYPE_DOUBLE: TypeId = 7;
pub(crate) const TYPE_IVEC2: TypeId = 8;
pub(crate) const TYPE_IVEC3: TypeId = 9;
pub(crate) const TYPE_IVEC4: TypeId = 10;
pub(crate) const TYPE_FVEC2: TypeId = 11;
pub(crate) const TYPE_FVEC3: TypeId = 12;
pub(crate) const TYPE_FVEC4: TypeId = 13;
pub(crate) const TYPE_MAT2X2: TypeId = 14;
pub(crate) const TYPE_MAT3X3: TypeId = 15;
pub(crate) const TYPE_MAT3X4: TypeId = 16;
pub(crate) const TYPE_MAT4X3: TypeId = 17;
pub(crate) const TYPE_MAT4X4: TypeId = 18;
pub(crate) const TYPE_BOOL: TypeId = 19;
pub(crate) const TYPE_STRING: TypeId = 20;
pub(crate) const TYPE_PATH: TypeId = 21;
pub(crate) const TYPE_FIXEDSTRING: TypeId = 22;
pub(crate) const TYPE_LSSTRING: TypeId = 23;
pub(crate) const TYPE_UINT64: TypeId = 24;
pub(crate) const TYPE_SCRATCHBUFFER: TypeId = 25;
pub(crate) const TYPE_OLD_INT64: TypeId = 26;
pub(crate) const TYPE_INT8: TypeId = 27;
pub(crate) const TYPE_TRANSLATEDSTRING: TypeId = 28;
pub(crate) const TYPE_WSTRING: TypeId = 29;
pub(crate) const TYPE_LSWSTRING: TypeId = 30;
pub(crate) const TYPE_GUID: TypeId = 31;
pub(crate) const TYPE_INT64: TypeId = 32;
pub(crate) const TYPE_TRANSLATEDFSSTRING: TypeId = 33;

/// Get the human-readable name for a type ID
#[must_use]
pub fn get_type_name(type_id: TypeId) -> &'static str {
    match type_id {
        TYPE_NONE => "None",
        TYPE_UINT8 => "uint8",
        TYPE_INT16 => "int16",
        TYPE_UINT16 => "uint16",
        TYPE_INT32 => "int32",
        TYPE_UINT32 => "uint32",
        TYPE_FLOAT => "float",
        TYPE_DOUBLE => "double",
        TYPE_IVEC2 => "ivec2",
        TYPE_IVEC3 => "ivec3",
        TYPE_IVEC4 => "ivec4",
        TYPE_FVEC2 => "fvec2",
        TYPE_FVEC3 => "fvec3",
        TYPE_FVEC4 => "fvec4",
        TYPE_MAT2X2 => "mat2x2",
        TYPE_MAT3X3 => "mat3x3",
        TYPE_MAT3X4 => "mat3x4",
        TYPE_MAT4X3 => "mat4x3",
        TYPE_MAT4X4 => "mat4x4",
        TYPE_BOOL => "bool",
        TYPE_STRING => "string",
        TYPE_PATH => "path",
        TYPE_FIXEDSTRING => "FixedString",
        TYPE_LSSTRING => "LSString",
        TYPE_UINT64 => "uint64",
        TYPE_SCRATCHBUFFER => "ScratchBuffer",
        TYPE_OLD_INT64 => "old_int64",
        TYPE_INT8 => "int8",
        TYPE_TRANSLATEDSTRING => "TranslatedString",
        TYPE_WSTRING => "WString",
        TYPE_LSWSTRING => "LSWString",
        TYPE_GUID => "guid",
        TYPE_INT64 => "int64",
        TYPE_TRANSLATEDFSSTRING => "TranslatedFSString",
        _ => "Unknown",
    }
}

/// Convert type name string to type ID
#[must_use]
pub fn type_name_to_id(type_name: &str) -> TypeId {
    match type_name {
        "None" => TYPE_NONE,
        "uint8" | "Byte" => TYPE_UINT8,
        "int16" | "Short" => TYPE_INT16,
        "uint16" | "UShort" => TYPE_UINT16,
        "int32" | "Int" => TYPE_INT32,
        "uint32" | "UInt" => TYPE_UINT32,
        "float" | "Float" => TYPE_FLOAT,
        "double" | "Double" => TYPE_DOUBLE,
        "ivec2" | "IVec2" => TYPE_IVEC2,
        "ivec3" | "IVec3" => TYPE_IVEC3,
        "ivec4" | "IVec4" => TYPE_IVEC4,
        "fvec2" | "Vec2" => TYPE_FVEC2,
        "fvec3" | "Vec3" => TYPE_FVEC3,
        "fvec4" | "Vec4" => TYPE_FVEC4,
        "mat2x2" | "Mat2" => TYPE_MAT2X2,
        "mat3x3" | "Mat3" => TYPE_MAT3X3,
        "mat3x4" | "Mat3x4" => TYPE_MAT3X4,
        "mat4x3" | "Mat4x3" => TYPE_MAT4X3,
        "mat4x4" | "Mat4" => TYPE_MAT4X4,
        "bool" | "Bool" => TYPE_BOOL,
        "string" | "String" => TYPE_STRING,
        "path" | "Path" => TYPE_PATH,
        "FixedString" => TYPE_FIXEDSTRING,
        "LSString" => TYPE_LSSTRING,
        "uint64" | "ULongLong" => TYPE_UINT64,
        "ScratchBuffer" => TYPE_SCRATCHBUFFER,
        "old_int64" | "Long" => TYPE_OLD_INT64,
        "int8" | "Int8" => TYPE_INT8,
        "TranslatedString" => TYPE_TRANSLATEDSTRING,
        "WString" => TYPE_WSTRING,
        "LSWString" => TYPE_LSWSTRING,
        "guid" | "UUID" => TYPE_GUID,
        "int64" | "Int64" => TYPE_INT64,
        "TranslatedFSString" => TYPE_TRANSLATEDFSSTRING,
        _ => TYPE_NONE,
    }
}

/// Check if a type is numeric
#[must_use]
pub fn is_numeric(type_id: TypeId) -> bool {
    matches!(type_id,
        TYPE_UINT8 | TYPE_INT16 | TYPE_UINT16 | TYPE_INT32 | TYPE_UINT32 |
        TYPE_FLOAT | TYPE_DOUBLE | TYPE_UINT64 | TYPE_OLD_INT64 | TYPE_INT8 | TYPE_INT64
    )
}

/// Get column count for vector/matrix types
#[must_use]
pub fn get_columns(type_id: TypeId) -> Option<usize> {
    match type_id {
        TYPE_IVEC2 | TYPE_FVEC2 | TYPE_MAT2X2 => Some(2),
        TYPE_IVEC3 | TYPE_FVEC3 | TYPE_MAT3X3 | TYPE_MAT4X3 => Some(3),
        TYPE_IVEC4 | TYPE_FVEC4 | TYPE_MAT3X4 | TYPE_MAT4X4 => Some(4),
        _ => None,
    }
}

/// Get row count for matrix types
#[must_use]
pub fn get_rows(type_id: TypeId) -> Option<usize> {
    match type_id {
        TYPE_IVEC2 | TYPE_IVEC3 | TYPE_IVEC4 | TYPE_FVEC2 | TYPE_FVEC3 | TYPE_FVEC4 => Some(1),
        TYPE_MAT2X2 => Some(2),
        TYPE_MAT3X3 | TYPE_MAT3X4 => Some(3),
        TYPE_MAT4X3 | TYPE_MAT4X4 => Some(4),
        _ => None,
    }
}

// ============================================================================
// SERIALIZATION: String → Bytes (for LSX → LSF)
// ============================================================================

/// Serialize a value to bytes based on type ID
///
/// # Errors
/// Returns an error if serialization fails for the given type.
pub fn serialize_value(buffer: &mut Vec<u8>, type_id: TypeId, value_str: &str) -> Result<usize> {
    let start = buffer.len();

    match type_id {
        // String types (null-terminated)
        TYPE_STRING | TYPE_PATH | TYPE_FIXEDSTRING | TYPE_LSSTRING | TYPE_WSTRING | TYPE_LSWSTRING => {
            buffer.extend_from_slice(value_str.as_bytes());
            buffer.push(0); // null terminator
        }
        // Bool
        TYPE_BOOL => {
            let val = match value_str {
                "True" | "true" | "1" => 1u8,
                _ => 0u8,
            };
            buffer.push(val);
        }
        // Integer types
        TYPE_NONE => {} // None type, no value
        TYPE_UINT8 | TYPE_INT8 => buffer.push(value_str.parse().unwrap_or(0)),
        TYPE_INT16 => buffer.write_i16::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        TYPE_UINT16 => buffer.write_u16::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        TYPE_INT32 => buffer.write_i32::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        TYPE_UINT32 => buffer.write_u32::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        TYPE_UINT64 => buffer.write_u64::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        TYPE_OLD_INT64 | TYPE_INT64 => buffer.write_i64::<LittleEndian>(value_str.parse().unwrap_or(0))?,
        // Float types
        TYPE_FLOAT => buffer.write_f32::<LittleEndian>(value_str.parse().unwrap_or(0.0))?,
        TYPE_DOUBLE => buffer.write_f64::<LittleEndian>(value_str.parse().unwrap_or(0.0))?,
        // Vector types (space-separated values)
        TYPE_IVEC2 => serialize_ivec(buffer, value_str, 2)?,
        TYPE_IVEC3 => serialize_ivec(buffer, value_str, 3)?,
        TYPE_IVEC4 => serialize_ivec(buffer, value_str, 4)?,
        TYPE_FVEC2 => serialize_fvec(buffer, value_str, 2)?,
        TYPE_FVEC3 => serialize_fvec(buffer, value_str, 3)?,
        TYPE_FVEC4 => serialize_fvec(buffer, value_str, 4)?,
        // Matrix types
        TYPE_MAT2X2 | TYPE_MAT3X3 | TYPE_MAT3X4 | TYPE_MAT4X3 | TYPE_MAT4X4 => {
            serialize_matrix(buffer, value_str)?;
        }
        // UUID
        TYPE_GUID => serialize_uuid(buffer, value_str)?,
        // Binary types
        TYPE_SCRATCHBUFFER => {
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

/// Serialize `TranslatedString`
///
/// # Errors
/// Returns an error if serialization fails.
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
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
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
///
/// # Errors
/// Returns an error if deserialization fails for the given type.
pub fn extract_value(values: &[u8], offset: usize, length: usize, type_id: TypeId) -> Result<String> {
    if offset + length > values.len() {
        return Ok(String::new());
    }

    let bytes = &values[offset..offset + length];

    Ok(match type_id {
        // String types (null-terminated)
        TYPE_STRING | TYPE_PATH | TYPE_FIXEDSTRING | TYPE_LSSTRING | TYPE_WSTRING | TYPE_LSWSTRING => {
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..end]).into_owned()
        }
        // Bool
        TYPE_BOOL => if bytes.first() == Some(&1) { "True" } else { "False" }.to_string(),
        // Integer types
        TYPE_NONE => String::new(),
        TYPE_UINT8 | TYPE_INT8 => bytes.first().map(std::string::ToString::to_string).unwrap_or_default(),
        TYPE_INT16 => i16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_UINT16 => u16::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_INT32 => i32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_UINT32 => u32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_UINT64 => u64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_OLD_INT64 | TYPE_INT64 => i64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Float types
        TYPE_FLOAT => f32::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        TYPE_DOUBLE => f64::from_le_bytes(bytes.try_into().unwrap_or_default()).to_string(),
        // Vector types
        TYPE_IVEC2 => format_ivec(bytes, 2),
        TYPE_IVEC3 => format_ivec(bytes, 3),
        TYPE_IVEC4 => format_ivec(bytes, 4),
        TYPE_FVEC2 => format_fvec(bytes, 2),
        TYPE_FVEC3 => format_fvec(bytes, 3),
        TYPE_FVEC4 => format_fvec(bytes, 4),
        // Matrix types
        TYPE_MAT2X2 | TYPE_MAT3X3 | TYPE_MAT3X4 | TYPE_MAT4X3 | TYPE_MAT4X4 => format_matrix(bytes),
        // UUID
        TYPE_GUID => format_uuid(bytes),
        // Binary types
        TYPE_SCRATCHBUFFER => BASE64.encode(bytes),
        // Unknown
        _ => {
            let byte_list: Vec<String> = bytes.iter().map(std::string::ToString::to_string).collect();
            format!("[{}]", byte_list.join(", "))
        }
    })
}

/// Format integer vector (space-separated)
#[must_use] 
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
#[must_use] 
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
#[must_use] 
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
#[must_use] 
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

/// Extract `TranslatedString` (handle, version, optional value)
///
/// # Errors
/// Returns an error if the bytes cannot be read as a translated string.
pub fn extract_translated_string(values: &[u8], offset: usize, length: usize) -> Result<(String, u16, Option<String>)> {
    if offset + length > values.len() {
        return Ok((String::new(), 0, None));
    }
    
    let bytes = &values[offset..offset + length];
    let mut cursor = Cursor::new(bytes);
    
    let version = cursor.read_u16::<LittleEndian>()?;
    #[allow(clippy::cast_sign_loss)]
    let handle_length = cursor.read_i32::<LittleEndian>()?.max(0) as usize;
    
    if handle_length == 0 {
        return Ok((String::new(), version, None));
    }
    
    let mut handle_bytes = vec![0u8; handle_length.saturating_sub(1)];
    cursor.read_exact(&mut handle_bytes)?;
    let _ = cursor.read_u8()?; // null terminator
    
    let handle = String::from_utf8_lossy(&handle_bytes).into_owned();
    
    Ok((handle, version, None))
}