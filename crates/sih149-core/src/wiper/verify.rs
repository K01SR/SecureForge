//! Post-wipe verification engine.
//!
//! Reads random sector samples after erasure and calculates
//! Shannon entropy to confirm successful sanitization:
//! - Entropy ≈ 0.0: zero-filled (Clear)
//! - Entropy ≈ 8.0 (uniform): random-filled or crypto-erased (Purge)
//! - Entropy between: potential residual data (FAIL)
//!
//! Logs bad sectors (EIO errors) for NIST compliance reporting.
