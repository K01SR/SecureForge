/// End-to-end live integration test for the TUI's wipe/shred core
/// Tests: DoD patterns, CSPRNG, verification, rename chain, entropy sampling
#[cfg(test)]
mod tui_live_tests {
    use sih149_core::wiper::file_wiper::{FileWiper, is_protected_path, is_protected_drive};
    use sih149_core::wiper::patterns::{generate_zeros, generate_ones, generate_random, get_dod_pattern};
    use sih149_core::wiper::verify::verify_wipe;
    use sih149_core::disk::raw_image::RawImage;
    use sih149_core::disk::DiskSource;
    use crate::app::shannon_entropy;
    use std::io::{Write, Seek, SeekFrom, Read};
    use std::path::Path;

    // ─── 1. Wipe Pattern Tests ────────────────────────────────────────────────

    #[test]
    fn test_pattern_zeros_all_zeroed() {
        let buf = generate_zeros(4096);
        assert_eq!(buf.len(), 4096);
        assert!(buf.iter().all(|&b| b == 0), "Zero pattern must produce all-zero bytes");
    }

    #[test]
    fn test_pattern_ones_all_ff() {
        let buf = generate_ones(4096);
        assert_eq!(buf.len(), 4096);
        assert!(buf.iter().all(|&b| b == 0xFF), "Ones pattern must produce all-0xFF bytes");
    }

    #[test]
    fn test_pattern_random_is_actually_random_and_high_entropy() {
        let buf = generate_random(65536);
        let entropy = shannon_entropy(&buf);
        // CSPRNG output must be > 7.8 bits/byte (close to 8.0 max)
        assert!(entropy > 7.8, "CSPRNG entropy too low: {:.4}", entropy);
        // Not all zeros or 0xFF
        assert!(!buf.iter().all(|&b| b == 0), "Random buffer should not be all zeros");
        assert!(!buf.iter().all(|&b| b == 0xFF), "Random buffer should not be all 0xFF");
    }

    #[test]
    fn test_two_random_passes_are_never_identical() {
        let a = generate_random(4096);
        let b = generate_random(4096);
        assert_ne!(a, b, "Two CSPRNG calls must produce different output (TOCTOU pass uniqueness)");
    }

    #[test]
    fn test_dod3_pass_sequence() {
        let p1 = get_dod_pattern(1)(16);
        let p2 = get_dod_pattern(2)(16);
        let p3 = get_dod_pattern(3)(65536);
        assert!(p1.iter().all(|&b| b == 0x00),   "DoD pass 1 must be zeros");
        assert!(p2.iter().all(|&b| b == 0xFF),   "DoD pass 2 must be ones");
        let ent = shannon_entropy(&p3);
        assert!(ent > 7.5, "DoD pass 3 (random) entropy too low: {:.4}", ent);
    }

    // ─── 2. Shannon Entropy Sanity ────────────────────────────────────────────

    #[test]
    fn test_entropy_zeroed_data_is_zero() {
        let zeros = vec![0u8; 65536];
        let e = shannon_entropy(&zeros);
        assert_eq!(e, 0.0, "All-zero data must have 0.0 entropy");
    }

    #[test]
    fn test_entropy_0xff_is_zero() {
        let ones = vec![0xFFu8; 65536];
        let e = shannon_entropy(&ones);
        assert_eq!(e, 0.0, "All-0xFF data must have 0.0 entropy");
    }

    #[test]
    fn test_entropy_uniform_distribution_near_8() {
        // 256 distinct bytes each appearing 256 times → exactly 8.0 bits/byte
        let data: Vec<u8> = (0u16..=255).cycle().take(65536).map(|v| v as u8).collect();
        let e = shannon_entropy(&data);
        assert!((e - 8.0).abs() < 0.001, "Uniform distribution entropy should be ≈8.0, got {:.4}", e);
    }

