//! SQLite database structure validator.
//!
//! Validates SQLite file format:
//! - Header: "SQLite format 3\000" (16 bytes)
//! - Page size at offset 16-17 (big-endian, power of 2, 512-65536)
//! - File format versions at offset 18-19
//! - B-tree page headers
//!
//! Critical for recovering WhatsApp, Signal, browser history databases.
