//! Integration tests for the Python pipeline workers
use std::process::{Command, Stdio};
use std::io::Write;
use std::path::PathBuf;

pub fn python3_available() -> bool {
    Command::new("python3").arg("--version").output().is_ok()
}

pub fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

#[test]
fn test_report_gen_help() {
    if !python3_available() { return; }
    let root = get_workspace_root();
    let script = root.join("pipeline/report_gen.py");
    if !script.exists() { return; }
    let status = Command::new("python3")
        .arg(&script).arg("--help")
        .status().unwrap();
    assert!(status.success());
}

#[test]
fn test_classify_help() {
    if !python3_available() { return; }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() { return; }
    let status = Command::new("python3")
        .arg(&script).arg("--help")
        .status().unwrap();
    assert!(status.success());
}

#[test]
fn test_timestamp_help() {
    if !python3_available() { return; }
    let root = get_workspace_root();
    let script = root.join("pipeline/timestamp.py");
    if !script.exists() { return; }
    let status = Command::new("python3")
        .arg(&script).arg("--help")
        .status().unwrap();
    assert!(status.success());
}

#[test]
fn test_classify_empty_dir() {
    if !python3_available() { return; }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() { return; }
    
    let temp_dir = std::env::temp_dir().join(format!("secforge_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let output = Command::new("python3")
        .arg(&script).arg("--scan-dir").arg(&temp_dir)
        .output().unwrap();
        
    assert!(output.status.success());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_classify_with_file() {
    if !python3_available() { return; }
    let root = get_workspace_root();
    let script = root.join("pipeline/classify.py");
    if !script.exists() { return; }
    
    let temp_dir = std::env::temp_dir().join(format!("secforge_test_file_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let file_path = temp_dir.join("test.txt");
    std::fs::write(&file_path, "Hello world").unwrap();
    
    let output = Command::new("python3")
        .arg(&script).arg("--scan-dir").arg(&temp_dir)
        .output().unwrap();
        
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        assert!(stdout.contains("path") || stdout.contains("test.txt"));
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}
