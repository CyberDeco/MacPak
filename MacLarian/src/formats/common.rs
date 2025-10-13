//! Common types shared across all formats

use serde::{Deserialize, Serialize};

/// Attribute value types supported by Larian formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    None,
    Byte(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat2([f32; 4]),
    Mat3([f32; 9]),
    Mat3x4([f32; 12]),
    Mat4x3([f32; 12]),
    Mat4([f32; 16]),
    Bool(bool),
    String(String),
    Path(String),
    FixedString(String),
    LSString(String),
    WString(String),
    LSWString(String),
    UUID(String),
    Int64(i64),
    UInt64(u64),
    ScratchBuffer(Vec<u8>),
    TranslatedString(TranslatedString),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslatedString {
    pub value: String,
    pub handle: String,
}

impl AttributeValue {
    /// Convert type ID to AttributeValue enum variant name
    pub fn from_type_id(type_id: u32) -> &'static str {
        match type_id {
            0 => "None",
            1 => "Byte",
            2 => "Short",
            3 => "UShort",
            4 => "Int",
            5 => "UInt",
            6 => "Float",
            7 => "Double",
            8 => "IVec2",
            9 => "IVec3",
            10 => "IVec4",
            11 => "Vec2",
            12 => "Vec3",
            13 => "Vec4",
            14 => "Mat2",
            15 => "Mat3",
            16 => "Mat3x4",
            17 => "Mat4x3",
            18 => "Mat4",
            19 => "Bool",
            20 => "String",
            21 => "Path",
            22 => "FixedString",
            23 => "LSString",
            24 => "UInt64",
            25 => "ScratchBuffer",
            26 => "Int64",
            27 => "Int8",
            28 => "TranslatedString",
            29 => "WString",
            30 => "LSWString",
            31 => "UUID",
            32 => "Int64",
            _ => "Unknown",
        }
    }
    
    /// Convert type name to type ID
    pub fn type_name_to_id(name: &str) -> Option<u32> {
        match name {
            "None" => Some(0),
            "byte" | "Byte" => Some(1),
            "short" | "Short" => Some(2),
            "ushort" | "UShort" => Some(3),
            "int" | "Int" => Some(4),
            "uint" | "UInt" => Some(5),
            "float" | "Float" => Some(6),
            "double" | "Double" => Some(7),
            "ivec2" | "IVec2" => Some(8),
            "ivec3" | "IVec3" => Some(9),
            "ivec4" | "IVec4" => Some(10),
            "vec2" | "Vec2" => Some(11),
            "vec3" | "Vec3" => Some(12),
            "vec4" | "Vec4" => Some(13),
            "mat2" | "Mat2" => Some(14),
            "mat3" | "Mat3" => Some(15),
            "mat3x4" | "Mat3x4" => Some(16),
            "mat4x3" | "Mat4x3" => Some(17),
            "mat4" | "Mat4" => Some(18),
            "bool" | "Bool" => Some(19),
            "string" | "String" => Some(20),
            "path" | "Path" => Some(21),
            "FixedString" => Some(22),
            "LSString" => Some(23),
            "uint64" | "UInt64" => Some(24),
            "ScratchBuffer" => Some(25),
            "long" | "int64" | "Int64" => Some(26),
            "TranslatedString" => Some(28),
            "WString" => Some(29),
            "LSWString" => Some(30),
            "uuid" | "UUID" => Some(31),
            _ => None,
        }
    }
}