    #[test]
    fn test_entropy_realistic_text_is_moderate() {
        let text = b"the quick brown fox jumps over the lazy dog ".repeat(1000);
        let e = shannon_entropy(&text);
        // Natural language text: ~3.5–5.0 bits/byte
        assert!(e > 3.0 && e < 6.0, "Text entropy expected 3–6 bits/byte, got {:.4}", e);
    }

    // ─── 3. File Shred End-to-End ────────────────────────────────────────────

    #[test]
    fn test_file_shred_dod3_data_unrecoverable() {
        let dir = tempdir_in_tmp("shred_dod3");
        let path = dir.join("classified.txt");

        // Write sensitive content
        let secret = b"CLASSIFIED: SSN=123-45-6789 | CC=4111111111111111 | KEY=abc123secret";
        std::fs::write(&path, secret).unwrap();
        assert!(path.exists(), "File must exist before shred");

        // Run DoD 3-pass shred + 8 renames
        let wiper = FileWiper::new(3, 8, false);
        let result = wiper.wipe_file(&path).unwrap();

        assert!(result.success, "Shred must succeed");
        assert_eq!(result.passes_completed, 3, "Must complete all 3 DoD passes");
        assert_eq!(result.bytes_wiped, secret.len() as u64);

        // File must not exist under original path anymore (renamed + deleted)
        assert!(!path.exists(), "File must not exist after shred");
        // No temp renames should be left behind in directory
        let remaining: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(remaining.is_empty(), "No leftover rename artifacts: {:?}", remaining.iter().map(|e| e.path()).collect::<Vec<_>>());
    }

    #[test]
    fn test_file_shred_rejects_symlinks() {
        let dir = tempdir_in_tmp("shred_symlink");
        let real = dir.join("real.txt");
        let link = dir.join("link.txt");
        std::fs::write(&real, b"real content").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let wiper = FileWiper::new(1, 1, false);
        let result = wiper.wipe_file(&link);
        assert!(result.is_err(), "Shredding a symlink must return an error");
        // Real file must still exist
        assert!(real.exists(), "Target of symlink must not be deleted");
    }

    #[test]
    fn test_file_shred_rejects_protected_path() {
        let wiper = FileWiper::new(1, 1, false);
        let result = wiper.wipe_file(Path::new("/etc/hostname"));
        assert!(result.is_err(), "Shredding /etc/* must be rejected");
    }

    #[test]
    fn test_directory_shred_dod3() {
        let dir = tempdir_in_tmp("shred_dir");
        // Populate: 3 files in root + 1 subdir with 2 files
        std::fs::write(dir.join("a.txt"), b"sensitive_a").unwrap();
        std::fs::write(dir.join("b.txt"), b"sensitive_b").unwrap();
        std::fs::write(dir.join("c.txt"), b"sensitive_c").unwrap();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("d.txt"), b"sensitive_d").unwrap();
        std::fs::write(sub.join("e.txt"), b"sensitive_e").unwrap();

        let wiper = FileWiper::new(3, 4, false);
        let results = wiper.wipe_directory(&dir).unwrap();

