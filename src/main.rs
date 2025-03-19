use anyhow::Result;
use clap::Parser;

use absentee_tracker::cli::Cli;
use absentee_tracker::create_default_manager;

/// Runs the absentee tracker application.
fn main() -> Result<()> {
    let mut manager = create_default_manager()?;

    let cli = Cli::parse();

    manager.run_command(&cli.command)?;

    Ok(())
}
