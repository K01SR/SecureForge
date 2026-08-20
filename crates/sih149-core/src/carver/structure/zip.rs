//! ZIP archive structure validator.
//!
//! Parses ZIP internal structure:
//! - Local file headers (PK\x03\x04)
//! - File data entries
//! - Central directory (PK\x01\x02)
//! - End of central directory record (PK\x05\x06)
//!
//! Also validates DOCX/XLSX/PPTX (which are ZIP containers).
