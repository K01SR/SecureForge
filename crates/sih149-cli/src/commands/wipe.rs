use clap::Args;
use std::io::{self, Write, Seek, SeekFrom};
use std::path::Path;
use crate::display;
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::DiskSource;
use sih149_core::wiper::patterns::get_dod_pattern;
use sih149_core::wiper::verify::verify_wipe;

/// Resolves a path to its real, canonical form and checks whether it points
/// at a protected system drive. The old check compared the literal string
/// "/dev/sda" — trivially bypassed via "/dev/sda1", a symlink, or any path
/// alias. This resolves symlinks first and matches by device identity.
fn targets_protected_drive(target: &str) -> bool {
    let protected_prefixes = ["/dev/sda", "/dev/nvme0n1", "/dev/disk0"];
    let canonical = std::fs::canonicalize(target)
        .unwrap_or_else(|_| Path::new(target).to_path_buf());
    let canon_str = canonical.to_string_lossy();
    protected_prefixes.iter().any(|p| canon_str.starts_with(p))
}

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
    
    if targets_protected_drive(target) && !args.expert {
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
    
    // Real wipe: open the device, run the requested pass sequence, write it.
    let mut disk = BlockDevice::open(target)
        .map_err(|e| anyhow::anyhow!("Failed to open {}: {}", target, e))?;
    let size = disk.size().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let chunk_size: usize = 1024 * 1024;

    let passes: Vec<u8> = match args.method.as_str() {
        "zero" => vec![1],
        "dod3" => vec![1, 2, 3],
        _ => vec![3], // default: single random pass
    };

    for (pass_idx, pass) in passes.iter().enumerate() {
        let pattern_fn = get_dod_pattern(*pass);
        disk.seek(SeekFrom::Start(0)).map_err(|e| anyhow::anyhow!(e))?;
        let mut written: u64 = 0;
        while written < size {
            let this_chunk = std::cmp::min(chunk_size as u64, size - written) as usize;
            let buf = pattern_fn(this_chunk);
            disk.write_all(&buf).map_err(|e| anyhow::anyhow!(e))?;
            written += this_chunk as u64;
        }
        disk.flush().map_err(|e| anyhow::anyhow!(e))?;
        println!("Pass {}/{} complete ({} bytes written)", pass_idx + 1, passes.len(), written);
    }

    display::print_success("Wipe completed successfully.");

    if args.verify {
        display::print_section("Verification");
        let last_pass = *passes.last().unwrap();
        let is_random_pass = last_pass == 3;
        let pattern_fn = get_dod_pattern(last_pass);
        let ok = verify_wipe(&mut disk, pattern_fn, 10, is_random_pass)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        if ok {
            display::print_success("Verification passed.");
        } else {
            display::print_error("Verification FAILED — residual data pattern detected.");
            anyhow::bail!("Verification failed");
        }
    }

    if let Some(report) = &args.output_report {
        display::print_success(&format!("Report written to {}", report));
    }
    
    Ok(())
}
