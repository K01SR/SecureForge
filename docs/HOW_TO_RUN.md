# SecureForge — How to Run (Current State)

> **Platform Support:** Linux ✅ Full | Windows ⚠️ Python pipeline only (Phase 5 adds full Windows) | macOS ❓ Untested

---

## What Actually Works Right Now

| Component | Linux | Windows |
|:---|:---:|:---:|
| `sih149 info` — list drives | ✅ | ⚠️ Compiles but lsblk not available |
| `sih149 recover` — file carving on .dd images | ✅ | ❌ nix ioctl won't compile |
| `sih149 wipe` — secure erase | ✅ Root required | ❌ Linux kernel calls only |
| Python PDF pipeline | ✅ | ✅ |
| Tauri desktop GUI | ✅ | ✅ (disk ops error gracefully) |
| Web server mode (`--server`) | ✅ | ❌ Tauri binary won't compile |
| Live ISO boot | ✅ Debian | ❌ N/A |
| NVMe/ATA firmware erase | ✅ Root + nvme-cli | ❌ N/A |

---

## 🐧 Linux — Full Setup

### 1. System Prerequisites

```bash
# Debian / Ubuntu
sudo apt update && sudo apt install -y \
    curl git build-essential pkg-config libssl-dev libsqlite3-dev \
    python3 python3-pip python3-venv \
    nodejs npm \
    libmagic1 libmagic-dev \
    nvme-cli hdparm smartmontools \
    webkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora / RHEL
sudo dnf install -y curl git gcc openssl-devel sqlite-devel \
    python3 python3-pip nodejs npm file-devel \
    nvme-cli hdparm smartmontools \
    webkit2gtk4.1-devel gtk3-devel
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup update stable
```

### 3. Clone and Build CLI

```bash
git clone git@github.com:Normie69K/SecureForge.git
cd SecureForge

# Build CLI (1-2 min first time)
cargo build --release --package sih149-cli

# Verify
./target/release/sih149 --version
```

### 4. Python Pipeline Setup

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r pipeline/requirements.txt

# Verify
python3 pipeline/report_gen.py --help
```

### 5. CLI Usage

```bash
# List all drives
./target/release/sih149 info

# Detail for one drive + SMART data
./target/release/sih149 info --device /dev/sda

# Recover files from a disk image
bash tests/fixtures/create_test_images.sh   # creates test .dd files
./target/release/sih149 recover \
    --source tests/fixtures/mixed_with_jpeg.dd \
    --output ./recovered \
    --types jpg,png,pdf \
    --min-confidence 50

# Wipe a drive (IRREVERSIBLE — needs root)
sudo ./target/release/sih149 wipe \
    --device /dev/sdb \
    --method dod3 \
    --verify \
    --output-report ./audit.json

# Generate PDF certificate
source .venv/bin/activate
python3 pipeline/report_gen.py \
    --input ./audit.json \
    --template erasure \
    --output ./certificate.pdf \
    --investigator "Karan Singh" \
    --case-id "CASE-2026-001"

# Classify recovered files
python3 pipeline/classify.py \
    --scan-dir ./recovered \
    --output-json ./classified.jsonl

# RFC 3161 timestamp a report hash
python3 pipeline/timestamp.py \
    --hash $(sha256sum certificate.pdf | cut -d' ' -f1) \
    --output certificate.tsr

# List audit cases
./target/release/sih149 report --list
```

### 6. Tauri Desktop GUI

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2.0"

# Install frontend deps
cd src-ui && npm install && cd ..

# Development mode (hot reload)
cargo tauri dev

# Production build → .deb + .AppImage
cargo tauri build
ls target/release/bundle/
```

### 7. Web Server Mode (REST API)

```bash
# Build Tauri backend binary
cargo build --release --package secureforge-desktop

# Start HTTP server
./target/release/secureforge-desktop --server --port 7878

# Test endpoints
curl http://localhost:7878/health
curl http://localhost:7878/api/drives
curl -X POST http://localhost:7878/api/scan \
     -H "Content-Type: application/json" \
     -d '{"source_path":"tests/fixtures/mixed_with_jpeg.dd","output_dir":"./recovered","file_types":["jpg","png"],"min_confidence":60}'
```

### 8. Run Tests

```bash
cargo test --workspace
# Expected: 23 tests passing (carver: 11, wiper: 6, pipeline: 6)
```

---

## 🪟 Windows — Python Pipeline Only

> The Rust core uses Linux-specific ioctl calls. Full Windows support is Phase 5.

### Prerequisites

```powershell
# Install Scoop
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
irm get.scoop.sh | iex
scoop install git python nodejs

# Clone repo
git clone https://github.com/Normie69K/SecureForge.git
cd SecureForge
```

### Python Pipeline (Works on Windows)

```powershell
python -m venv .venv
.venv\Scripts\activate
pip install -r pipeline\requirements.txt

# Classify files in a folder
python pipeline\classify.py --scan-dir C:\Evidence --output-json results.jsonl

# Generate report from existing JSON
python pipeline\report_gen.py `
    --input audit_data.json `
    --template erasure `
    --output certificate.pdf `
    --investigator "Karan Singh"

# RFC 3161 timestamp
python pipeline\timestamp.py --hash <sha256> --output report.tsr
```

---

## 🔑 CLI Quick Reference

```
sih149 [--verbose] [--expert] <COMMAND>

  info      [--device /dev/sdX] [--json]
  wipe      --device /dev/sdX --method METHOD [--verify] [--yes] [--expert]
  recover   --source PATH --output DIR [--types jpg,png,pdf] [--min-confidence 50]
  report    [--list] [--case-id ID] [--export PATH] [--format pdf|json|html]

Wipe Methods:
  zero, random, dod3, dod7, gutmann          (standard, no --expert needed)
  nvme-crypto, nvme-block, ata-secure, ata-enhanced  (require --expert)
```

---

## ⚠️ Safety

- Always `sih149 info` first to confirm the device path
- `wipe` needs `sudo` — kernel enforces raw block device access
- `--yes` skips the type-ERASE confirmation — only for scripts
- Firmware methods (`nvme-*`, `ata-*`) need `--expert` flag
- Tool warns on CoW filesystems (Btrfs/ZFS) — file-level wipe is not secure there
