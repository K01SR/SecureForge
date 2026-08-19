//! Hardware firmware sanitization commands.
//!
//! Wraps system tools as subprocesses:
//! - `hdparm --security-erase` / `--security-erase-enhanced` (ATA)
//! - `nvme sanitize --sanact=2` (Block Erase)
//! - `nvme sanitize --sanact=4` (Cryptographic Erase)
//! - `nvme format --ses=1` (User Data Erase)
//!
//! Also handles HPA/DCO detection and removal via `hdparm`.
