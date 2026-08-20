//! Difference hash (dHash) for perceptual image deduplication.
//!
//! Computes a 64-bit hash by:
//! 1. Resizing image to 9x8 grayscale
//! 2. Comparing adjacent pixel brightness (left vs right)
//! 3. Encoding differences as bits
//!
//! Hamming distance between hashes indicates visual similarity.
//! Distance < 10 = likely duplicates.
