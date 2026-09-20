use rusqlite::{Connection, params};
use std::path::Path;
use crate::error::CoreError;

pub struct CaseDatabase {
    conn: Connection,
}

impl CaseDatabase {
    pub fn open(db_path: &Path) -> Result<Self, CoreError> {
        let conn = Connection::open(db_path).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cases (
                id TEXT PRIMARY KEY, created_at TEXT NOT NULL, operator TEXT,
                target TEXT, operation TEXT, status TEXT DEFAULT 'running',
                report_path TEXT
            )",
            [],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS carved_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT, case_id TEXT NOT NULL,
                filename TEXT, file_type TEXT, size_bytes INTEGER,
                confidence INTEGER, offset_bytes INTEGER, output_path TEXT,
                category TEXT, sha256 TEXT,
                FOREIGN KEY (case_id) REFERENCES cases(id)
            )",
            [],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT, case_id TEXT NOT NULL,
                entry_json TEXT, entry_hash TEXT, recorded_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;

        Ok(Self { conn })
    }
    
    pub fn create_case(&self, case_id: &str, operator: &str, target: &str, operation: &str) -> Result<(), CoreError> {
        let created_at = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO cases (id, created_at, operator, target, operation) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![case_id, created_at, operator, target, operation],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }
    
    pub fn update_case_status(&self, case_id: &str, status: &str) -> Result<(), CoreError> {
        self.conn.execute(
            "UPDATE cases SET status = ?1 WHERE id = ?2",
            params![status, case_id],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }
    
    pub fn list_cases(&self) -> Result<Vec<CaseRecord>, CoreError> {
        let mut stmt = self.conn.prepare("SELECT id, created_at, operator, target, operation, status, report_path FROM cases")
            .map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let case_iter = stmt.query_map([], |row| {
            Ok(CaseRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                operator: row.get(2)?,
                target: row.get(3)?,
                operation: row.get(4)?,
                status: row.get(5)?,
                report_path: row.get(6)?,
            })
        }).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let mut cases = Vec::new();
        for case in case_iter {
            cases.push(case.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(cases)
    }
    
    pub fn get_case(&self, case_id: &str) -> Result<Option<CaseRecord>, CoreError> {
        let mut stmt = self.conn.prepare("SELECT id, created_at, operator, target, operation, status, report_path FROM cases WHERE id = ?1")
            .map_err(|e| CoreError::Disk(e.to_string()))?;
            
        let mut case_iter = stmt.query_map(params![case_id], |row| {
            Ok(CaseRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                operator: row.get(2)?,
                target: row.get(3)?,
                operation: row.get(4)?,
                status: row.get(5)?,
                report_path: row.get(6)?,
            })
        }).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        if let Some(case) = case_iter.next() {
            return Ok(Some(case.map_err(|e| CoreError::Disk(e.to_string()))?));
        }
        Ok(None)
    }
    
    pub fn insert_carved_file(&self, case_id: &str, file: &CarvedFileRecord) -> Result<i64, CoreError> {
        self.conn.execute(
            "INSERT INTO carved_files (case_id, filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![case_id, file.filename, file.file_type, file.size_bytes, file.confidence, file.offset_bytes, file.output_path, file.category, file.sha256],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn list_carved_files(&self, case_id: &str) -> Result<Vec<CarvedFileRecord>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256 
             FROM carved_files WHERE case_id = ?1"
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let file_iter = stmt.query_map(params![case_id], |row| {
            Ok(CarvedFileRecord {
                filename: row.get(0)?,
                file_type: row.get(1)?,
                size_bytes: row.get(2)?,
                confidence: row.get(3)?,
                offset_bytes: row.get(4)?,
                output_path: row.get(5)?,
                category: row.get(6)?,
                sha256: row.get(7)?,
            })
        }).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let mut files = Vec::new();
        for file in file_iter {
            files.push(file.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(files)
    }
    
    pub fn list_carved_files_by_category(&self, case_id: &str, category: &str) -> Result<Vec<CarvedFileRecord>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT filename, file_type, size_bytes, confidence, offset_bytes, output_path, category, sha256 
             FROM carved_files WHERE case_id = ?1 AND category = ?2"
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let file_iter = stmt.query_map(params![case_id, category], |row| {
            Ok(CarvedFileRecord {
                filename: row.get(0)?,
                file_type: row.get(1)?,
                size_bytes: row.get(2)?,
                confidence: row.get(3)?,
                offset_bytes: row.get(4)?,
                output_path: row.get(5)?,
                category: row.get(6)?,
                sha256: row.get(7)?,
            })
        }).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let mut files = Vec::new();
        for file in file_iter {
            files.push(file.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(files)
    }
    
    pub fn insert_audit_entry(&self, case_id: &str, entry_json: &str, entry_hash: &str) -> Result<(), CoreError> {
        self.conn.execute(
            "INSERT INTO audit_entries (case_id, entry_json, entry_hash) VALUES (?1, ?2, ?3)",
            params![case_id, entry_json, entry_hash],
        ).map_err(|e| CoreError::Disk(e.to_string()))?;
        Ok(())
    }
    
    pub fn get_audit_entries(&self, case_id: &str) -> Result<Vec<String>, CoreError> {
        let mut stmt = self.conn.prepare("SELECT entry_json FROM audit_entries WHERE case_id = ?1 ORDER BY id ASC")
            .map_err(|e| CoreError::Disk(e.to_string()))?;
            
        let entry_iter = stmt.query_map(params![case_id], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| CoreError::Disk(e.to_string()))?;
        
        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry.map_err(|e| CoreError::Disk(e.to_string()))?);
        }
        Ok(entries)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CaseRecord {
    pub id: String, pub created_at: String, pub operator: String,
    pub target: String, pub operation: String, pub status: String,
    pub report_path: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CarvedFileRecord {
    pub filename: String, pub file_type: String, pub size_bytes: u64,
    pub confidence: u8, pub offset_bytes: u64, pub output_path: String,
    pub category: String, pub sha256: Option<String>,
}
