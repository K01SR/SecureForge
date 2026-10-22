/// Calculates Shannon entropy of a given byte slice.
/// Entropy is a measure of randomness, returning a value between 0.0 and 8.0.
/// Higher values (e.g., > 7.5) indicate compressed or encrypted data.
pub fn calculate_shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut frequencies = [0usize; 256];
    for &byte in data {
        frequencies[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &freq in &frequencies {
        if freq > 0 {
            let p = freq as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Helper to determine if a block of data is likely encrypted or highly compressed.
pub fn is_high_entropy(data: &[u8], threshold: f64) -> bool {
    calculate_shannon_entropy(data) >= threshold
}
