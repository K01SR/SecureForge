//! Integration tests for the secure wiper
use sih149_core::wiper::patterns::*;
use sih149_core::wiper::verify::verify_wipe;
use sih149_core::wiper::file_wiper::{FileWiper, is_protected_path, detect_cow_filesystem};
use sih149_core::carver::entropy::calculate_shannon_entropy;
use sih149_core::disk::raw_image::RawImage;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

#[test]
fn test_zero_pattern() {
    let buf = generate_zeros(512);
    assert!(buf.iter().all(|&b| b == 0x00));
}

#[test]
fn test_random_pattern_entropy() {
    let buf = generate_random(1024);
    let entropy = calculate_shannon_entropy(&buf);
    assert!(entropy > 7.0, "Entropy should be high for random pattern");
}

#[test]
fn test_dod3_pass_count() {
    let pass1 = get_dod_pattern(1);
    let pass2 = get_dod_pattern(2);
    let pass3 = get_dod_pattern(3);
    assert_eq!(pass1(5), vec![0, 0, 0, 0, 0]);
    assert_eq!(pass2(5), vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    let b3 = pass3(5);
    assert_ne!(b3, vec![0, 0, 0, 0, 0]);
}

#[test]
fn test_pattern_no_two_passes_identical() {
    let buf1 = get_dod_pattern(1)(512);
    let buf2 = get_dod_pattern(2)(512);
    assert_ne!(buf1, buf2, "Consecutive passes should differ");
}

#[test]
fn test_file_wiper_cow_detection() {
    let tmp_path = PathBuf::from("/tmp");
    let result = detect_cow_filesystem(&tmp_path);
    assert!(result.is_ok(), "detect_cow_filesystem should execute successfully");
}

#[test]
fn test_file_wiper_is_protected_path() {
    assert!(is_protected_path(Path::new("/etc")));
    assert!(is_protected_path(Path::new("/boot")));
    assert!(is_protected_path(Path::new("/usr/bin")));
    assert!(is_protected_path(Path::new("/bin/sh")));
    assert!(!is_protected_path(Path::new("/tmp/some_safe_shred_test_file.txt")));
}

#[test]
fn test_file_wiper_shred_real_file() {
    let test_path = PathBuf::from("/tmp/test_wiper_shred_sample.txt");
    fs::write(&test_path, b"Confidential sensitive user data to shred").unwrap();
    assert!(test_path.exists());

    let wiper = FileWiper::new(3, 4, false);
    let result = wiper.wipe_file(&test_path).unwrap();

    assert!(result.success);
    assert_eq!(result.passes_completed, 3);
    assert!(!test_path.exists(), "File should be deleted after wiping");
}

#[test]
fn test_file_wiper_refuses_symlink() {
    let target_file = PathBuf::from("/tmp/test_target_keep_intact.txt");
    let symlink_file = PathBuf::from("/tmp/test_evil_symlink.txt");

    fs::write(&target_file, b"CRITICAL_DATA_DO_NOT_TOUCH").unwrap();
    let _ = fs::remove_file(&symlink_file);

    #[cfg(unix)]
    std::os::unix::fs::symlink(&target_file, &symlink_file).unwrap();

    let wiper = FileWiper::new(2, 2, false);
    let result = wiper.wipe_file(&symlink_file);

    // Should refuse to shred symlink directly
    assert!(result.is_err(), "Wiper should refuse symlink directly");

    // Target file must remain untouched
    assert!(target_file.exists());
    assert_eq!(fs::read(&target_file).unwrap(), b"CRITICAL_DATA_DO_NOT_TOUCH");

    let _ = fs::remove_file(symlink_file);
    let _ = fs::remove_file(target_file);
}

#[test]
fn test_file_wiper_directory_shred() {
    let dir = PathBuf::from("/tmp/test_wiper_dir_shred");
    let sub = dir.join("nested");
    fs::create_dir_all(&sub).unwrap();
    fs::write(dir.join("f1.txt"), b"file one data").unwrap();
    fs::write(sub.join("f2.txt"), b"file two nested data").unwrap();

    let wiper = FileWiper::new(2, 2, false);
    let results = wiper.wipe_directory(&dir).unwrap();

    assert_eq!(results.len(), 2);
    assert!(!dir.exists(), "Directory should be completely removed after wipe");
}

#[test]
fn test_wipe_verification_on_zeroed_data() {
    let path = PathBuf::from("/tmp/test_verify.dd");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(&vec![0u8; 1024 * 1024]).unwrap(); // 1 MB
    drop(file);
    
    let mut disk = RawImage::open(&path).unwrap();
    let verified = verify_wipe(&mut disk, generate_zeros, 100, false).unwrap();
    assert!(verified, "Should verify clean zeroed data");
    
    let _ = fs::remove_file(path);
}
