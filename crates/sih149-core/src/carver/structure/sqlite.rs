use crate::error::Result;

/// Validates SQLite database structure from a buffer.
pub fn validate_sqlite(data: &[u8]) -> Result<bool> {
    // SQLite header is 100 bytes long
    if data.len() < 100 {
        return Ok(false);
    }

    // SQLite 3 magic header string: "SQLite format 3\0"
    let magic = b"SQLite format 3\0";
    if &data[0..16] != magic {
        return Ok(false);
    }

    // Page size is at offset 16 (2 bytes, big-endian)
    let page_size_bytes: [u8; 2] = data[16..18].try_into().unwrap();
    let mut page_size = u16::from_be_bytes(page_size_bytes) as u32;
    if page_size == 1 {
        page_size = 65536;
    }
    if page_size == 0 {
        return Ok(false);
    }

    // The database size must be a multiple of the page size
    if (data.len() as u32) % page_size != 0 {
        // Since we are carving, we might have over-carved, 
        // so we just check if it's large enough to contain at least 1 page.
        if data.len() as u32 >= page_size {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}
