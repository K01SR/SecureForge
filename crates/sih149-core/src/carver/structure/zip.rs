use crate::error::Result;

/// Validates ZIP structure from a buffer.
pub fn validate_zip(data: &[u8]) -> Result<bool> {
    if data.len() < 22 { // Minimum size of End of Central Directory record
        return Ok(false);
    }

    // ZIP Magic Header (Local File Header)
    if data[0] != 0x50 || data[1] != 0x4B || data[2] != 0x03 || data[3] != 0x04 {
        return Ok(false);
    }

    // A complete ZIP file usually ends with the End of Central Directory (EOCD) signature
    // PK\x05\x06 followed by at least 18 bytes. We can just scan backwards for it.
    let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
    
    // Scan last 64KB max for EOCD
    let scan_start = if data.len() > 65535 + 22 { data.len() - 65535 - 22 } else { 0 };
    
    for i in (scan_start..=data.len() - 22).rev() {
        if &data[i..i+4] == eocd_sig {
            return Ok(true);
        }
    }

    Ok(false)
}
