use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Dashboard,
    DriveManager,
    WipeWizard,
    Carver,
    Help,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WipeMethod {
    Zero,
    Random,
    Dod3,
    Dod7,
    Nist,
    Gutmann,
}

impl WipeMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Zero => "Zero (Single Pass)",
            Self::Random => "Random (PRNG)",
            Self::Dod3 => "DoD 5220.22-M 3-Pass",
            Self::Dod7 => "DoD 5220.22-M 7-Pass",
            Self::Nist => "NIST SP 800-88 Clear",
            Self::Gutmann => "Gutmann 35-Pass",
        }
    }
    pub fn all() -> Vec<WipeMethod> {
        vec![Self::Zero, Self::Random, Self::Dod3, Self::Dod7, Self::Nist, Self::Gutmann]
    }
}

#[derive(Clone, Debug)]
pub struct DriveEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    pub is_system: bool,
    pub drive_type: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WipePhase {
    Idle,
    Confirming,
    Running { pass: usize, total_passes: usize, bytes_done: u64, bytes_total: u64, speed_mbps: f64, started: Instant },
    Verifying,
    Done { success: bool, elapsed: Duration },
    Error(String),
}

#[derive(Clone, Debug)]
pub struct CarverState {
    pub source: String,
    pub output_dir: String,
    pub min_confidence: u8,
    pub scanning: bool,
    pub progress: f64,
    pub found: Vec<String>,
    pub log: Vec<String>,
    pub cursor_field: usize,
}

impl Default for CarverState {
    fn default() -> Self {
        Self {
            source: String::new(),
            output_dir: dirs::home_dir()
                .map(|h| h.join("recovered").to_string_lossy().to_string())
                .unwrap_or_default(),
            min_confidence: 70,
            scanning: false,
            progress: 0.0,
            found: Vec::new(),
            log: Vec::new(),
            cursor_field: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Popup {
    None,
    Confirm { title: String, message: String },
    Error(String),
    /// Reserved for informational popups (e.g. scan results summary)
    Info(String),
}

pub struct App {
    pub screen: Screen,
    pub drives: Vec<DriveEntry>,
    pub drive_cursor: usize,
    pub selected_drive: Option<usize>,
    pub wipe_method_cursor: usize,
    pub wipe_verify: bool,
    pub wipe_expert: bool,
    pub wipe_phase: WipePhase,
    pub carver: CarverState,
    pub popup: Popup,
    pub log: Vec<String>,
    /// Tracks last background refresh for future auto-reload support
    #[allow(dead_code)]
    pub last_refresh: Instant,
    pub tick: u64,
    pub should_quit: bool,
    pub status_msg: Option<(String, Instant)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dashboard,
            drives: Vec::new(),
            drive_cursor: 0,
            selected_drive: None,
            wipe_method_cursor: 2, // DoD3 default
            wipe_verify: true,
            wipe_expert: false,
            wipe_phase: WipePhase::Idle,
            carver: CarverState::default(),
            popup: Popup::None,
            log: Vec::new(),
            last_refresh: Instant::now(),
            tick: 0,
            should_quit: false,
            status_msg: None,
        }
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log.push(m);
        if self.log.len() > 200 { self.log.remove(0); }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now()));
    }

    pub fn selected_method(&self) -> WipeMethod {
        WipeMethod::all().into_iter().nth(self.wipe_method_cursor).unwrap_or(WipeMethod::Dod3)
    }

    pub fn load_drives(&mut self) {
        self.drives.clear();
        // Parse lsblk
        if let Ok(output) = std::process::Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,MODEL,ROTA"])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(bds) = json["blockdevices"].as_array() {
                        for bd in bds {
                            if bd["type"].as_str() != Some("disk") { continue; }
                            let name = bd["name"].as_str().unwrap_or("").to_string();
                            let path = format!("/dev/{}", name);
                            let size = bd["size"].as_u64().or_else(|| bd["size"].as_str().and_then(|s| s.parse().ok())).unwrap_or(0);
                            let model = bd["model"].as_str().unwrap_or("Unknown").trim().to_string();
                            let rota = bd["rota"].as_bool().unwrap_or(true);
                            let is_nvme = name.starts_with("nvme");
                            let dtype = if is_nvme { "NVMe" } else if !rota { "SSD" } else { "HDD" }.to_string();
                            // check system
                            let is_system = sih149_core::wiper::file_wiper::is_protected_drive(std::path::Path::new(&path));
                            self.drives.push(DriveEntry { name, path, size_bytes: size, model, is_system, drive_type: dtype });
                        }
                    }
                }
            }
        }
        if self.drives.is_empty() {
            // fallback: list /dev/sd* and /dev/nvme*
            for prefix in &["sda", "sdb", "sdc", "nvme0n1", "nvme1n1"] {
                let path = format!("/dev/{}", prefix);
                if std::path::Path::new(&path).exists() {
                    let is_system = sih149_core::wiper::file_wiper::is_protected_drive(std::path::Path::new(&path));
                    self.drives.push(DriveEntry {
                        name: prefix.to_string(), path: path.clone(),
                        size_bytes: 0, model: "Unknown".to_string(),
                        is_system, drive_type: "Disk".to_string(),
                    });
                }
            }
        }
        self.push_log(format!("Detected {} drive(s)", self.drives.len()));
    }
}
