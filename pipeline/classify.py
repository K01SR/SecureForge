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
