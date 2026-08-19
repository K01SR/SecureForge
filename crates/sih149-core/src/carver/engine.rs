//! Core carving engine.
//!
//! Orchestrates the carving pipeline:
//! 1. Scanner reads sectors and matches signatures
//! 2. Matched regions are carved (contiguous extraction)
//! 3. Structure validators check internal file integrity
//! 4. Confidence scores are calculated
//! 5. Results are written to the output directory and database
