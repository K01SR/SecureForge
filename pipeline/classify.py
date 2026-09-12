#!/usr/bin/env python3
import argparse, json, sys, os, hashlib
from pathlib import Path
from datetime import datetime

try:
    import magic
    MAGIC_AVAILABLE = True
except ImportError:
    MAGIC_AVAILABLE = False

try:
    import exifread
    EXIF_AVAILABLE = True
except ImportError:
    EXIF_AVAILABLE = False

try:
    from PIL import Image
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False
def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge File Classifier")
    parser.add_argument('--scan-dir', required=True, help="Directory to scan")
    parser.add_argument('--min-size', type=int, default=0, help="Minimum file size in bytes")
    parser.add_argument('--output-json', help="Output JSON Lines file")
    return parser.parse_args()
