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
    from jinja2 import Environment, FileSystemLoader, select_autoescape
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

# Allowlist of template variables consumed by the bundled report templates.
# Any other (attacker-influenced) key present in the audit JSON is stripped
# before rendering so it cannot pollute the Jinja2 context or override a
# template global / reserved name.
ALLOWED_CONTEXT_KEYS = {
    'case_id', 'date', 'investigator', 'method', 'passes',
    'device_model', 'device_serial', 'device_capacity', 'device_interface',
    'entropy', 'device', 'files',
}

def sanitize_context(data):
    return {k: v for k, v in data.items() if k in ALLOWED_CONTEXT_KEYS}

def render_html(data, template_name, template_dir):
    try:
        env = Environment(
            loader=FileSystemLoader(template_dir),
            autoescape=select_autoescape(['html', 'xml'])
        )
    except (TypeError, NameError):
        env = Environment(loader=FileSystemLoader(template_dir), autoescape=True)

    template_file = f"{template_name}_report.html"
    if template_name == 'erasure':
        template_file = f"{template_name}_certificate.html"
    try:
        template = env.get_template(template_file)
        # Only spread the allowlisted subset of keys into the template context
        # so that extra keys from the audit JSON cannot pollute the context or
        # override Jinja2 globals.
        return template.render(**sanitize_context(data))
    except Exception as e:
        print(f"Error rendering template: {e}", file=sys.stderr)
        sys.exit(1)

def generate_pdf(html_str, output_path):
    if WEASYPRINT_AVAILABLE:
        try:
            HTML(string=html_str).write_pdf(output_path)
            return True
        except Exception as e:
            print(f"WeasyPrint error: {e}", file=sys.stderr)
    
    html_path = str(output_path).replace('.pdf', '.html')
    print(f"Warning: WeasyPrint not available or failed. Saving as HTML to {html_path}", file=sys.stderr)
    with open(html_path, 'w') as f:
        f.write(html_str)
    return False

def compute_report_hash(path):
    if not os.path.exists(path):
        return None
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        while chunk := f.read(8192):
            h.update(chunk)
    return h.hexdigest()

def main():
    args = parse_args()
    data = load_audit_data(args.input)
    data['investigator'] = args.investigator
    data['case_id'] = args.case_id
    
    template_dir = os.path.join(os.path.dirname(__file__), 'templates')
    html_str = render_html(data, args.template, template_dir)
    
    is_pdf = generate_pdf(html_str, args.output)
    final_output = args.output if is_pdf else str(args.output).replace('.pdf', '.html')
    
    file_hash = compute_report_hash(final_output)
    
    print(json.dumps({"status": "ok", "output": final_output, "hash": file_hash}))

if __name__ == '__main__':
    main()
