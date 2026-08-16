# Contributing to SecureForge

Thank you for your interest in contributing to SecureForge! This document provides guidelines for contributing to the project.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Development Setup

### Prerequisites

- Rust 1.80+ (`rustup update stable`)
- Node.js 20+ and npm
- Python 3.11+
- Tauri CLI (`cargo install tauri-cli`)
- System packages: `libwebkit2gtk-4.1-dev`, `libewf-dev`, `nvme-cli`, `hdparm`, `smartmontools`

### Building

```bash
# Core engine and CLI
cargo build

# Frontend dependencies
cd src-ui && npm install

# Python pipeline
cd pipeline && pip install -r requirements.txt

# Full desktop app
cargo tauri dev
```

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
[optional footer]
```

### Types

| Type | Description |
| :--- | :--- |
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Code style (formatting, no logic change) |
| `refactor` | Code refactoring |
| `perf` | Performance improvement |
| `test` | Adding or modifying tests |
| `chore` | Build process, dependencies, tooling |
| `ci` | CI/CD changes |
| `security` | Security-related changes |

### Scopes

`core`, `cli`, `tauri`, `ui`, `pipeline`, `plugins`, `iso`, `docs`, `ci`

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Make your changes with conventional commits
4. Run tests: `cargo test && cd src-ui && npm test`
5. Run lints: `cargo clippy && cargo fmt --check`
6. Submit a PR against `main`

## Security

If you discover a security vulnerability, please report it responsibly. See [SECURITY.md](SECURITY.md).
