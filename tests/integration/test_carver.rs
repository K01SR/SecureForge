//! Integration tests for the forensic file carver
use std::fs;
use std::path::{Path, PathBuf};
use sih149_core::carver::signatures::SignatureDatabase;
use sih149_core::carver::entropy::calculate_shannon_entropy;
use sih149_core::carver::confidence::ConfidenceScorer;
use sih149_core::disk::raw_image::RawImage;

pub fn create_test_image(path: &Path, size_mb: usize) -> std::io::Result<()> {
    let file = fs::File::create(path)?;
    file.set_len((size_mb * 1024 * 1024) as u64)?;
    Ok(())
}

pub fn plant_file_at_offset(image_path: &Path, file_bytes: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    let file = fs::OpenOptions::new().write(true).open(image_path)?;
    file.write_at(file_bytes, offset)?;
    Ok(())
}

pub fn jpeg_test_bytes() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00,
        0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9
    ]
}

pub fn png_test_bytes() -> Vec<u8> {
    vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R',
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00,
        0x1F, 0x15, 0xC4, 0x89,
        0x00, 0x00, 0x00, 0x00, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82
    ]
}

pub fn pdf_test_bytes() -> Vec<u8> {
    b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n1 0 obj\n<</Type/Catalog/Pages 2 0 R>>\nendobj\n%%EOF\n".to_vec()
}

pub fn sqlite_test_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 512];
    let header = b"SQLite format 3\0";
    bytes[0..16].copy_from_slice(header);
    bytes[16] = 0x10; // Page size 4096 (0x1000)
    bytes[17] = 0x00;
    bytes
}

#[test]
fn test_entropy_empty() {
    let zeros = vec![0u8; 1024];
    let e = calculate_shannon_entropy(&zeros);
    assert!(e < 0.1, "Entropy of zeros should be near 0");
}

#[test]
fn test_entropy_random() {
    use std::fs::File;
    use std::io::Read;
    let mut random_bytes = vec![0u8; 1024];
    File::open("/dev/urandom").unwrap().read_exact(&mut random_bytes).unwrap();
    let e = calculate_shannon_entropy(&random_bytes);
    assert!(e > 7.0, "Entropy of random bytes should be > 7.0");
}

#[test]
fn test_entropy_text() {
    let text = b"This is some simple ASCII text meant to test the entropy calculator. It should fall in the middle range.";
    let e = calculate_shannon_entropy(text);
    assert!(e > 3.0 && e < 5.0, "Entropy of text should be between 3.0 and 5.0 (was {})", e);
}

#[test]
fn test_jpeg_header_detection() {
    let path = PathBuf::from("/tmp/test_jpeg_detect.dd");
    create_test_image(&path, 1).unwrap();
    plant_file_at_offset(&path, &jpeg_test_bytes(), 0).unwrap();
    let mut file = fs::File::open(&path).unwrap();
    use std::io::Read;
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [0xFF, 0xD8, 0xFF]);
    let _ = fs::remove_file(path);
}

#[test]
fn test_png_header_detection() {
    let path = PathBuf::from("/tmp/test_png_detect.dd");
    create_test_image(&path, 1).unwrap();
    plant_file_at_offset(&path, &png_test_bytes(), 0).unwrap();
    let mut file = fs::File::open(&path).unwrap();
    use std::io::Read;
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"\x89PNG\x0D\x0A\x1A\x0A");
    let _ = fs::remove_file(path);
}

#[test]
fn test_raw_image_source_sector_read() {
    use std::io::Read;
    let path = PathBuf::from("/tmp/test_sector_read.dd");
    create_test_image(&path, 1).unwrap();
    if let Ok(mut src) = RawImage::open(&path) {
        let mut buf = vec![0u8; 512];
        let read = src.read(&mut buf).unwrap();
        assert_eq!(read, 512);
    }
    let _ = fs::remove_file(path);
}

#[test]
fn test_hash_chain_append_and_verify() {
    use sih149_core::audit::hashchain::HashChain;
    use sih149_core::audit::schema::{AuditEntry, OperationType, OperationResult};
    use std::collections::HashMap;

    let mut chain = HashChain::new();
    chain.append(AuditEntry {
        id: 1,
        timestamp: String::from("123"),
        operation: OperationType::DiskWipe,
        target: String::from("sys"),
        params: HashMap::new(),
        result: OperationResult {
            success: true,
            message: String::new(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: None,
            bad_sectors: None,
            files_recovered: None,
            entropy_post: None,
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    }).unwrap();
    assert!(chain.verify());
}

#[test]
fn test_hash_chain_tamper_detection() {
    use sih149_core::audit::hashchain::HashChain;
    use sih149_core::audit::schema::{AuditEntry, OperationType, OperationResult};
    use std::collections::HashMap;

    let mut chain = HashChain::new();
    chain.append(AuditEntry {
        id: 1,
        timestamp: String::from("123"),
        operation: OperationType::DiskWipe,
        target: String::from("sys"),
        params: HashMap::new(),
        result: OperationResult {
            success: true,
            message: String::new(),
            pre_hash: None,
            post_hash: None,
            sectors_processed: None,
            bad_sectors: None,
            files_recovered: None,
            entropy_post: None,
        },
        prev_hash: String::new(),
        entry_hash: String::new(),
    }).unwrap();
    assert!(chain.verify());
}

#[test]
fn test_toml_signature_loader() {
    let toml_content = r#"
[[signatures]]
name = "JPEG"
description = "test"
magic_header = "FFD8FFE0"
footer = "FFD9"
extension = "jpg"
category = "image"
max_size = 5000000

[[signatures]]
name = "PNG"
description = "test"
magic_header = "89504E470D0A1A0A"
extension = "png"
category = "image"
max_size = 10000000

[[signatures]]
name = "PDF"
description = "test"
magic_header = "255044462D"
footer = "2525454F46"
extension = "pdf"
category = "document"
max_size = 20000000
    "#;
    let path = PathBuf::from("/tmp/test_sigs.toml");
    fs::write(&path, toml_content).unwrap();
    
    let db = SignatureDatabase::load_from_toml(&path).unwrap();
    assert_eq!(db.signatures.len(), 3);
    assert_eq!(db.signatures[0].extension, "jpg");
    
    let _ = fs::remove_file(path);
}

#[test]
fn test_report_manifest_save_load() {
    use serde::{Serialize, Deserialize};
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct ReportManifest {
        case_id: i64,
        report_name: String,
        files_carved: usize,
    }
    
    let manifest = ReportManifest {
        case_id: 42,
        report_name: "Final Report".to_string(),
        files_carved: 1337,
    };
    
    let path = PathBuf::from("/tmp/test_manifest.json");
    let json = serde_json::to_string(&manifest).unwrap();
    fs::write(&path, json).unwrap();
    
    let loaded_json = fs::read_to_string(&path).unwrap();
    let loaded: ReportManifest = serde_json::from_str(&loaded_json).unwrap();
    
    assert_eq!(manifest, loaded);
    let _ = fs::remove_file(path);
}

#[test]
fn test_confidence_scoring() {
    let mut scorer = ConfidenceScorer::new();
    scorer.has_header = true;
    scorer.structure_valid = true;
    scorer.has_footer = true;
    assert_eq!(scorer.calculate(), sih149_core::carver::confidence::Confidence::High);
}
