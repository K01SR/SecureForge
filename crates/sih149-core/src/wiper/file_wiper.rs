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

/// Returns `true` when `path` (after canonicalization) is under a protected
/// system prefix. Callers should refuse to shred or return an error.
///
/// Protected prefixes:
///   /boot, /bin, /sbin, /lib, /lib64, /usr, /etc, /root, /sys, /proc, /dev
///
/// Symlinks are resolved before comparison so that e.g.
///   --target /tmp/evil_link   where the link points to /etc/passwd
/// is correctly blocked, not silently followed.
pub fn is_protected_path(path: &Path) -> bool {
    let resolved = match std::fs::canonicalize(path) {
        Ok(p) => p,
        // Canonicalize fails on non-existent targets — treat as not protected,
        // existence check is the caller's responsibility.
        Err(_) => path.to_path_buf(),
    };
    let protected = [
        "/boot", "/bin", "/sbin", "/lib", "/lib64",
        "/usr", "/etc", "/root", "/sys", "/proc", "/dev",
    ];
    let s = resolved.to_string_lossy();
    protected.iter().any(|prefix| s == *prefix || s.starts_with(&format!("{}/", prefix)))
}

/// Returns `true` if `device` points to a primary boot or system block device
/// (e.g. /dev/sda, /dev/nvme0n1, /dev/disk0, /dev/vda, /dev/root).
/// Symlinks and partition aliases are resolved before comparison.
pub fn is_protected_drive(device: &Path) -> bool {
    let protected_prefixes = [
        "/dev/sda",
        "/dev/nvme0n1",
        "/dev/disk0",
        "/dev/vda",
        "/dev/root",
    ];
    let canonical = std::fs::canonicalize(device)
        .unwrap_or_else(|_| device.to_path_buf());
    let s = canonical.to_string_lossy();
    protected_prefixes.iter().any(|prefix| s == *prefix || s.starts_with(prefix))
}

impl FileWiper {
    pub fn new(passes: u32, rename_count: u32, scrub_slack_space: bool) -> Self {
        Self { passes, rename_count, scrub_slack_space }
    }

    pub fn wipe_file(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        // Path validation — block obvious escape attempts here in the core
        // so every caller (CLI and Tauri) gets the same protection.
        if is_protected_path(path) {
            return Err(CoreError::Wiper(format!(
                "Refusing to shred protected system path: {} — use the expert CLI with explicit override if intentional",
                path.display()
            )));
        }

        // Use symlink_metadata (lstat) NOT metadata (stat) — metadata() follows
        // symlinks and would report a symlink as a regular file, letting an
        // attacker place a symlink at --target pointing to /etc/passwd.
        let metadata = fs::symlink_metadata(path).map_err(CoreError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(CoreError::Wiper(format!(
                "Refusing to shred a symlink directly: {} (target not followed) — \
                 pass the real file path if you intend to shred its content",
                path.display()
            )));
        }
        let size = metadata.len();
        let chunk_size: usize = 1024 * 1024;

        // O_NOFOLLOW as a second defense layer: even if something replaces
        // this path with a symlink in the gap between the metadata check above
        // and this open() (a TOCTOU race), the open fails with ELOOP instead
        // of silently writing through the symlink to an arbitrary target.
        #[cfg(unix)]
        let open_opts = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut o = OpenOptions::new();
            o.write(true).custom_flags(libc::O_NOFOLLOW);
            o
        };
        #[cfg(not(unix))]
        let open_opts = { let mut o = OpenOptions::new(); o.write(true); o };

