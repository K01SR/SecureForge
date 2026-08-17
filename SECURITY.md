# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :--- |
| latest main | ✅ |
| older releases | ❌ |

## Reporting a Vulnerability

SecureForge is a forensic security tool. We take security vulnerabilities extremely seriously.

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report vulnerabilities by emailing: **security@secureforge.dev**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a detailed response within 7 days.

## Security Considerations

As a forensic tool, SecureForge handles sensitive evidence. Key security design decisions:

1. **No telemetry or analytics** — zero data leaves the machine
2. **Air-gapped Live ISO mode** — network disabled by default
3. **Read-only evidence access** — kernel-enforced write blocking via O_RDONLY
4. **Expert Mode gating** — Argon2id passphrase required for destructive operations
5. **Report integrity** — SHA-256 hash chain prevents tampering
