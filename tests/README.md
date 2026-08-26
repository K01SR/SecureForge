# SecureForge Test Suite

## Structure

- `fixtures/` — Test disk images and sample files
- `integration/` — End-to-end integration tests
- `output/` — Test output directory (gitignored)

## Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires sudo for block device access)
sudo cargo test --test integration

# Generate test disk images
cd fixtures && ./create_test_images.sh
```

## Virtual Block Devices

Integration tests use loopback devices to simulate real drives
without risking actual hardware:

```bash
dd if=/dev/urandom of=test_disk.dd bs=1M count=128
sudo losetup /dev/loop0 test_disk.dd
# test against /dev/loop0
sudo losetup -d /dev/loop0
```
