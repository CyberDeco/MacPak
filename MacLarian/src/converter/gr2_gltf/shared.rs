//! Shared utilities for GR2 ↔ glTF conversions
//!
//! Contains common data conversion functions used by both directions.
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

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

    let (qx, qy, qz, qw) = (q[0], q[1], q[2], q[3]);

    // Normal (Z axis of rotation matrix)
    let normal = [
        2.0 * (qx * qz + qw * qy),
        2.0 * (qy * qz - qw * qx),
        1.0 - 2.0 * (qx * qx + qy * qy),
    ];

    // Tangent (X axis) with handedness stored in sign of w
    let handedness = if qw < 0.0 { -1.0 } else { 1.0 };
    let tangent = [
        1.0 - 2.0 * (qy * qy + qz * qz),
        2.0 * (qx * qy + qw.abs() * qz),
        2.0 * (qx * qz - qw.abs() * qy),
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

    // Compute binormal
    let b = n.cross(t) * handedness;

    // Build rotation matrix from TBN (tangent, binormal, normal as columns)
    let mat = Mat3::from_cols(t, b, n);

    // Convert rotation matrix to quaternion
    let mut quat = Quat::from_mat3(&mat);

    // Normalize quaternion
    quat = quat.normalize();

    // Encode handedness in sign of W
    // If handedness is negative, ensure W is negative
    if handedness < 0.0 && quat.w > 0.0 {
        quat = -quat;
    } else if handedness >= 0.0 && quat.w < 0.0 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_roundtrip() {
        let values = [0.0f32, 1.0, -1.0, 0.5, 2.0, 0.001, 100.0];
        for &v in &values {
            let half_bits = f32_to_half(v);
            let back = half_to_f32(half_bits);
            // Half precision has limited accuracy
            if v.abs() > 0.0001 && v.abs() < 65000.0 {
                assert!((v - back).abs() / v.abs() < 0.01, "Failed for {}: got {}", v, back);
            }
        }
    }

    #[test]
    fn test_qtangent_roundtrip() {
        let normal = [0.0f32, 0.0, 1.0];
        let tangent = [1.0f32, 0.0, 0.0, 1.0];

        let encoded = encode_qtangent(&normal, &tangent);
        let (decoded_normal, decoded_tangent) = decode_qtangent(&encoded);

        // Check normal is approximately correct
        for i in 0..3 {
            assert!((normal[i] - decoded_normal[i]).abs() < 0.01,
                "Normal mismatch at {}: {} vs {}", i, normal[i], decoded_normal[i]);
        }

        // Check tangent direction is approximately correct
        for i in 0..3 {
            assert!((tangent[i] - decoded_tangent[i]).abs() < 0.01,
                "Tangent mismatch at {}: {} vs {}", i, tangent[i], decoded_tangent[i]);
        }

        // Check handedness preserved
        assert_eq!(tangent[3].signum(), decoded_tangent[3].signum());
    }
}
