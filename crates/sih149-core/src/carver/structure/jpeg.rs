//! JPEG structure validator.
//!
//! Parses JPEG marker segments:
//! - SOI (Start of Image): FF D8
//! - APP0/APP1 (JFIF/EXIF metadata)
//! - DQT (Quantization tables)
//! - SOF (Start of Frame)
//! - DHT (Huffman tables)
//! - SOS (Start of Scan)
//! - RST0-RST7 (Restart markers for fragmented carving)
//! - EOI (End of Image): FF D9
