use clap::{Parser, Subcommand};

pub mod display;
pub mod commands;

#[derive(Parser)]
#[command(name = "sih149", about = "SecureForge — Sanitize. Recover. Certify.", version = "0.1.0", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, global = true, help = "Enable verbose logging")]
    verbose: bool,
    #[arg(long, global = true, help = "Enable expert mode features")]
    expert: bool,
}

#[derive(Subcommand)]
enum Commands {
    Info(commands::info::InfoArgs),
    Wipe(commands::wipe::WipeArgs),
    Recover(commands::recover::RecoverArgs),
    Report(commands::report::ReportArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        tracing_subscriber::fmt().init();
    }
    
    let is_json = match &cli.command {
        Commands::Info(args) => args.json,
        _ => false,
    };
    
    if !is_json {
        display::print_banner();
    }
    
    match &cli.command {
        Commands::Info(args) => commands::info::run(args),
        Commands::Wipe(args) => commands::wipe::run(args),
        Commands::Recover(args) => commands::recover::run(args),
        Commands::Report(args) => commands::report::run(args),
    }
}
