//! Prepared SQL statements for case database operations.
//!
//! All queries use parameterized statements to prevent SQL injection.
//! Includes queries for:
//! - Inserting/querying scan sessions
//! - Inserting/filtering recovered files by type, confidence, drive
//! - Appending and verifying hash chain integrity
//! - Aggregating statistics for reports
