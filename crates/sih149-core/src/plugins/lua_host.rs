use mlua::prelude::*;
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
        let lua = Lua::new();
        
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
        
        let lua = Lua::new();
        let globals = lua.globals();
        
        let signature_func = lua.create_function(move |_lua, table: mlua::Table| {
            Ok(table) // just return the table during validation
        }).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        globals.set("signature", signature_func).map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let chunk = lua.load(&plugin.script_source);
        let result: mlua::Table = chunk.eval().map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let validate_func: mlua::Function = result.get("validate")
            .map_err(|_| CoreError::Parse("validate function not found".to_string()))?;
            
        let data_str = String::from_utf8_lossy(data).into_owned();
        let is_valid: bool = validate_func.call(data_str)
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
