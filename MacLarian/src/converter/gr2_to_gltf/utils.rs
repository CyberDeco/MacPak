//! Utility functions for data conversion.

/// Convert half-float (f16) to f32.
pub fn half_to_f32(bits: u16) -> f32 {
    let sign = ((bits >> 15) & 1) as u32;
    let exp = ((bits >> 10) & 0x1F) as i32;
    let mant = (bits & 0x3FF) as u32;

    if exp == 0 {
        if mant == 0 {
            return f32::from_bits(sign << 31);
        }
        // Denormalized
        let mut m = mant;
        let mut e = 1i32;
        while (m & 0x400) == 0 {
            m <<= 1;
            e -= 1;
        }
        let exp32 = (127 - 15 + e) as u32;
        let mant32 = (m & 0x3FF) << 13;
        return f32::from_bits((sign << 31) | (exp32 << 23) | mant32);
    } else if exp == 31 {
        let exp32 = 255u32;
        let mant32 = mant << 13;
        return f32::from_bits((sign << 31) | (exp32 << 23) | mant32);
    }

    let exp32 = (exp + 127 - 15) as u32;
    let mant32 = mant << 13;
    f32::from_bits((sign << 31) | (exp32 << 23) | mant32)
}

/// Decode QTangent quaternion to normal and tangent vectors.
pub fn decode_qtangent(qt: &[i16; 4]) -> ([f32; 3], [f32; 4]) {
    let q: [f32; 4] = [
        qt[0] as f32 / 32767.0,
        qt[1] as f32 / 32767.0,
        qt[2] as f32 / 32767.0,
        qt[3] as f32 / 32767.0,
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
