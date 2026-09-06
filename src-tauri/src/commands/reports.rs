use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use rusqlite::{Connection, Result as SqlResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseRecord {
    pub id: String,
    pub created_at: String,
    pub operation_type: String,
    pub target: String,
    pub status: String,
}

fn get_db_path() -> Result<PathBuf, String> {
    let mut path = dirs::data_local_dir().ok_or("Cannot find local data dir")?;
    path.push("secureforge");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("cases.db");
    Ok(path)
}

fn init_db(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cases (
            id TEXT PRIMARY KEY,
            created_at TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            target TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

#[tauri::command]
pub fn list_cases() -> Result<Vec<CaseRecord>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, created_at, operation_type, target, status FROM cases ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let cases = stmt.query_map([], |row| {
        Ok(CaseRecord {
            id: row.get(0)?,
            created_at: row.get(1)?,
            operation_type: row.get(2)?,
            target: row.get(3)?,
            status: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for case in cases {
        result.push(case.map_err(|e| e.to_string())?);
    }

    Ok(result)
}

#[tauri::command]
pub fn export_report(case_id: String, format: String, output_path: String) -> Result<(), String> {
    // spawn python3 pipeline/report_gen.py
    let output = Command::new("python3")
        .args(["pipeline/report_gen.py", &case_id, &format, &output_path])
        .output()
        .map_err(|e| e.to_string())?;
        
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_audit_log(case_id: String) -> Result<String, String> {
    // Dummy JSON
    Ok(format!(r#"{{ "case_id": "{}", "events": [] }}"#, case_id))
}
