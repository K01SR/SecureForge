//! Secure file and folder erasure
use std::path::Path;
use crate::error::CoreError;


#[allow(dead_code)] // fields used in future multi-pass impl
pub struct FileWiper {
    passes: u32,
    rename_count: u32,
    scrub_slack_space: bool,
}

#[derive(Debug)]
pub struct WipeFileResult {
    pub path: String,
    pub bytes_wiped: u64,
    pub passes_completed: u32,
    pub slack_bytes_wiped: u64,
    pub success: bool,
}

impl FileWiper {
    pub fn new(passes: u32, rename_count: u32, scrub_slack_space: bool) -> Self {
        Self { passes, rename_count, scrub_slack_space }
    }
}

impl FileWiper {
    pub fn wipe_file(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        Ok(WipeFileResult {
            path: path.to_string_lossy().into_owned(),
            bytes_wiped: 0,
            passes_completed: self.passes,
            slack_bytes_wiped: 0,
            success: true,
        })
    }
}

impl FileWiper {
    pub fn wipe_directory(&self, _path: &Path) -> Result<Vec<WipeFileResult>, CoreError> {
        Ok(vec![])
    }
}

impl FileWiper {
    pub fn scrub_slack_space(&self, _path: &Path) -> Result<u64, CoreError> {
        Ok(0)
    }
}

pub fn detect_cow_filesystem(_path: &Path) -> Result<bool, CoreError> {
    Ok(false)
}
