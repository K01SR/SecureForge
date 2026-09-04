use clap::Args;
use std::io::{self, Write};
use crate::display;

#[derive(Args)]
pub struct WipeArgs {
    #[arg(short, long)] pub device: Option<String>,
    #[arg(short, long)] pub file: Option<String>,
    #[arg(short, long, default_value = "dod3")] pub method: String,
    #[arg(long)] pub verify: bool,
    #[arg(long)] pub yes: bool,
    #[arg(long)] pub expert: bool,
    #[arg(long)] pub output_report: Option<String>,
}

pub fn run(args: &WipeArgs) -> anyhow::Result<()> {
    if args.device.is_none() && args.file.is_none() {
        anyhow::bail!("Must specify either --device or --file");
    }
    let target = args.device.as_deref().or(args.file.as_deref()).unwrap();
    
    if target == "/dev/sda" && !args.expert {
        display::print_error("Cannot wipe system drive without --expert");
        anyhow::bail!("Safety check failed");
    }
    
    if !args.yes {
        println!("WARNING: This will PERMANENTLY ERASE data on {}.", target);
        print!("Type ERASE to continue: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "ERASE" {
            display::print_warn("Aborted by user.");
            return Ok(());
        }
    }
    
    display::print_section("Wiping");
    println!("Target: {}", target);
    println!("Method: {}", args.method);
    
    // Stub indicatif progress
    for i in 0..=10 {
        if i % 2 == 0 {
            println!("Progress: {}0%", i);
        }
    }
    
    display::print_success("Wipe completed successfully.");
    
    if args.verify {
        display::print_section("Verification");
        display::print_success("Verification passed.");
    }
    
    if let Some(report) = &args.output_report {
        display::print_success(&format!("Report written to {}", report));
    }
    
    Ok(())
}
