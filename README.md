<div align="center">

# 🔒 SecureForge

**Sanitize. Recover. Certify.**

An integrated platform for secure data erasure and advanced forensic file recovery.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![NIST SP 800-88](https://img.shields.io/badge/NIST_SP_800--88-Rev_2-green.svg)](https://csrc.nist.gov/publications/detail/sp/800-88/rev-2/final)

</div>

---

## Overview

SecureForge is a unified, open-source platform that combines **secure data sanitization** and **forensic-grade file recovery** within a single tool — replacing expensive commercial solutions like Blancco ($5,000-$50,000/yr), Magnet AXIOM ($3,000-$8,000/yr), and OpenText EnCase ($3,500+/yr) at **zero cost**.

### Core Modules

| Module | Purpose |
| :--- | :--- |
| **Secure Drive Eraser** | NIST 800-88 compliant drive sanitization (NVMe Crypto Erase, ATA Secure Erase, DoD 5220.22-M) |
| **Secure File & Folder Eraser** | Selective file deletion with metadata scrubbing (MFT, inodes, directory entries, slack space) |
| **Advanced File Carver** | Forensic recovery from formatted/damaged media using signature, structure, and entropy analysis |
| **Reporting & Audit System** | Tamper-proof SHA-256 hash chain, RFC 3161 timestamps, PKI signing, PDF certificates |
| **Plugin System** | User-extensible file signatures via TOML definitions and Lua scripting |

### Deployment Modes

- **CLI** — Scriptable command-line interface for automation and expert use
- **Desktop GUI** — Tauri-based React application with beginner-friendly wizards
- **Web Server** — Same UI served over HTTPS for remote forensic stations
- **Live ISO** — Bootable Debian-based forensic environment with air-gapped mode

## Architecture

```text
Rust Core Engine ──► Tauri Desktop GUI (React/TypeScript)
       │           ├► CLI (Rust/clap)
       │           └► Web Server (axum/HTTPS)
       │
       ├── Python Pipeline (PDF reports, file classification, EXIF)
       ├── Lua Plugin Host (user-extensible file signatures)
       ├── Shell Tools (hdparm, nvme-cli, smartctl)
       └── SQLite (case management database)
```

## Standards Compliance

- **NIST SP 800-88 Rev. 2** (Media Sanitization Guidelines)
- **IEEE 2883-2022** (Device-specific sanitization)
- **Bharatiya Sakshya Adhiniyam, 2023 — Section 63** (Indian electronic evidence admissibility)
- **Information Technology Act, 2000 — Section 79A** (Digital forensics procedures)

## Quick Start

```bash
# CLI: Recover files from a disk image
sih149 recover --source evidence.dd --output ./recovered/

# CLI: Securely wipe a USB drive (DoD 3-pass)
sih149 wipe --target /dev/sdb --method dod-3 --report ./reports/

# Desktop GUI
sih149 --mode desktop

# Web Server
sih149 --mode server --port 8443 --tls-cert cert.pem
```

## Building

```bash
# Prerequisites
rustup update stable
cargo install tauri-cli

# Build CLI
cargo build --release -p sih149-cli

# Build Desktop App
cargo tauri build

# Build Live ISO
cd iso && sudo ./build.sh
```

## License

MIT — see [LICENSE](LICENSE) for details.

`libewf` is dynamically linked under LGPL-3.0.
