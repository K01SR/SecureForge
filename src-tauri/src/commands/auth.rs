use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Serialize, Deserialize)]
struct AuthConfig {
    expert_hash: String,
}

fn get_config_path() -> Result<PathBuf, String> {
    let mut path = dirs::config_dir().ok_or("Cannot find config dir")?;
    path.push("secureforge");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("expert.toml");
    Ok(path)
}

#[tauri::command]
pub async fn setup_expert_passphrase(passphrase: String) -> Result<(), String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    let config = AuthConfig { expert_hash: password_hash };
    let toml = toml::to_string(&config).map_err(|e| e.to_string())?;
    
    let path = get_config_path()?;
    fs::write(&path, toml).map_err(|e| e.to_string())?;

    // Restrict config file permissions to owner-only (rw-------) so the
    // Argon2 hash and any future secrets aren't world-readable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::metadata(&path).map_err(|e| e.to_string())?;
        fs::set_permissions(&path, fs::Permissions::from_mode(perms.permissions().mode() & 0o600))
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn verify_expert_passphrase(passphrase: String) -> Result<bool, String> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(false);
    }
    
    let toml_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: AuthConfig = toml::from_str(&toml_str).map_err(|e| e.to_string())?;
    
    let parsed_hash = PasswordHash::new(&config.expert_hash).map_err(|e| e.to_string())?;
    let is_valid = Argon2::default().verify_password(passphrase.as_bytes(), &parsed_hash).is_ok();
    
    Ok(is_valid)
}

#[tauri::command]
pub async fn is_expert_configured() -> Result<bool, String> {
    let path = get_config_path()?;
    Ok(path.exists())
}
