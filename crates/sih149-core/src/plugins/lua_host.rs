//! Embedded Lua 5.4 virtual machine for plugin scripting.
//!
//! Uses the `mlua` crate to embed a sandboxed Lua interpreter.
//! Plugins define file signatures with optional `validate()` functions
//! that receive raw file bytes and return true/false.
//!
//! The Lua environment is sandboxed: no filesystem access, no network,
//! no OS commands. Only the `signature {}` function and byte-level
//! string operations are exposed.
