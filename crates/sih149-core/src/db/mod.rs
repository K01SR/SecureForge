//! SQLite case management database.
//!
//! Stores all operational data in a single `.db` file per case:
//! - Scan sessions and drive metadata
//! - Recovered file records with confidence scores
//! - Erasure operation logs
//! - Hash chain entries
//! - Plugin-detected custom file types

pub mod queries;
