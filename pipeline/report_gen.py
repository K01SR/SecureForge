#!/usr/bin/env python3
"""
SecureForge Report Generator

Reads JSON audit data from stdin or file, renders a styled HTML
template via Jinja2, and converts it to PDF using WeasyPrint.

Usage:
    python3 report_gen.py --input audit.json --template erasure --output report.pdf
    python3 report_gen.py --input audit.json --template recovery --output report.pdf
"""

def main():
    # TODO: Implement report generation pipeline
    print("SecureForge Report Generator — not yet implemented")

if __name__ == "__main__":
    main()
