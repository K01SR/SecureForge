use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriveType {
    HDD,
    SSD,
    NVMe,
    USB,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    pub serial: String,
    pub drive_type: DriveType,
    pub is_mounted: bool,
    pub mount_points: Vec<String>,
    pub is_system_drive: bool,
    pub smart_status: SmartStatus,
}

#[derive(Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Deserialize)]
struct LsblkDevice {
    name: String,
    size: u64,
    #[serde(rename = "type")]
    dev_type: String,
    mountpoint: Option<String>,
    mountpoints: Option<Vec<Option<String>>>,
    // vendor: Option<String>,
    model: Option<String>,
    serial: Option<String>,
    rota: Option<String>,
}

fn determine_drive_type(name: &str, rota: Option<&str>) -> DriveType {
    if name.starts_with("nvme") {
        return DriveType::NVMe;
    }
    if let Some(rota) = rota {
        if rota == "1" || rota == "true" {
            return DriveType::HDD;
        } else if rota == "0" || rota == "false" {
            return DriveType::SSD;
        }
    }
    DriveType::Unknown
}

#[tauri::command]
pub fn list_drives() -> Result<Vec<DriveInfo>, String> {
    let output = Command::new("lsblk")
        .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,MOUNTPOINTS,VENDOR,MODEL,SERIAL,ROTA"])
        .output()
        .map_err(|e| format!("Failed to run lsblk: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk output: {}", e))?;

    let mut drives = Vec::new();

    for dev in parsed.blockdevices {
        if dev.dev_type != "disk" {
            continue;
        }

        let drive_type = determine_drive_type(&dev.name, dev.rota.as_deref());
        let path = format!("/dev/{}", dev.name);
        
        let mut mount_points = Vec::new();
        if let Some(mp) = dev.mountpoint {
            mount_points.push(mp);
        }
        if let Some(mps) = dev.mountpoints {
            for mp in mps.into_iter().flatten() {
                if !mount_points.contains(&mp) {
                    mount_points.push(mp);
                }
            }
        }

        let is_mounted = !mount_points.is_empty();
        let is_system_drive = mount_points.contains(&"/".to_string()) || mount_points.contains(&"/boot".to_string()) || mount_points.contains(&"/boot/efi".to_string());

        let model = dev.model.unwrap_or_else(|| "Unknown".to_string());
        let serial = dev.serial.unwrap_or_else(|| "Unknown".to_string());
        
        drives.push(DriveInfo {
            name: dev.name,
            path,
            size_bytes: dev.size,
            model: model.trim().to_string(),
            serial: serial.trim().to_string(),
            drive_type,
            is_mounted,
            mount_points,
            is_system_drive,
            smart_status: SmartStatus::Unknown,
        });
    }

    Ok(drives)
}

#[derive(Deserialize)]
struct SmartctlOutput {
    smart_status: Option<SmartctlStatus>,
}

#[derive(Deserialize)]
struct SmartctlStatus {
    passed: bool,
}

#[tauri::command]
pub fn get_drive_info(device_path: String) -> Result<DriveInfo, String> {
    let drives = list_drives()?;
    let mut drive = drives.into_iter().find(|d| d.path == device_path)
        .ok_or_else(|| format!("Drive not found: {}", device_path))?;

    let output = Command::new("smartctl")
        .args(["-j", "-a", &device_path])
        .output();

    if let Ok(output) = output {
        if let Ok(parsed) = serde_json::from_slice::<SmartctlOutput>(&output.stdout) {
            if let Some(status) = parsed.smart_status {
                drive.smart_status = if status.passed {
                    SmartStatus::Healthy
                } else {
                    SmartStatus::Critical
                };
            } else {
                drive.smart_status = SmartStatus::Unknown;
            }
        }
    }

    Ok(drive)
}
