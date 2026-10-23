use crate::error::Result;

/// Validates PDF structure from a buffer.
pub fn validate_pdf(data: &[u8]) -> Result<bool> {
    if data.len() < 9 {
        return Ok(false);
    }

    // PDF Magic Header: %PDF-
    if &data[0..5] != b"%PDF-" {
        return Ok(false);
    }

    // PDF Magic Footer: %%EOF
    // We scan the last 1024 bytes for it, as it might have trailing whitespace
    let eof_sig = b"%%EOF";
    
    let scan_start = if data.len() > 1024 { data.len() - 1024 } else { 0 };
    
    for i in (scan_start..=data.len() - eof_sig.len()).rev() {
        if &data[i..i+eof_sig.len()] == eof_sig {
            return Ok(true);
        }
    }

    Ok(false)
}
