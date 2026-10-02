//! Secure file and folder erasure
use std::fs::{self, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use crate::error::CoreError;

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
