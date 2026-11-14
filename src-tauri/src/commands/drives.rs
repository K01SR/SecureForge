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
    #[serde(default)]
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Deserialize)]
struct LsblkDevice {
    name: String,
    #[serde(default)]
    size: Option<serde_json::Value>,
    #[serde(rename = "type", default)]
    dev_type: Option<String>,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    serial: Option<String>,
    #[serde(default)]
    rota: Option<serde_json::Value>,
    #[serde(default)]
    children: Option<Vec<LsblkDevice>>,
}

fn determine_drive_type(name: &str, rota: Option<&serde_json::Value>) -> DriveType {
    if name.starts_with("nvme") {
        return DriveType::NVMe;
    }
    if let Some(r) = rota {
        if r.as_bool() == Some(true)
            || r.as_str() == Some("1")
            || r.as_str() == Some("true")
            || r.as_i64() == Some(1)
            || r.as_u64() == Some(1)
        {
            return DriveType::HDD;
        } else if r.as_bool() == Some(false)
            || r.as_str() == Some("0")
            || r.as_str() == Some("false")
            || r.as_i64() == Some(0)
            || r.as_u64() == Some(0)
        {
            return DriveType::SSD;
        }
    }
    DriveType::Unknown
}

fn extract_size_bytes(val: Option<&serde_json::Value>) -> u64 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn collect_mount_points(dev: &LsblkDevice, out: &mut Vec<String>) {
    if let Some(ref mp) = dev.mountpoint {
        if !mp.is_empty() && !out.contains(mp) {
            out.push(mp.clone());
        }
    }
    if let Some(ref mps) = dev.mountpoints {
        for mp in mps.iter().flatten() {
            if !mp.is_empty() && !out.contains(mp) {
                out.push(mp.clone());
            }
        }
    }
    if let Some(ref children) = dev.children {
        for child in children {
            collect_mount_points(child, out);
        }
    }
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
        if dev.dev_type.as_deref() != Some("disk") {
            continue;
        }

        let drive_type = determine_drive_type(&dev.name, dev.rota.as_ref());
        let path = format!("/dev/{}", dev.name);
        
        let mut mount_points = Vec::new();
        collect_mount_points(&dev, &mut mount_points);

        let is_mounted = !mount_points.is_empty();
        let is_system_drive = mount_points.contains(&"/".to_string())
            || mount_points.contains(&"/boot".to_string())
            || mount_points.contains(&"/boot/efi".to_string())
            || mount_points.contains(&"/home".to_string());

        let model = dev.model.unwrap_or_else(|| "Unknown".to_string());
        let serial = dev.serial.unwrap_or_else(|| "Unknown".to_string());
        let size_bytes = extract_size_bytes(dev.size.as_ref());

        drives.push(DriveInfo {
            name: dev.name,
            path,
            size_bytes,
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
