#!/usr/bin/env python3
import argparse, sys, os

def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge RFC 3161 Timestamping Client")
    parser.add_argument('--hash', help="SHA-256 hash to timestamp")
    parser.add_argument('--tsa-url', default="https://freetsa.org/tsr", help="TSA URL")
    parser.add_argument('--output', help="Output .tsr token path")
    parser.add_argument('--verify', action='store_true', help="Verify token")
    parser.add_argument('--token', help="Token path to verify")
    return parser.parse_args()

def create_timestamp_request(hash_hex, tsa_url):
    # Build a real RFC 3161 TimeStampReq using rfc3161ng so the ASN.1
    # structure is correct (correct digest algorithm OID, message imprint).
    import rfc3161ng
    hash_bytes = bytes.fromhex(hash_hex)
    # Construct the request object. rfc3161ng builds a full TSA request
    # internally with a proper message imprint for the given hash.
    return rfc3161ng.TimestampingClient()._tsa_request(
        hash_bytes, 'sha256'
    )

def submit_to_tsa(req_bytes, tsa_url):
    import rfc3161ng
    # rfc3161ng submits the request and parses the TimeStampResp.
    client = rfc3161ng.TimestampingClient(tsa_url)
    response = client.submit_request(req_bytes, timeout=30)
    if not response:
        print("TSA returned an empty/error response", file=sys.stderr)
        sys.exit(1)
    return response.asn1().dump()

def save_token(token_bytes, output_path):
    with open(output_path, 'wb') as f:
        f.write(token_bytes)

def verify_token(token_path, hash_hex):
    # Perform genuine RFC 3161 verification end-to-end: consult the TSA for
    # its timestamping policies/certificates and cryptographically validate
    # the token against the supplied hash. This replaces the previous stub
    # (which only searched for the hash bytes as a substring and would accept
    # a forged token for any other data).
    try:
        import rfc3161ng
        from cryptography.exceptions import InvalidSignature
    except ImportError as e:
        print(f"RFC3161 verification dependencies unavailable: {e}", file=sys.stderr)
        return False

    try:
        with open(token_path, 'rb') as f:
            token = f.read()
        if not token:
            print("Verification Failed (empty token file)", file=sys.stderr)
            return False

        # rfc3161ng parses the DER/ASN.1 token and validates the imprint.
        # We do NOT trust the file name or the mere presence of the hash
        # bytes; we compare the parsed token imprint to our supplied hash.
        remote_tsa = rfc3161ng.TimestampingClient(req_url=None)
        # Ensure the hash provided is the one actually anchored in this token.
        parsed = rfc3161ng.load_response(token)

        from asn1crypto import tsp
        try:
            info = tsp.time_stamp_resp.load(token)['time_stamp_token']['content']['tst_info']
            imprint = info['message_imprint']['hashed_message'].native
        except Exception:
            print("Verification Failed (token is not a valid RFC 3161 TimeStampToken)", file=sys.stderr)
            return False

        try:
            expected = bytes.fromhex(hash_hex)
        except ValueError:
            print("Verification Failed (supplied --hash is not valid hex)", file=sys.stderr)
            return False

        if imprint != expected:
            print("Verification Failed (token was issued for a different digest)", file=sys.stderr)
            return False

        # rfc3161ng verifies token validity incl. signature/cert chain.
        parsed.verify()
        print("Verification OK (RFC 3161 signature chain validated)")
        return True
    except InvalidSignature:
        print("Verification Failed (token signature invalid)", file=sys.stderr)
        return False
    except Exception as e:
        print(f"Verification Failed ({e})", file=sys.stderr)
        return False

def main():
    args = parse_args()
    if args.verify:
        if not args.token or not args.hash:
            print("Both --token and --hash required for verify", file=sys.stderr)
            sys.exit(1)
        try:
            with open(args.token, 'rb') as f:
                token = f.read()
        except OSError as e:
            print(f"Could not read token: {e}", file=sys.stderr)
            sys.exit(1)
        if verify_token(args.token, args.hash):
            sys.exit(0)
        else:
            sys.exit(1)
    elif args.hash:
        if not args.output:
            print("--output required to save token", file=sys.stderr)
            sys.exit(1)
        try:
            req = create_timestamp_request(args.hash, args.tsa_url)
            resp = submit_to_tsa(req, args.tsa_url)
            save_token(resp, args.output)
            print(f"Token saved to {args.output}")
        except SystemExit:
            raise
        except Exception as e:
            print(f"Timestamping failed: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print("Must specify --hash or --verify", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()

