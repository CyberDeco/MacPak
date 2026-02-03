//! Shared utilities for GR2 ↔ glTF conversions
//!
//! Contains common data conversion functions used by both directions.
//!
//!

#![allow(clippy::cast_possible_truncation, clippy::if_same_then_else)]

use glam::{Mat3, Quat, Vec3};
use half::f16;

/// Convert half-float (f16) to f32.
#[must_use]
pub fn half_to_f32(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}

/// Convert f32 to half-float (f16).
#[must_use]
pub fn f32_to_half(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

/// Decode `QTangent` quaternion to normal and tangent vectors.
#[must_use]
pub fn decode_qtangent(qt: &[i16; 4]) -> ([f32; 3], [f32; 4]) {
    let q: [f32; 4] = [
        f32::from(qt[0]) / 32767.0,
        f32::from(qt[1]) / 32767.0,
        f32::from(qt[2]) / 32767.0,
        f32::from(qt[3]) / 32767.0,
    ];

    // Extract handedness from sign of W before normalizing
    let handedness = if q[3] < 0.0 { -1.0 } else { 1.0 };

    // If W is negative, the quaternion was negated during encoding to store handedness.
    // Un-negate it to get the canonical quaternion (W >= 0) for axis extraction.
    // This is necessary because negating all four components affects the tangent
    // calculation (the normal is invariant under quaternion negation).
    let (qx, qy, qz, qw) = if q[3] < 0.0 {
        (-q[0], -q[1], -q[2], -q[3])
    } else {
        (q[0], q[1], q[2], q[3])
    };

    // Normal (Z axis of rotation matrix)
    let normal = [
        2.0 * (qx * qz + qw * qy),
        2.0 * (qy * qz - qw * qx),
        1.0 - 2.0 * (qx * qx + qy * qy),
    ];

    // Tangent (X axis) - using canonical quaternion (W >= 0)
    let tangent = [
        1.0 - 2.0 * (qy * qy + qz * qz),
        2.0 * (qx * qy + qw * qz),
        2.0 * (qx * qz - qw * qy),
        handedness,
    ];

    (normal, tangent)
}

/// Encode normal and tangent vectors to `QTangent` quaternion.
/// This is the inverse of `decode_qtangent`.
#[must_use]
pub fn encode_qtangent(normal: &[f32; 3], tangent: &[f32; 4]) -> [i16; 4] {
    // Normalize inputs
    let n = Vec3::from_array(*normal).normalize_or_zero();
    let t = Vec3::new(tangent[0], tangent[1], tangent[2]).normalize_or_zero();
    let handedness = tangent[3];

    // Compute binormal and ensure valid rotation matrix (determinant = +1).
    // Handedness is encoded separately in the sign of W.
    let b = n.cross(t);

    // Build rotation matrix from TBN (tangent, binormal, normal as columns)
    let mat = Mat3::from_cols(t, b, n);

    // Convert rotation matrix to quaternion
    let mut quat = Quat::from_mat3(&mat);

    // Normalize quaternion
    quat = quat.normalize();

    // Canonicalize: ensure W >= 0 first
    if quat.w < 0.0 {
        quat = -quat;
    }

    // Now encode handedness: if negative, flip to make W negative
    if handedness < 0.0 {
        quat = -quat;
    }

    // Scale to i16 range [-32767, 32767]
    [
        (quat.x * 32767.0).clamp(-32767.0, 32767.0) as i16,
        (quat.y * 32767.0).clamp(-32767.0, 32767.0) as i16,
        (quat.z * 32767.0).clamp(-32767.0, 32767.0) as i16,
        (quat.w * 32767.0).clamp(-32767.0, 32767.0) as i16,
    ]
}
