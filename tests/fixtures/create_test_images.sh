#!/usr/bin/env bash
set -euo pipefail

# Creates test disk images with known files for carver validation

OUTPUT_DIR="$(dirname "$0")/generated"
mkdir -p "$OUTPUT_DIR"

echo "[+] Creating 128MB test disk image with ext4..."
dd if=/dev/zero of="$OUTPUT_DIR/test_ext4.dd" bs=1M count=128 2>/dev/null
mkfs.ext4 -q "$OUTPUT_DIR/test_ext4.dd"

echo "[+] Creating 64MB test disk image with FAT32..."
dd if=/dev/zero of="$OUTPUT_DIR/test_fat32.dd" bs=1M count=64 2>/dev/null
mkfs.vfat -F 32 "$OUTPUT_DIR/test_fat32.dd"

echo "[✓] Test images created in $OUTPUT_DIR/"
