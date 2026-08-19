//! Linux block device reader.
//!
//! Opens `/dev/sdX` or `/dev/nvmeXnY` devices with `O_RDONLY | O_DIRECT`
//! for forensic read operations, or `O_RDWR | O_DIRECT` for sanitization.
//!
//! Uses `ioctl` calls to query device geometry, SMART health, and
//! supported sanitization capabilities.
