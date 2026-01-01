//! UUID generation utilities for BG3 modding

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum UuidFormat {
    Standard,    // 8-4-4-4-12
    Compact,     // No dashes
    Larian,      // Larian's format (h prefix + specific format)
}

/// Generate a new UUID v4 in the specified format
pub fn generate_uuid(format: UuidFormat) -> String {
    let uuid = uuid::Uuid::new_v4();

    match format {
        UuidFormat::Standard => uuid.to_string(),
        UuidFormat::Compact => uuid.simple().to_string(),
        UuidFormat::Larian => {
            let simple = uuid.simple().to_string();
            format!(
                "h{}g{}g{}g{}g{}",
                &simple[0..8],
                &simple[8..12],
                &simple[12..16],
                &simple[16..20],
                &simple[20..32]
            )
        }
    }
}
