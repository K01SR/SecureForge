use crate::error::Result;

/// Validates JPEG structure from a buffer.
/// Looks for SOI (Start of Image) and EOI (End of Image) markers.
pub fn validate_jpeg(data: &[u8]) -> Result<bool> {
    if data.len() < 4 {
        return Ok(false);
    }

    // Check SOI
    if data[0] != 0xFF || data[1] != 0xD8 {
        return Ok(false);
    }

    // Scan for EOI
    for i in 2..data.len() - 1 {
        if data[i] == 0xFF && data[i + 1] == 0xD9 {
            return Ok(true);
        }
    }

    Ok(false)
}