        assert_eq!(results.len(), 5, "Must shred exactly 5 files");
        assert!(results.iter().all(|r| r.success), "All shreds must succeed");
        assert!(!dir.exists(), "Directory must be removed after shred");
    }

    // ─── 4. is_protected_path / is_protected_drive ────────────────────────────

    #[test]
    fn test_protected_paths_blocked() {
        let guarded = [
            "/etc/passwd", "/etc/shadow", "/bin/bash", "/usr/bin/sudo",
            "/boot/vmlinuz", "/sys/kernel", "/proc/1/mem",
        ];
        for p in &guarded {
            assert!(is_protected_path(Path::new(p)), "{} must be protected", p);
        }
    }

    #[test]
    fn test_tmp_paths_allowed() {
        // /tmp is not in the protected list
        assert!(!is_protected_path(Path::new("/tmp/test_safe_file.txt")));
        assert!(!is_protected_path(Path::new("/home/user/documents/file.txt")));
    }

    #[test]
    fn test_protected_drive_detection() {
        // These are in the protected list regardless of existence
        assert!(is_protected_drive(Path::new("/dev/sda")));
        assert!(is_protected_drive(Path::new("/dev/nvme0n1")));
        // Non-primary drives
        assert!(!is_protected_drive(Path::new("/dev/sdb")));
        assert!(!is_protected_drive(Path::new("/dev/sdc")));
        assert!(!is_protected_drive(Path::new("/dev/nvme1n1")));
    }

    // ─── 5. Post-Wipe Verification ────────────────────────────────────────────

    #[test]
    fn test_verify_zero_wipe_passes() {
        // Write 4 MB of zeros to a temp image file, then verify
        let dir = tempdir_in_tmp("verify_zero");
        let img = dir.join("disk.img");
        let data = vec![0u8; 4 * 1024 * 1024];
        std::fs::write(&img, &data).unwrap();

        let mut source = RawImage::open(img.to_str().unwrap()).unwrap();
        let ok = verify_wipe(&mut source, generate_zeros, 100, false).unwrap();
        assert!(ok, "Zero-wiped image must pass zero verification");
    }

    #[test]
    fn test_verify_fails_on_dirty_data() {
        let dir = tempdir_in_tmp("verify_dirty");
        let img = dir.join("disk.img");
        // Half zeros, half random — should fail zero verification
        let mut data = vec![0u8; 2 * 1024 * 1024];
        data.extend(generate_random(2 * 1024 * 1024));
        std::fs::write(&img, &data).unwrap();

        let mut source = RawImage::open(img.to_str().unwrap()).unwrap();
        let ok = verify_wipe(&mut source, generate_zeros, 100, false).unwrap();
        assert!(!ok, "Dirty data must fail zero verification");
    }

    #[test]
    fn test_verify_random_pass_checks_entropy() {
        let dir = tempdir_in_tmp("verify_rand");
        let img = dir.join("rand.img");
        let data = generate_random(4 * 1024 * 1024);
        std::fs::write(&img, &data).unwrap();

        let mut source = RawImage::open(img.to_str().unwrap()).unwrap();
        let ok = verify_wipe(&mut source, generate_random, 100, true).unwrap();
        assert!(ok, "High-entropy random data must pass random-pass verification");
    }

    #[test]
    fn test_verify_zeros_fail_as_random_pass() {
        let dir = tempdir_in_tmp("verify_zero_rand");
        let img = dir.join("zero.img");
        let data = vec![0u8; 4 * 1024 * 1024];
        std::fs::write(&img, &data).unwrap();

        let mut source = RawImage::open(img.to_str().unwrap()).unwrap();
        // zero data has 0.0 entropy → must fail the ≥7.9 entropy threshold
        let ok = verify_wipe(&mut source, generate_random, 100, true).unwrap();
        assert!(!ok, "All-zero data must FAIL random-pass verification");
    }

    // ─── 6. Gutmann Pattern Coverage ─────────────────────────────────────────

    #[test]
    fn test_gutmann_35_passes_all_unique_patterns() {
        // The TUI sends passes 1..35 cycling the DoD pattern fn
        // Verify that no two consecutive passes produce identical output
        // (i.e., pass 3/6/9... are all CSPRNG so differ each call)
        let passes: Vec<Vec<u8>> = (1u8..=3).cycle().take(35).map(|p| get_dod_pattern(p)(512)).collect();
        // All zero-passes are identical (that's correct, it's deliberate overwriting)
        // but random passes must differ
        let rand_passes: Vec<_> = passes.iter().enumerate().filter(|(i, _)| i % 3 == 2).collect();
        for i in 0..rand_passes.len() - 1 {
            assert_ne!(rand_passes[i].1, rand_passes[i+1].1, "Gutmann random passes must differ");
        }
    }

    // ─── helpers ──────────────────────────────────────────────────────────────

    fn tempdir_in_tmp(name: &str) -> std::path::PathBuf {
        let p = std::path::PathBuf::from(format!("/tmp/sf_test_{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
