#!/usr/bin/env python3
import argparse, sys, os, urllib.request, urllib.error
def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge RFC 3161 Timestamping Client")
    parser.add_argument('--hash', help="SHA-256 hash to timestamp")
    parser.add_argument('--tsa-url', default="https://freetsa.org/tsr", help="TSA URL")
    parser.add_argument('--output', help="Output .tsr token path")
    parser.add_argument('--verify', action='store_true', help="Verify token")
    parser.add_argument('--token', help="Token path to verify")
    return parser.parse_args()
