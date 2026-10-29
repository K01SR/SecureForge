# SecureForge User Manual

## 1. Introduction & Overview
SecureForge is an advanced, cross-platform utility combining secure data erasure and forensic file recovery. Tailored for enterprise and forensic professionals, it surpasses conventional utilities (like DBAN or PhotoRec) by offering native NVMe Crypto Erase, detailed entropy-based confidence scoring, and cryptographic chain-of-custody for audit logs.

## 2. Installation
**Prerequisites:** Rust toolchain, Python 3.10+, and administrative privileges.
- Download from source: `git clone https://github.com/Normie69K/SecureForge.git`
- Build Core: `cargo build --release`
- Install Python deps: `pip install -r pipeline/requirements.txt`
First Launch: Run the UI via `cargo tauri dev` or `sih149 --help` for CLI.

## 3. Quick Start Guide
1. **Check Drives:** `sih149 info`
2. **Wipe a USB:** `sih149 wipe --device /dev/sdb --method dod3`
3. **Recover Files:** `sih149 recover --source disk.dd --output ./out`

## 4. CLI Reference
- `sih149 info [--device PATH] [--json]`: Display disk information.
- `sih149 wipe --device PATH --method METHOD [--verify] [--yes] [--expert]`: Erase data securely.
- `sih149 recover --source PATH --output DIR [--types jpg,png,pdf] [--min-confidence 50]`: Recover files with filters.
- `sih149 report [--list] [--case-id ID] [--export PATH] [--format pdf|json|html|zip]`: Generate audit reports.

