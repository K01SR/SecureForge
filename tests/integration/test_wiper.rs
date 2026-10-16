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
