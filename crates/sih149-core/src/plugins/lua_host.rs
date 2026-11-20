use mlua::{prelude::*, LuaOptions, StdLib};
use std::path::Path;
use std::fs;
use crate::error::CoreError;

#[derive(Debug, Clone)]
pub struct PluginSignature {
    pub name: String,
    pub category: String,
    pub header: Vec<u8>,
    pub footer: Option<Vec<u8>>,
    pub max_size: u64,
    pub has_validator: bool,
}

pub struct LuaPlugin {
    pub signature: PluginSignature,
    script_source: String,
}

pub struct LuaPluginHost {
    plugins: Vec<LuaPlugin>,
}

impl LuaPluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn load_script(&mut self, script_path: &Path) -> Result<(), CoreError> {
        let script_source = fs::read_to_string(script_path).map_err(CoreError::Io)?;
        // SAFETY-CRITICAL: plugins are untrusted third-party scripts.
        // Only load string/table/math — NOT io, os, or package (which allow
        // filesystem access, process execution, and loading native libs).
        let safe_libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH;
        let lua = Lua::new_with(safe_libs, LuaOptions::default())
            .map_err(|e| CoreError::Parse(format!("Failed to init sandboxed Lua: {e}")))?;
        
        let globals = lua.globals();
        
        // Define signature function to capture data
        let _sig_data = lua.create_table().map_err(|e| CoreError::Parse(e.to_string()))?;
        
        // Let's create a proxy table to intercept the 'signature' call
        // Actually, mlua allows creating a Rust function to bind to "signature"
        
        let signature_func = lua.create_function(move |lua, table: mlua::Table| {
            let name: String = table.get("name")?;
            let category: String = table.get("category")?;
            let header_hex: String = table.get("header")?;
            let footer_hex: Option<String> = table.get("footer")?;
            let max_size: u64 = table.get("max_size")?;
            let has_validate: bool = table.contains_key("validate")?;
            
            // Convert hex to bytes
            let header = (0..header_hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&header_hex[i..i + 2], 16))
                .collect::<Result<Vec<u8>, _>>()
                .map_err(|e| mlua::Error::RuntimeError(format!("Invalid hex: {}", e)))?;
                
            let footer = match footer_hex {
                Some(hex) => Some((0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                    .collect::<Result<Vec<u8>, _>>()
                    .map_err(|e| mlua::Error::RuntimeError(format!("Invalid hex: {}", e)))?),
                None => None,
            };
            
            // We just need to return these somehow. We can return them as a table
            let res = lua.create_table()?;
            res.set("name", name)?;
            res.set("category", category)?;
            res.set("header", header)?;
            if let Some(f) = footer {
                res.set("footer", f)?;
            }
            res.set("max_size", max_size)?;
            res.set("has_validator", has_validate)?;
            Ok(res)
        }).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        globals.set("signature", signature_func).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        // Evaluate the script
        let chunk = lua.load(&script_source);
        let result: mlua::Table = chunk.eval().map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let name: String = result.get("name").unwrap_or_default();
        let category: String = result.get("category").unwrap_or_default();
        let header: Vec<u8> = result.get("header").unwrap_or_default();
        let footer: Option<Vec<u8>> = result.get("footer").ok();
        let max_size: u64 = result.get("max_size").unwrap_or(0);
        let has_validator: bool = result.get("has_validator").unwrap_or(false);
        
        let plugin_sig = PluginSignature {
            name,
            category,
            header,
            footer,
            max_size,
            has_validator,
        };
        
        self.plugins.push(LuaPlugin {
            signature: plugin_sig,
            script_source,
        });
        
        Ok(())
    }
    
    pub fn loaded_signatures(&self) -> Vec<&PluginSignature> {
        self.plugins.iter().map(|p| &p.signature).collect()
    }
    
    pub fn validate(&self, plugin_name: &str, data: &[u8]) -> Result<bool, CoreError> {
        let plugin = self.plugins.iter().find(|p| p.signature.name == plugin_name)
            .ok_or_else(|| CoreError::Parse(format!("Plugin not found: {}", plugin_name)))?;
            
        if !plugin.signature.has_validator {
            return Ok(true);
        }
        
        let safe_libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH;
        let lua = Lua::new_with(safe_libs, LuaOptions::default())
            .map_err(|e| CoreError::Parse(format!("Failed to init sandboxed Lua: {e}")))?;

        // Set instruction hook limit (100k instructions) to prevent infinite loop DoS
        lua.set_hook(
            mlua::HookTriggers::default().every_nth_instruction(100_000),
            |_lua, _debug| {
                Err(mlua::Error::RuntimeError(
                    "Lua execution instruction limit exceeded (100,000 instructions) — execution terminated to prevent DoS".to_string()
                ))
            },
        );

        let globals = lua.globals();
        
        let signature_func = lua.create_function(move |_lua, table: mlua::Table| {
            Ok(table) // just return the table during validation
        }).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        globals.set("signature", signature_func).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let chunk = lua.load(&plugin.script_source);
        let result: mlua::Table = chunk.eval().map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let validate_func: mlua::Function = result.get("validate")
            .map_err(|_| CoreError::Parse("validate function not found".to_string()))?;
            
        // Pass raw bytes as a Lua string (Lua strings are byte arrays, not
        // UTF-8 text) — from_utf8_lossy previously corrupted binary magic
        // bytes (JPEG/PNG/ZIP headers) before the validator ever saw them.
        let lua_bytes = lua.create_string(data)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        let is_valid: bool = validate_func.call(lua_bytes)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
            
        Ok(is_valid)
    }
    
    pub fn load_directory(&mut self, dir: &Path) -> Result<usize, CoreError> {
        let mut count = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(CoreError::Io)? {
                let entry = entry.map_err(CoreError::Io)?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
                    if self.load_script(&path).is_ok() {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }
}
