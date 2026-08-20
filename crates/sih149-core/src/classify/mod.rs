//! Recovered file classification and duplicate detection.
//!
//! After carving, files are automatically categorized into:
//! Documents, Media, Archives, Databases, System, Unknown.
//!
//! Image files are deduplicated using perceptual hashing (dHash)
//! to group visually identical files regardless of compression.

pub mod dhash;
