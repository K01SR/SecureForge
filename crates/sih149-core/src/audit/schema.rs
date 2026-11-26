use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Top-level audit log — wraps all entries for a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub case_id: String,
    pub created_at: String,     // ISO 8601
    pub tool_version: String,
    pub operator: Option<String>,
    pub entries: Vec<AuditEntry>,
    pub chain_tip: String,      // SHA-256 of last entry
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: String,
    pub operation: OperationType,
    pub target: String,
    pub params: HashMap<String, serde_json::Value>,
    pub result: OperationResult,
    pub prev_hash: String,
    pub entry_hash: String,
    /// Monotonic sequence number (chain position) bound into the hash to
    /// prevent reordering/duplication attacks against the chain.
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType { DiskWipe, FileWipe, RecoveryScan, ImageAcquisition, HashVerification }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub pre_hash: Option<String>,
    pub post_hash: Option<String>,
    pub sectors_processed: Option<u64>,
    pub bad_sectors: Option<Vec<u64>>,
    pub files_recovered: Option<u32>,
    pub entropy_post: Option<f64>,
}

/// Device info embedded in audit reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub path: String,
    pub model: String,
    pub serial: String,
    pub capacity_bytes: u64,
    pub interface: String,      // SATA/NVMe/USB
    pub smart_status: String,
    pub firmware: String,
}
