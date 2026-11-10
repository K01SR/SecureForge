//! Secure file and folder erasure.
//!
//! Overwrites file content in place (respecting the same DoD pattern
//! sequence as the drive wiper), renames the file N times with random
//! names to scrub the original filename from directory-entry/journal
//! history, then deletes it.
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use rand::RngCore;
use rand::rngs::OsRng;
use crate::error::CoreError;
use crate::wiper::patterns::get_dod_pattern;

#[allow(dead_code)] // scrub_slack_space read via method call below, not dead
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

    pub fn wipe_file(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        let metadata = fs::metadata(path).map_err(CoreError::Io)?;
        let size = metadata.len();
        let chunk_size: usize = 1024 * 1024;

        let mut file = OpenOptions::new().write(true).open(path).map_err(CoreError::Io)?;

        let mut passes_completed = 0u32;
        for pass in 1..=self.passes.max(1) {
            let pattern_fn = get_dod_pattern(pass as u8);
            file.seek(SeekFrom::Start(0)).map_err(CoreError::Io)?;
            let mut written: u64 = 0;
            while written < size {
                let this_chunk = std::cmp::min(chunk_size as u64, size - written) as usize;
                let buf = pattern_fn(this_chunk);
                file.write_all(&buf).map_err(CoreError::Io)?;
                written += this_chunk as u64;
            }
            file.flush().map_err(CoreError::Io)?;
            // sync_all forces the write to physical media, not just the
            // page cache — without this, a power loss right after wipe_file
            // returns could leave old data recoverable.
            file.sync_all().map_err(CoreError::Io)?;
            passes_completed += 1;
        }
        drop(file);

        // Rename N times with random hex names, then delete. Each rename
        // is a separate journal/directory-entry transaction, which is the
        // point — it overwrites the *name* history, not just content.
        let mut current_path = path.to_path_buf();
        let parent = current_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        for _ in 0..self.rename_count {
            let new_path = parent.join(random_filename());
            fs::rename(&current_path, &new_path).map_err(CoreError::Io)?;
            current_path = new_path;
        }
        fs::remove_file(&current_path).map_err(CoreError::Io)?;

        let slack_bytes_wiped = if self.scrub_slack_space {
            // Propagate the error instead of unwrap_or(0) — if the user
            // explicitly asked for slack scrubbing, silently reporting 0
            // bytes wiped would misrepresent an unimplemented feature as
            // "ran and found nothing to do."
            self.scrub_slack_space(path)?
        } else {
            0
        };

        Ok(WipeFileResult {
            path: path.to_string_lossy().into_owned(),
            bytes_wiped: size,
            passes_completed,
            slack_bytes_wiped,
            success: true,
        })
    }

    pub fn wipe_directory(&self, path: &Path) -> Result<Vec<WipeFileResult>, CoreError> {
        let mut results = Vec::new();
        self.wipe_directory_inner(path, &mut results)?;
        Ok(results)
    }

    fn wipe_directory_inner(&self, path: &Path, results: &mut Vec<WipeFileResult>) -> Result<(), CoreError> {
        for entry in fs::read_dir(path).map_err(CoreError::Io)? {
            let entry = entry.map_err(CoreError::Io)?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.wipe_directory_inner(&entry_path, results)?;
                fs::remove_dir(&entry_path).map_err(CoreError::Io)?;
            } else {
                results.push(self.wipe_file(&entry_path)?);
            }
        }
        Ok(())
    }

    /// NOT YET IMPLEMENTED. Slack space (the unused tail of a file's last
    /// filesystem block) can retain fragments of the file's own old
    /// content or even a previous, unrelated file's data. Scrubbing it
    /// correctly requires reading the filesystem's block size and writing
    /// directly to the raw block device at the file's last-block offset —
    /// see wiper::metadata::{ext4,ntfs,fat} for the per-filesystem layout
    /// this needs. Returns an explicit error rather than a fake success.
    pub fn scrub_slack_space(&self, _path: &Path) -> Result<u64, CoreError> {
        Err(CoreError::Wiper(
            "Slack space scrubbing not yet implemented — requires raw block-level filesystem access".to_string()
        ))
    }
}

fn random_filename() -> String {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// NOT YET IMPLEMENTED (returns a hardcoded `false`, i.e. "assume not
/// CoW"). Copy-on-write filesystems (Btrfs, ZFS, APFS) can retain old
/// blocks via snapshots even after an in-place overwrite — a real gap in
/// this tool's threat model until this is implemented. Do not treat the
/// current `false` as a verified guarantee; it is an unchecked default.
pub fn detect_cow_filesystem(_path: &Path) -> Result<bool, CoreError> {
    Ok(false)
}
