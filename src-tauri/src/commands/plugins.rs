use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use sih149_core::carver::entropy::calculate_shannon_entropy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginItem {
    pub name: String,
    pub category: String,
    pub plugin_type: String, // "TOML" | "Lua"
    pub extension: String,
    pub has_validator: bool,
    pub status: String,
    pub description: String,
}

#[tauri::command]
pub fn list_plugins() -> Result<Vec<PluginItem>, String> {
    let mut items = Vec::new();

    // 1. Read real TOML signatures from plugins/signatures
    let sig_dir = Path::new("plugins/signatures");
    if sig_dir.exists() {
        if let Ok(sigs) = sih149_core::plugins::toml_loader::load_signatures_from_dir(sig_dir) {
            for s in sigs {
                let ext_str = if s.extensions.is_empty() {
                    format!(".{}", s.name.to_lowercase())
                } else {
                    s.extensions.iter().map(|e| format!(".{}", e)).collect::<Vec<_>>().join(", ")
                };
                items.push(PluginItem {
                    name: s.name.clone(),
                    category: s.category.clone(),
                    plugin_type: "TOML".to_string(),
                    extension: ext_str,
                    has_validator: s.footer_bytes.is_some(),
                    status: "Active".to_string(),
                    description: format!(
                        "Active TOML signature (Header: {} bytes, Max Size: {} MB)",
                        s.header_bytes.len(),
                        (s.max_size_bytes / (1024 * 1024)).max(1)
                    ),
                });
            }
        }
    }

    // 2. Read real Lua scripts from plugins/scripts
    let script_dir = Path::new("plugins/scripts");
    if script_dir.exists() {
        if let Ok(entries) = fs::read_dir(script_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "lua").unwrap_or(false) {
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let desc = format!("Sandboxed Lua bytecode validator ({})", fname);
                    items.push(PluginItem {
                        name: fname.trim_end_matches(".lua").replace('_', " ").to_uppercase(),
                        category: if fname.contains("sqlite") { "Database".into() } else { "Media".into() },
                        plugin_type: "Lua".to_string(),
                        extension: if fname.contains("sqlite") { ".sqlite, .db".into() } else { ".jpg, .jpeg".into() },
                        has_validator: true,
                        status: "Sandboxed".to_string(),
                        description: desc,
                    });
                }
            }
        }
    }

    Ok(items)
}

#[tauri::command]
pub fn get_drive_entropy(device_path: String, chunks: Option<usize>) -> Result<Vec<f64>, String> {
    let path = Path::new(&device_path);
    let mut file = File::open(path).map_err(|e| format!("Failed to open {}: {}", device_path, e))?;
    let target_chunks = chunks.unwrap_or(64).min(256).max(16);
    let chunk_size = 64 * 1024; // 64KB per block
    let mut buffer = vec![0u8; chunk_size];
    let mut entropies = Vec::new();

    for _ in 0..target_chunks {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let ent = calculate_shannon_entropy(&buffer[..n]);
        entropies.push(ent);
    }

    if entropies.is_empty() {
        return Ok(vec![0.0; target_chunks]);
    }

    Ok(entropies)
}
