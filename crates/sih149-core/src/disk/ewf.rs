//! Expert Witness Format (E01) image reader.
//!
//! Provides read access to `.E01` forensic images via FFI bindings
//! to `libewf` (LGPL-3.0, dynamically linked).
//!
//! E01 images include built-in compression (zlib), checksumming
//! (CRC32 per chunk + MD5/SHA1 full image), and case metadata
//! (examiner name, case number, timestamps).
