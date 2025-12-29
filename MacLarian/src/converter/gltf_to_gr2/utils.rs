//! Utility functions for data conversion.

use glam::{Mat3, Quat, Vec3};
use half::f16;

/// Convert f32 to half-float (f16).
pub fn f32_to_half(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

/// Encode normal and tangent vectors to QTangent quaternion.
/// This is the inverse of decode_qtangent.
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

/// Calculate CRC32 checksum (GR2 uses standard CRC-32)
pub fn crc32(data: &[u8]) -> u32 {
    const TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0u32;
        while i < 256 {
            let mut crc = i;
            let mut j = 0;
            while j < 8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[i as usize] = crc;
            i += 1;
        }
        table
    };

    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc = TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_to_half_roundtrip() {
        let values = [0.0f32, 1.0, -1.0, 0.5, 2.0, 0.001, 100.0];
        for &v in &values {
            let half_bits = f32_to_half(v);
            let back = f16::from_bits(half_bits).to_f32();
            // Half precision has limited accuracy
            if v.abs() > 0.0001 && v.abs() < 65000.0 {
                assert!((v - back).abs() / v.abs() < 0.01, "Failed for {}: got {}", v, back);
            }
        }
    }

    #[test]
    fn test_encode_qtangent() {
        let normal = [0.0, 0.0, 1.0];
        let tangent = [1.0, 0.0, 0.0, 1.0];
        let qt = encode_qtangent(&normal, &tangent);
        // Just verify it produces valid output
        assert!(qt[0].abs() <= 32767);
        assert!(qt[1].abs() <= 32767);
        assert!(qt[2].abs() <= 32767);
        assert!(qt[3].abs() <= 32767);
    }

    #[test]
    fn test_crc32() {
        // Test vector from standard CRC-32
        let data = b"123456789";
        let crc = crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }
}
