use crate::error::Result;

/// Validates PNG structure from a buffer.
pub fn validate_png(data: &[u8]) -> Result<bool> {
    if data.len() < 8 {
        return Ok(false);
    }

    // PNG Magic Header
    const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if &data[0..8] != PNG_HEADER {
        return Ok(false);
    }

    // Look for IEND chunk (end of PNG)
    const IEND: [u8; 4] = [0x49, 0x45, 0x4E, 0x44]; // "IEND"

    let mut offset = 8;
    while offset + 8 <= data.len() {
        let length = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type = &data[offset + 4..offset + 8];

        if chunk_type == IEND {
            return Ok(true);
        }

        // chunk length + chunk type (4) + data (length) + CRC (4)
        offset += 4 + 4 + length + 4;
    }

    Ok(false)
}
