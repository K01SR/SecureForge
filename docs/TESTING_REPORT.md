# SecureForge Testing Report

## 1. Test Strategy
- **Unit Tests:** `cargo test` covering isolated modules (entropy calculation, PRNG generation).
- **Integration Tests:** End-to-end IPC tests with Python pipeline mock data.
- **Manual Hardware Tests:** Conducted on physical SATA SSDs, NVMe drives, and USB flash media.
- **CI/CD Automation:** GitHub Actions ensures compilation and format checks across Linux environments.

## 2. Test Results Table
| Module | Component | Environment | Status |
|---|---|---|---|
| Core | Disk Reader | Linux 6.x | PASS |
| Core | Carving Engine | Linux 6.x | PASS |
| UI | Reactivity | Tauri / WebKit | PASS |
| Wipe | NVMe Crypto Erase | Ubuntu 22.04 | PASS |
| Wipe | DoD 3-Pass | VirtualBox | PASS |

## 3. Known Issues & Mitigations
- **Issue:** ATA unfreeze command fails on certain older motherboards.
- **Mitigation:** Hot-plug the SATA drive post-boot or suspend/resume the host machine before executing sanitization.

