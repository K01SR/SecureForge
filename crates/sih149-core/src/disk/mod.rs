//! Unified disk source abstraction.
//!
//! Provides a common [`DiskSource`] trait that abstracts over:
//! - Raw Linux block devices (`/dev/sdX`, `/dev/nvmeXnY`)
//! - Raw disk image files (`.dd`, `.raw`, `.img`)
//! - Expert Witness Format images (`.E01`) via libewf FFI
//!
//! All implementations provide sector-aligned read access with
//! configurable block sizes.

pub mod block_device;
pub mod ewf;
pub mod raw_image;
