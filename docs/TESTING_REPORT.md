# SecureForge Testing Report

## 1. Test Strategy
- **Unit Tests:** `cargo test` covering isolated modules (entropy calculation, PRNG generation).
- **Integration Tests:** End-to-end IPC tests with Python pipeline mock data.
- **Manual Hardware Tests:** Conducted on physical SATA SSDs, NVMe drives, and USB flash media.
- **CI/CD Automation:** GitHub Actions ensures compilation and format checks across Linux environments.

