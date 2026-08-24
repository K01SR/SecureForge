# SecureForge Live ISO Builder

## Prerequisites

```bash
sudo apt install live-build
```

## Building

```bash
cd iso
sudo ./build.sh
```

## USB Partitioning

After flashing the ISO to a USB drive, create a second partition
for persistent report storage:

```bash
# Assuming USB is /dev/sdc and ISO occupies ~1.5GB
sudo parted /dev/sdc mkpart primary fat32 1.5GB 100%
sudo mkfs.vfat -F 32 -n RECOVERY_DATA /dev/sdc2
```

## Boot Modes

- **Desktop Mode**: Boots into Openbox + SecureForge GUI (default)
- **CLI Mode**: Press Ctrl+Alt+F2 for terminal access
- **Server Mode**: `sih149 --mode server --port 8443`
