pub mod schema;
pub mod hashchain;
pub mod signing;

use crate::audit::schema::*;
use crate::audit::hashchain::HashChain;
use crate::error::CoreError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct AuditEngine {
    chain: Arc<Mutex<HashChain>>,
    case_id: String,
    operator: Option<String>,
    output_dir: PathBuf,
}

impl AuditEngine {
    pub fn new(case_id: String, operator: Option<String>, output_dir: &Path) -> Self {
        Self {
            chain: Arc::new(Mutex::new(HashChain::new())),
            case_id,
            operator,
            output_dir: output_dir.to_path_buf(),
        }
    }
    
    pub fn record_wipe(&self, device: &DeviceInfo, method: &str, result: OperationResult) -> Result<String, CoreError> {
        let entry = AuditEntry {
            id: chrono::Utc::now().timestamp() as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::DiskWipe,
            target: device.path.clone(),
            params: {
                let mut map = HashMap::new();
                map.insert("method".to_string(), serde_json::Value::String(method.to_string()));
                map.insert("device".to_string(), serde_json::to_value(device).unwrap_or(serde_json::Value::Null));
                map
            },
            result,
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        
        let mut chain = self.chain.lock().unwrap();
        chain.append(entry)
    }
    
    pub fn record_scan(&self, source: &str, result: OperationResult) -> Result<String, CoreError> {
        let entry = AuditEntry {
            id: chrono::Utc::now().timestamp() as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: OperationType::RecoveryScan,
            target: source.to_string(),
            params: HashMap::new(),
            result,
            prev_hash: String::new(),
            entry_hash: String::new(),
        };
        
        let mut chain = self.chain.lock().unwrap();
        chain.append(entry)
    }
    
    pub fn verify_chain(&self) -> bool {
        let chain = self.chain.lock().unwrap();
        chain.verify()
    }
    
    pub fn export_json(&self, path: &Path) -> Result<(), CoreError> {
        let chain = self.chain.lock().unwrap();
        let log = chain.to_audit_log(self.case_id.clone(), self.operator.clone());
        let json = serde_json::to_string_pretty(&log)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        std::fs::write(path, json).map_err(CoreError::Io)
    }
    
    pub fn chain_tip(&self) -> String {
        let chain = self.chain.lock().unwrap();
        chain.tip().to_string()
    }
}
