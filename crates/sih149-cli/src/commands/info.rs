use clap::Args;
use std::process::Command;
use crate::display;

#[derive(Args)]
pub struct InfoArgs {
    #[arg(short, long)] pub device: Option<String>,
    #[arg(long)] pub json: bool,
}

pub fn run(args: &InfoArgs) -> anyhow::Result<()> {
    let output = Command::new("lsblk")
        .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,VENDOR,MODEL,SERIAL,ROTA"])
        .output()?;
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)?;
    
    let blockdevices = parsed["blockdevices"].as_array().unwrap();
    
    if args.json {
        println!("{}", serde_json::to_string_pretty(blockdevices)?);
    } else {
        display::print_section("System Drives");
        display::print_drive_table(blockdevices);
    }
    
    if let Some(dev) = &args.device {
        display::print_section(&format!("SMART Info for {}", dev));
        let smart_out = Command::new("smartctl").args(["-j", "-a", dev]).output();
        match smart_out {
            Ok(o) => {
                let s_out = String::from_utf8_lossy(&o.stdout);
                let s_parsed: serde_json::Value = serde_json::from_str(&s_out).unwrap_or(serde_json::json!({}));
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&s_parsed)?);
                } else {
                    println!("Health Status: {:?}", s_parsed["smart_status"]["passed"]);
                }
            },
            Err(e) => {
                display::print_error(&format!("Failed to run smartctl: {}", e));
            }
        }
    }
    
    Ok(())
}
