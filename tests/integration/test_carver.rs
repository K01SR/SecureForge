//! Integration tests for the forensic file carver
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use sih149_core::carver::signatures::load_builtin_signatures;
use sih149_core::carver::entropy::calculate_entropy;
use sih149_core::carver::confidence::score_carved_file;
use sih149_core::disk::raw_image::RawImageSource;

pub fn create_test_image(path: &Path, size_mb: usize) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.set_len((size_mb * 1024 * 1024) as u64)?;
    Ok(())
}

pub fn plant_file_at_offset(image_path: &Path, file_bytes: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    let file = fs::OpenOptions::new().write(true).open(image_path)?;
    file.write_at(file_bytes, offset)?;
    Ok(())
}
