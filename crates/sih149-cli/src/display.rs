pub fn print_banner() {
    println!("========================================");
    println!("              SECUREFORGE               ");
    println!("========================================");
}

pub fn print_drive_table(drives: &[serde_json::Value]) {
    println!("{:<20} {:<15} {:<10}", "NAME", "SIZE", "TYPE");
    for d in drives {
        let name = d["name"].as_str().unwrap_or("Unknown");
        let size = d["size"].as_u64().unwrap_or(0);
        let typ = d["type"].as_str().unwrap_or("Unknown");
        println!("{:<20} {:<15} {:<10}", name, format_bytes(size), typ);
    }
}

pub fn print_success(msg: &str) {
    println!("\x1b[32m[+] {}\x1b[0m", msg);
}

pub fn print_error(msg: &str) {
    eprintln!("\x1b[31m[-] {}\x1b[0m", msg);
}

pub fn print_warn(msg: &str) {
    println!("\x1b[33m[!] {}\x1b[0m", msg);
}

pub fn print_section(title: &str) {
    println!("\x1b[1m{}\n{}\x1b[0m", title, "-".repeat(title.len()));
}

pub fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, units[unit_idx])
}

pub fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
