use clap::Args;
use crate::display;
use std::process::Command;

#[derive(Args)]
pub struct ReportArgs {
    #[arg(long)] pub list: bool,
    #[arg(long)] pub case_id: Option<String>,
    #[arg(long)] pub export: Option<String>,
    #[arg(long, default_value = "pdf")] pub format: String,
}

pub fn run(args: &ReportArgs) -> anyhow::Result<()> {
    if args.list {
        display::print_section("Cases");
        let path = dirs::data_local_dir().unwrap_or_default().join("secureforge/cases.db");
        if path.exists() {
            println!("Cases found in DB (stub)");
        } else {
            println!("No cases found.");
        }
        return Ok(());
    }
    
    if let Some(export) = &args.export {
        display::print_section("Exporting Report");
        let status = Command::new("python3")
            .args(["pipeline/report_gen.py", "--input", export, "--template", "erasure", "--output", "report_out.pdf"])
            .status();
        
        match status {
            Ok(s) if s.success() => display::print_success("Report exported successfully."),
            _ => display::print_error("Failed to export report."),
        }
    }
    
    Ok(())
}
