# SecureForge User Manual

## 1. Introduction & Overview
SecureForge is an advanced, cross-platform utility combining secure data erasure and forensic file recovery. Tailored for enterprise and forensic professionals, it surpasses conventional utilities (like DBAN or PhotoRec) by offering native NVMe Crypto Erase, detailed entropy-based confidence scoring, and cryptographic chain-of-custody for audit logs.

## 2. Installation
**Prerequisites:** Rust toolchain, Python 3.10+, and administrative privileges.
- Download from source: `git clone https://github.com/Normie69K/SecureForge.git`
- Build Core: `cargo build --release`
- Install Python deps: `pip install -r pipeline/requirements.txt`
First Launch: Run the UI via `cargo tauri dev` or `sih149 --help` for CLI.

