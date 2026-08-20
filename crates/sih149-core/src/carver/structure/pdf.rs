//! PDF structure validator.
//!
//! Parses PDF document structure:
//! - Header: %PDF-1.x or %PDF-2.0
//! - Cross-reference table (xref)
//! - Trailer dictionary
//! - Footer: %%EOF
//!
//! Validates object references and stream lengths.
