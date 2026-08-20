//! TOML signature definition loader.
//!
//! Parses `plugins/signatures/*.toml` files containing simple
//! file type definitions with header bytes, optional footer bytes,
//! maximum file size, and category classification.
//!
//! TOML signatures are simpler but faster to define than Lua plugins.
//! They do not support custom validation logic.
