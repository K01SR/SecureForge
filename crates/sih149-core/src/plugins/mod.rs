//! User-extensible plugin system.
//!
//! Three-tier signature hierarchy:
//! 1. **Rust built-in**: Highest performance, compiled into binary
//! 2. **TOML definitions**: Simple header/footer, user-editable
//! 3. **Lua scripts**: Full programmability with custom validation
//!
//! Plugins are hot-reloadable: new files in `plugins/` are detected
//! without restarting the application.

pub mod lua_host;
pub mod toml_loader;
