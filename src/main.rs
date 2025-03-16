mod mailer;
mod roster;

use clap::{Args, Parser, Subcommand};
use config::Config;
use roster::AttendanceManager;
use std::error::Error;

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CURRENT_WEEK: &str = "\x1b[1;32m"; // bright green

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new student to the roster
    AddStudent(AddStudentArgs),

    /// Remove a student from the roster
    RemoveStudent(StudentIdArg),

    /// Mark a student as excused for the current week
    MarkExcused(StudentIdArg),

    /// Mark a student as attended for the current week
    MarkAttended(StudentIdArg),

    /// Mark multiple students as attended from a file
    BulkMarkAttended(FilePathArg),

    /// List students with unexcused absences
    ListUnexcused,

    /// Email students with unexcused absences
    EmailUnexcused,

    /// Display the current week
    ShowWeek,

    /// Reset attendance data for the current week
    ResetWeek,

    /// Increment to the next week
    BumpWeek,

    /// Set the current week number
    SetWeek(WeekArg),

    /// Show aggregate absence statistics
    AggregateStats,

    /// Flag students at risk due to multiple absences
    FlagAtRisk,
}

#[derive(Args)]
struct AddStudentArgs {
    /// Student's Andrew ID
    andrew_id: String,

    /// Student's full name
    name: String,

    /// Student's email address
    email: String,
}

#[derive(Args)]
struct StudentIdArg {
    /// Student's Andrew ID
    andrew_id: String,
}

#[derive(Args)]
struct FilePathArg {
    /// Path to file containing Andrew IDs
    file_path: String,
}

#[derive(Args)]
struct WeekArg {
    /// Week number to set as current
    week_number: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let roster_path = settings.get_string("attendance_manager.roster_path")?;
    let weekly_data_path = settings.get_string("attendance_manager.weekly_data_path")?;

    let mut manager = AttendanceManager::new(&roster_path, &weekly_data_path)?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::AddStudent(args) => {
            manager.add_student(
                args.andrew_id.clone(),
                args.name.clone(),
                args.email.clone(),
            )?;
            println!("Student added successfully.");
        }
        Commands::RemoveStudent(args) => {
            manager.remove_student(&args.andrew_id)?;
            println!("Student removed successfully.");
        }
        Commands::MarkExcused(args) => {
            manager.mark_excused(&args.andrew_id)?;
            println!("Student marked as excused.");
            print_weekly_summary(&manager);
        }
        Commands::MarkAttended(args) => {
            manager.mark_attended(&args.andrew_id)?;
            println!("Student marked as attended.");
            print_weekly_summary(&manager);
        }
        Commands::BulkMarkAttended(args) => {
            manager.bulk_mark_attended(&args.file_path)?;
            println!("Bulk attendance marked successfully.");
            print_weekly_summary(&manager);
        }
        Commands::ListUnexcused => {
            let unexcused = manager.get_unexcused_absentees();
            println!("Unexcused absentees:");
            for (id, student) in unexcused {
                println!("{}: {} ({})", id, student.name, student.email);
            }
        }
        Commands::EmailUnexcused => {
            manager.email_unexcused_absentees()?;
        }
        Commands::ShowWeek => {
            print_weekly_summary(&manager);
        }
        Commands::BumpWeek => {
            manager.bump_week()?;
            println!("Week bumped successfully.");
            print_weekly_summary(&manager);
        }
        Commands::ResetWeek => {
            manager.reset_weekly_data()?;
            println!("Weekly data reset successfully.");
            print_weekly_summary(&manager);
        }
        Commands::SetWeek(args) => {
            match manager.set_current_week(args.week_number) {
                Ok(_) => println!("Set current week to {}", args.week_number),
                Err(e) => eprintln!("Error: {}", e),
            };
            print_weekly_summary(&manager);
        }
        Commands::AggregateStats => {
            let counts = manager.aggregate_unexcused();
            println!("Unexcused absences:");
            let mut sorted: Vec<_> = counts.iter().collect();
            // Sort by highest count first, then alphabetically by andrewid
            sorted.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(&b.0)));

            for (andrew_id, count) in sorted {
                println!("- {}: {} absences", andrew_id, count);
            }
        }
        Commands::FlagAtRisk => {
            let (_, warnings) = manager.aggregate_unexcused_with_warning(2);
            if !warnings.is_empty() {
                println!("Students at risk:");
                for (andrew_id, count) in warnings {
                    println!("! {} has {} unexcused absences", andrew_id, count);
                }
            }
        }
    }

    Ok(())
}

fn print_weekly_summary(manager: &AttendanceManager) {
    let current_week = manager.get_current_week();
    let weekly_summary = manager.get_weekly_summary();
    println!("\nWeekly Data Summary:");

    let mut weeks: Vec<_> = weekly_summary.iter().collect();
    weeks.sort_by_key(|(k, _)| *k);

    for (week, (excused, attended)) in weeks {
        let is_current = *week == current_week;

        let status = if is_current { " (current)" } else { "" };
        let week_color = if is_current {
            COLOR_CURRENT_WEEK
        } else {
            COLOR_RESET
        };
        println!(
            "{}Week {}{}: {} excused, {} attended",
            week_color, week, status, excused, attended
        );
    }
}
