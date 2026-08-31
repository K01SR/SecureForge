/// Generates a buffer filled with zeros.
pub fn generate_zeros(size: usize) -> Vec<u8> {
    vec![0; size]
}

/// Generates a buffer filled with ones (0xFF).
pub fn generate_ones(size: usize) -> Vec<u8> {
    vec![0xFF; size]
}

/// Generates a buffer filled with pseudo-random bytes.
/// Note: In a real implementation, use a secure PRNG.
pub fn generate_random(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    // Simple fast weak random for demonstration (xorshift or similar)
    let mut state = 0x1234567890abcdefu64;
    for chunk in buf.chunks_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let bytes = state.to_le_bytes();
        for (i, b) in chunk.iter_mut().enumerate() {
            *b = bytes[i];
        }
    }
    buf
}

/// DoD 5220.22-M 3-pass pattern: Zeros, Ones, Random.
/// Returns the pattern function for the given pass (1-indexed).
pub fn get_dod_pattern(pass: u8) -> fn(usize) -> Vec<u8> {
    match pass {
        1 => generate_zeros,
        2 => generate_ones,
        _ => generate_random,
    }
}
