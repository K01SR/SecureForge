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
