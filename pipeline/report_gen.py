#!/usr/bin/env python3
import argparse, json, sys, os, hashlib
from pathlib import Path
from datetime import datetime

try:
    from weasyprint import HTML
    WEASYPRINT_AVAILABLE = True
except ImportError:
    WEASYPRINT_AVAILABLE = False

try:
    from jinja2 import Environment, FileSystemLoader
except ImportError:
    pass

def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge Report Generator")
    parser.add_argument('--input', required=True, help="Input JSON file")
    parser.add_argument('--template', required=True, choices=['erasure', 'recovery', 'custody'])
    parser.add_argument('--output', required=True, help="Output PDF path")
    parser.add_argument('--investigator', help="Investigator name")
    parser.add_argument('--case-id', help="Case ID")
    parser.add_argument('--tsa-token', help="TSA token path")
    return parser.parse_args()

def load_audit_data(path):
    try:
        with open(path, 'r') as f:
            data = json.load(f)
            return data
    except Exception as e:
        print(f"Error loading audit data: {e}", file=sys.stderr)
        sys.exit(1)

def render_html(data, template_name, template_dir):
    env = Environment(loader=FileSystemLoader(template_dir))
    template_file = f"{template_name}_report.html"
    if template_name == 'erasure':
        template_file = f"{template_name}_certificate.html"
    try:
        template = env.get_template(template_file)
        return template.render(**data)
    except Exception as e:
        print(f"Error rendering template: {e}", file=sys.stderr)
        sys.exit(1)

