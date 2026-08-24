#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════
# SecureForge Live ISO Builder
# Builds a bootable Debian 12 (Bookworm) forensic environment
# ═══════════════════════════════════════════════════════════

echo "╔═══════════════════════════════════════════╗"
echo "║  SecureForge Live ISO Builder             ║"
echo "║  Base: Debian 12 (Bookworm)               ║"
echo "╚═══════════════════════════════════════════╝"

# Check prerequisites
command -v lb >/dev/null 2>&1 || { echo "Error: live-build not installed. Run: sudo apt install live-build"; exit 1; }

# Clean previous builds
sudo lb clean

# Configure live-build
lb config \
    --distribution bookworm \
    --architectures amd64 \
    --binary-images iso-hybrid \
    --bootappend-live "boot=live components quiet splash" \
    --debian-installer false \
    --memtest none \
    --iso-application "SecureForge Forensic Environment" \
    --iso-publisher "SecureForge Project" \
    --iso-volume "SECUREFORGE"

echo "[+] Building ISO image..."
sudo lb build

echo "[✓] ISO build complete: live-image-amd64.hybrid.iso"
