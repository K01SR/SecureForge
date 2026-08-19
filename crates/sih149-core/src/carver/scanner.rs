//! Multi-threaded sector scanner.
//!
//! Uses `rayon` to parallelize sector reading across multiple threads.
//! Each thread processes 64KB chunks, checking against the signature
//! database and calculating per-sector entropy values.
//!
//! Bad sectors (EIO errors) are logged and skipped gracefully.
