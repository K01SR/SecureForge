use clap::Args;
use std::io::{self, Write};
use std::path::Path;
use sih149_core::wiper::file_wiper::{FileWiper, is_protected_path};
use crate::display;

#[derive(Args)]
pub struct ShredArgs {
    /// File or directory to securely shred
    #[arg(short, long)]
    pub target: String,

    /// Number of overwrite passes (default: 3 for DoD-style)
    #[arg(short, long, default_value = "3")]
    pub passes: u32,

    /// Number of random renames before deletion (scrubs directory-entry history)
    #[arg(long, default_value = "8")]
    pub renames: u32,

    /// Also attempt to scrub slack space (requires raw device access — will error if unsupported)
    #[arg(long)]
    pub scrub_slack: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: &ShredArgs) -> anyhow::Result<()> {
    let target = Path::new(&args.target);

    if !target.exists() {
        anyhow::bail!("Target does not exist: {}", args.target);
    }

    // Canonicalize + protected-path check before the ERASE prompt so the
    // user can't race the guard by creating a symlink between prompt and exec.
    if is_protected_path(target) {
        anyhow::bail!(
            "Refusing to shred protected system path: {}\n\
             If this is genuinely intentional, use hdparm/nvme-cli for firmware erase instead.",
            args.target
        );
    }

    if !args.yes {
        println!(
            "WARNING: {} will be PERMANENTLY ERASED ({} passes, {} renames).",
            args.target, args.passes, args.renames
        );
        print!("Type ERASE to continue: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "ERASE" {
            display::print_warn("Aborted by user.");
            return Ok(());
        }
    }

    display::print_section("Shredding");
    println!("Target : {}", args.target);
    println!("Passes : {}", args.passes);
    println!("Renames: {}", args.renames);

    let wiper = FileWiper::new(args.passes, args.renames, args.scrub_slack);

    if target.is_dir() {
        let results = wiper
            .wipe_directory(target)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let total_bytes: u64 = results.iter().map(|r| r.bytes_wiped).sum();
        let file_count = results.len();
        display::print_success(&format!(
            "Shredded {} file(s), {} total.",
            file_count,
            display::format_bytes(total_bytes)
        ));
    } else {
        let result = wiper
            .wipe_file(target)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        display::print_success(&format!(
            "Shredded {} ({} in {} pass(es)).",
            args.target,
            display::format_bytes(result.bytes_wiped),
            result.passes_completed,
        ));
    }

    Ok(())
}
