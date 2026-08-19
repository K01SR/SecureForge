//! Secure file and folder erasure.
//!
//! For selective deletion of individual files:
//! 1. Overwrite file content with selected pattern
//! 2. Scrub filesystem metadata (delegated to `metadata` submodule)
//! 3. Rename file 10+ times to random strings
//! 4. Wipe slack space (tail of last allocated cluster)
//! 5. Unlink file
//!
//! Supports batch operations on directory trees and glob patterns.
