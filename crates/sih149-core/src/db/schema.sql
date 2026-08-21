-- SecureForge Case Database Schema
-- SQLite 3.40+

CREATE TABLE IF NOT EXISTS drives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_number TEXT NOT NULL,
    model TEXT,
    bus_type TEXT, -- NVMe, SATA, USB, SD
    capacity_bytes INTEGER,
    smart_status TEXT,
    first_seen_at TEXT DEFAULT (datetime('now')),
    UNIQUE(serial_number)
);

CREATE TABLE IF NOT EXISTS scan_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    drive_id INTEGER REFERENCES drives(id),
    session_type TEXT NOT NULL, -- 'carve', 'wipe', 'verify'
    source_path TEXT,
    source_sha256 TEXT, -- pre-operation hash (evidence baseline)
    started_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT,
    status TEXT DEFAULT 'running', -- running, completed, failed
    parameters TEXT -- JSON blob of operation parameters
);

CREATE TABLE IF NOT EXISTS recovered_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER REFERENCES scan_sessions(id),
    filename TEXT,
    file_type TEXT,
    mime_type TEXT,
    category TEXT, -- Documents, Media, Archives, Databases, System, Unknown
    file_size INTEGER,
    sector_offset INTEGER,
    confidence_score REAL,
    sha256 TEXT,
    dhash TEXT, -- perceptual hash for images
    exif_data TEXT, -- JSON blob
    recovered_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS erasure_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER REFERENCES scan_sessions(id),
    method TEXT, -- zero, random, dod-3, dod-7, gutmann, nvme-crypto, ata-secure
    passes_completed INTEGER,
    passes_total INTEGER,
    bad_sectors TEXT, -- JSON array of failed LBAs
    post_entropy REAL,
    verified INTEGER DEFAULT 0 -- boolean
);

CREATE TABLE IF NOT EXISTS hash_chain (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_hash TEXT NOT NULL,
    previous_hash TEXT,
    payload TEXT NOT NULL, -- JSON audit entry
    timestamp TEXT DEFAULT (datetime('now')),
    rfc3161_token BLOB -- optional TSA response
);

CREATE INDEX IF NOT EXISTS idx_recovered_files_type ON recovered_files(file_type);
CREATE INDEX IF NOT EXISTS idx_recovered_files_confidence ON recovered_files(confidence_score);
CREATE INDEX IF NOT EXISTS idx_recovered_files_category ON recovered_files(category);
