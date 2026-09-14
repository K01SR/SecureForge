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
def detect_mime(path):
    if MAGIC_AVAILABLE:
        try:
            return magic.from_file(str(path), mime=True)
        except:
            pass
    
    ext = os.path.splitext(path)[1].lower()
    ext_map = {
        '.txt': 'text/plain', '.jpg': 'image/jpeg', '.png': 'image/png',
        '.pdf': 'application/pdf', '.zip': 'application/zip',
        '.exe': 'application/x-msdownload', '.db': 'application/x-sqlite3'
    }
    return ext_map.get(ext, 'application/octet-stream')
def get_category(mime):
    if mime.startswith('image/') or mime.startswith('video/') or mime.startswith('audio/'):
        return 'Media'
    if mime in ['application/pdf', 'text/plain']:
        return 'Documents'
    if mime in ['application/zip', 'application/x-tar']:
        return 'Archives'
    if 'sqlite' in mime or 'sql' in mime:
        return 'Databases'
    if 'executable' in mime or 'x-msdownload' in mime:
        return 'Executables'
    return 'Unknown'
def extract_exif(path):
    if not EXIF_AVAILABLE:
        return {}
    try:
        with open(path, 'rb') as f:
            tags = exifread.process_file(f, details=False)
            return {k: str(v) for k, v in tags.items() if k in ['EXIF DateTimeOriginal', 'Image Make', 'Image Model']}
    except:
        return {}
def compute_dhash(path):
    if not PIL_AVAILABLE:
        return None
    try:
        with Image.open(path) as img:
            img = img.convert('L').resize((9, 8), Image.Resampling.LANCZOS)
            pixels = list(img.getdata())
            diff = []
            for row in range(8):
                for col in range(8):
                    pixel_left = img.getpixel((col, row))
                    pixel_right = img.getpixel((col + 1, row))
                    diff.append(pixel_left > pixel_right)
            
            decimal_value = 0
            hex_string = []
            for index, value in enumerate(diff):
                if value:
                    decimal_value += 2**(index % 8)
                if (index % 8) == 7:
                    hex_string.append(hex(decimal_value)[2:].rjust(2, '0'))
                    decimal_value = 0
            return ''.join(hex_string)
    except:
        return None
def compute_sha256(path):
    h = hashlib.sha256()
    try:
        with open(path, 'rb') as f:
            while chunk := f.read(8192):
                h.update(chunk)
        return h.hexdigest()
    except:
        return None