        let mut file = open_opts.open(path).map_err(CoreError::Io)?;

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
        if is_protected_path(path) {
            return Err(CoreError::Wiper(format!(
                "Refusing to shred protected system directory: {}",
                path.display()
            )));
        }
        let mut results = Vec::new();
        self.wipe_directory_inner(path, &mut results)?;
        fs::remove_dir(path).map_err(CoreError::Io)?;
        Ok(results)
    }

    fn wipe_directory_inner(&self, path: &Path, results: &mut Vec<WipeFileResult>) -> Result<(), CoreError> {
        for entry in fs::read_dir(path).map_err(CoreError::Io)? {
            let entry = entry.map_err(CoreError::Io)?;
            let entry_path = entry.path();

            // entry.file_type() comes from the directory entry itself (like
            // lstat) and does NOT follow symlinks — unlike entry_path.is_dir(),
            // which calls metadata() and DOES follow them. Using is_dir() here
            // previously let a symlink inside the target directory (pointing
            // anywhere on the filesystem, e.g. /etc/shadow, or even another
            // directory entirely) get silently followed and overwritten/
            // recursed-into. Refuse symlinks explicitly instead of following
            // them into wipe_file_unchecked/recursion.
            let file_type = entry.file_type().map_err(CoreError::Io)?;
            if file_type.is_symlink() {
                results.push(WipeFileResult {
                    path: entry_path.to_string_lossy().into_owned(),
                    bytes_wiped: 0,
                    passes_completed: 0,
                    slack_bytes_wiped: 0,
                    success: false,
                });
                tracing::warn!(
                    "Skipped symlink during directory shred (not followed): {}",
                    entry_path.display()
                );
                continue;
            }

            if file_type.is_dir() {
                self.wipe_directory_inner(&entry_path, results)?;
                fs::remove_dir(&entry_path).map_err(CoreError::Io)?;
            } else {
                // Per-file is_protected_path skipped here — the outer
                // wipe_directory already blocked on the parent; individual
                // files inside are implicitly under it. Symlinks (the actual
                // escape vector) are already excluded above.
                results.push(self.wipe_file_unchecked(&entry_path)?);
            }
        }
        Ok(())
    }

    /// Like `wipe_file` but skips the protected-path check.
    /// Used internally when the parent directory has already been validated.
    fn wipe_file_unchecked(&self, path: &Path) -> Result<WipeFileResult, CoreError> {
        let metadata = fs::symlink_metadata(path).map_err(CoreError::Io)?;
        if metadata.file_type().is_symlink() {
            // Defense in depth: wipe_directory_inner already filters symlinks
            // out before calling this, but wipe_file (the public single-file
            // entry point) can still be pointed at a symlink directly by a
            // caller — refuse here too rather than relying on callers to remember.
            return Err(CoreError::Wiper(format!(
                "Refusing to shred a symlink directly: {} (target not followed)",
                path.display()
            )));
        }
        let size = metadata.len();
        let chunk_size: usize = 1024 * 1024;

        // O_NOFOLLOW as a second layer: even if something replaces this path
        // with a symlink in the gap between the metadata check above and this
        // open() (a TOCTOU race), the open fails instead of silently writing
        // through the symlink to an arbitrary target.
        #[cfg(unix)]
        let open_opts = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut o = OpenOptions::new();
            o.write(true).custom_flags(libc::O_NOFOLLOW);
            o
        };
        #[cfg(not(unix))]
        let open_opts = { let mut o = OpenOptions::new(); o.write(true); o };

        let mut file = open_opts.open(path).map_err(CoreError::Io)?;
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
            file.sync_all().map_err(CoreError::Io)?;
            passes_completed += 1;
        }
        drop(file);

        let mut current_path = path.to_path_buf();
        let parent = current_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        for _ in 0..self.rename_count {
            let new_path = parent.join(random_filename());
            fs::rename(&current_path, &new_path).map_err(CoreError::Io)?;
            current_path = new_path;
        }
        fs::remove_file(&current_path).map_err(CoreError::Io)?;

        Ok(WipeFileResult {
            path: path.to_string_lossy().into_owned(),
            bytes_wiped: size,
            passes_completed,
            slack_bytes_wiped: 0,
            success: true,
        })
    }

    /// NOT YET IMPLEMENTED — requires raw block-level filesystem access.
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

/// Returns `true` if `path` lives on a Copy-on-Write filesystem (Btrfs, ZFS).
///
/// On CoW filesystems, even a byte-for-byte in-place overwrite does not
/// guarantee the old data is destroyed — the filesystem may retain old
/// block versions accessible via snapshots or free-space reclamation.
///
/// Implementation reads the statfs f_type field via libc for the mounted
/// filesystem at the given path. Falls back to `false` on any error so
/// callers at minimum see a "possibly CoW" advisory rather than a hard fail.
pub fn detect_cow_filesystem(path: &Path) -> Result<bool, CoreError> {
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|e| CoreError::Disk(format!("Invalid path: {}", e)))?;

        let mut buf: libc::statfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statfs(c_path.as_ptr(), &mut buf) };

        if ret != 0 {
            // Don't hard-fail — return false and let the caller add an advisory
            return Ok(false);
        }

        // f_type magic numbers for CoW filesystems:
        //   Btrfs:  0x9123683E
        //   ZFS:    0x2FC12FC1  (OpenZFS on Linux)
        let is_cow = matches!(buf.f_type, 0x9123683E | 0x2FC12FC1_i64);
        Ok(is_cow)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux platforms (macOS/APFS, Windows/ReFS) we can't portably
        // query the filesystem type without OS-specific APIs. Return false
        // conservatively — the UI/CLI should show an advisory on these platforms.
        let _ = path;
        Ok(false)
    }
}
