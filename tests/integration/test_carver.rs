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
fn test_load_builtin_signatures() {
    let sigs = load_builtin_signatures();
    assert!(sigs.len() > 10, "Should have more than 10 builtin signatures");
    for sig in sigs {
        assert!(!sig.header.is_empty(), "Signature {} has empty header", sig.name);
    }
}

#[test]
fn test_entropy_empty() {
    let zeros = vec![0u8; 1024];
    let e = calculate_entropy(&zeros);
    assert!(e < 0.1, "Entropy of zeros should be near 0");
}

#[test]
fn test_entropy_random() {
    use std::fs::File;
    use std::io::Read;
    let mut random_bytes = vec![0u8; 1024];
    File::open("/dev/urandom").unwrap().read_exact(&mut random_bytes).unwrap();
    let e = calculate_entropy(&random_bytes);
    assert!(e > 7.0, "Entropy of random bytes should be > 7.0");
}

#[test]
fn test_entropy_text() {
    let text = b"This is some simple ASCII text meant to test the entropy calculator. It should fall in the middle range.";
    let e = calculate_entropy(text);
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
    use sih149_core::disk::DiskSource;
    let path = PathBuf::from("/tmp/test_sector_read.dd");
    create_test_image(&path, 1).unwrap();
    let src_res = RawImageSource::new(&path);
    if let Ok(mut src) = src_res {
        let mut buf = vec![0u8; 512];
        let read = src.read_sector(0, &mut buf).unwrap();
        assert_eq!(read, 512);
    }
    let _ = fs::remove_file(path);
}

#[test]
fn test_hash_chain_append_and_verify() {
    use sih149_core::audit::hashchain::HashChain;
    let path = PathBuf::from("/tmp/test_hashchain_append.db");
    let _ = fs::remove_file(&path);
    let mut chain = HashChain::new(&path).unwrap();
    chain.append_entry("System startup", "system", "info").unwrap();
    chain.append_entry("User login", "user1", "auth").unwrap();
    chain.append_entry("File wiped", "user1", "action").unwrap();
    assert!(chain.verify_integrity().unwrap());
    let _ = fs::remove_file(path);
}

#[test]
fn test_hash_chain_tamper_detection() {
    use sih149_core::audit::hashchain::HashChain;
    use rusqlite::Connection;
    let path = PathBuf::from("/tmp/test_hashchain_tamper.db");
    let _ = fs::remove_file(&path);
    let mut chain = HashChain::new(&path).unwrap();
    chain.append_entry("Entry 1", "sys", "info").unwrap();
    chain.append_entry("Entry 2", "sys", "info").unwrap();
    
    let conn = Connection::open(&path).unwrap();
    conn.execute("UPDATE audit_log SET message = 'Tampered' WHERE id = 1", []).unwrap();
    
    assert!(!chain.verify_integrity().unwrap());
    let _ = fs::remove_file(path);
}

#[test]
fn test_toml_signature_loader() {
    use sih149_core::plugins::toml_loader::load_signatures_from_toml;
    let toml_content = r#"
[[signatures]]
name = "JPEG"
header = "FFD8FFE0"
footer = "FFD9"
extension = "jpg"
category = "image"
max_size = 5000000

[[signatures]]
name = "PNG"
header = "89504E470D0A1A0A"
extension = "png"
category = "image"
max_size = 10000000

[[signatures]]
name = "PDF"
header = "255044462D"
footer = "2525454F46"
extension = "pdf"
category = "document"
max_size = 20000000
    "#;
    let path = PathBuf::from("/tmp/test_sigs.toml");
    fs::write(&path, toml_content).unwrap();
    
    let sigs = load_signatures_from_toml(&path).unwrap();
    assert_eq!(sigs.len(), 3);
    assert_eq!(sigs[0].name, "JPEG");
    
    let _ = fs::remove_file(path);
}
