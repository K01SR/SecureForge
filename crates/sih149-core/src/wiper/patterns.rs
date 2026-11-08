use rand::RngCore;
use rand::rngs::OsRng;

/// Generates a buffer filled with zeros.
pub fn generate_zeros(size: usize) -> Vec<u8> {
    vec![0; size]
}

/// Generates a buffer filled with ones (0xFF).
pub fn generate_ones(size: usize) -> Vec<u8> {
    vec![0xFF; size]
}

/// Generates a buffer filled with cryptographically secure random bytes.
/// Uses the OS CSPRNG (getrandom/OsRng) — output differs on every call,
/// which is required for DoD 5220.22-M pass 3 to be meaningful.
pub fn generate_random(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    OsRng.fill_bytes(&mut buf);
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
