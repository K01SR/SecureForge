//! Integration tests for the secure wiper
use sih149_core::wiper::patterns::*;
use sih149_core::wiper::verify::verify_wipe;
use std::fs;
use std::io::{Write, Read};

#[test]
fn test_zero_pattern() {
    let mut buf = vec![0xFF; 512];
    zero_fill(&mut buf);
    assert!(buf.iter().all(|&b| b == 0x00));
}

#[test]
fn test_random_pattern_entropy() {
    use sih149_core::carver::entropy::calculate_entropy;
    let mut buf = vec![0x00; 1024];
    random_fill(&mut buf);
    let entropy = calculate_entropy(&buf);
    assert!(entropy > 7.0, "Entropy should be high for random pattern");
}

#[test]
fn test_dod3_pass_count() {
    let passes = get_dod_3pass_sequence();
    assert_eq!(passes.len(), 3);
}
