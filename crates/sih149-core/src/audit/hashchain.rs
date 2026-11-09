use sha2::{Sha256, Digest};
use crate::audit::schema::{AuditLog, AuditEntry};
use crate::error::CoreError;

pub struct HashChain {
    entries: Vec<AuditEntry>,
    tip: String,  // hash of last entry
}

impl HashChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            // SHA-256 hex digest is exactly 64 chars
            tip: "0".repeat(64),
        }
    }

    pub fn append(&mut self, mut entry: AuditEntry) -> Result<String, CoreError> {
        let mut entry_clone = entry.clone();
        entry_clone.prev_hash = String::new();
        entry_clone.entry_hash = String::new();
        
        let entry_json = serde_json::to_string(&entry_clone)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        
        let prev_hash = self.tip.clone();
        
        let mut hasher = Sha256::new();
        hasher.update(entry_json.as_bytes());
        hasher.update(prev_hash.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        entry.prev_hash = prev_hash;
        entry.entry_hash = hash.clone();
        
        self.entries.push(entry);
        self.tip = hash.clone();
        
        Ok(hash)
    }
    
    pub fn verify(&self) -> bool {
        let mut expected_prev = "0".repeat(64);
        for entry in &self.entries {
            if entry.prev_hash != expected_prev {
                return false;
            }
            
            let mut entry_clone = entry.clone();
            entry_clone.prev_hash = String::new();
            entry_clone.entry_hash = String::new();
            
            let entry_json = match serde_json::to_string(&entry_clone) {
                Ok(s) => s,
                Err(_) => return false,
            };
            
            let mut hasher = Sha256::new();
            hasher.update(entry_json.as_bytes());
            hasher.update(entry.prev_hash.as_bytes());
            let computed = format!("{:x}", hasher.finalize());
            
            if computed != entry.entry_hash {
                return false;
            }
            expected_prev = computed;
        }
        true
    }
    
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }
    
    pub fn tip(&self) -> &str {
        &self.tip
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn to_audit_log(&self, case_id: String, operator: Option<String>) -> AuditLog {
        AuditLog {
            case_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            operator,
            entries: self.entries.clone(),
            chain_tip: self.tip.clone(),
        }
    }
}
