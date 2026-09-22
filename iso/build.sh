#!/bin/bash
# SecureForge Live ISO Builder
# Builds a bootable Debian 12 Bookworm live ISO with SecureForge pre-installed
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$SCRIPT_DIR/build"
OUTPUT_DIR="$SCRIPT_DIR/output"
VERSION="0.1.0"
ISO_NAME="secureforge-${VERSION}-live-amd64.iso"
