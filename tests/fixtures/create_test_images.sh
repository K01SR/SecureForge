#!/bin/bash
# Creates test disk images for SecureForge integration tests
set -euo pipefail

FIXTURES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Creating SecureForge test fixtures..."

# 1. Create a 10MB blank image (all zeros)
dd if=/dev/zero of="$FIXTURES_DIR/blank_10mb.dd" bs=1M count=10 2>/dev/null
echo "✓ blank_10mb.dd created"

# 2. Create a 10MB random image
dd if=/dev/urandom of="$FIXTURES_DIR/random_10mb.dd" bs=1M count=10 2>/dev/null  
echo "✓ random_10mb.dd created"

# 3. Create a mixed image: random data with a JPEG planted at offset 1MB
dd if=/dev/urandom of="$FIXTURES_DIR/mixed_with_jpeg.dd" bs=1M count=10 2>/dev/null
# Plant JPEG SOI marker at 1MB offset
printf '\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00\xFF\xD9' | \
    dd of="$FIXTURES_DIR/mixed_with_jpeg.dd" bs=1 seek=1048576 conv=notrunc 2>/dev/null
echo "✓ mixed_with_jpeg.dd created (JPEG at 1MB offset)"

# 4. Create a wiped image (all zeros — simulates post-wipe)
dd if=/dev/zero of="$FIXTURES_DIR/wiped.dd" bs=1M count=5 2>/dev/null
echo "✓ wiped.dd created"

echo ""
echo "All fixtures created in $FIXTURES_DIR"
ls -lh "$FIXTURES_DIR"/*.dd
