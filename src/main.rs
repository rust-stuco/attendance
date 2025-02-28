mod mailer;
mod roster;

use config::Config;
use roster::AttendanceManager;
use std::error::Error;

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CURRENT_WEEK: &str = "\x1b[1;32m"; // bright green

fn print_usage() {
    println!("Usage:");
    println!("  program add-student <andrew_id> <name> <email>");
    println!("  program remove-student <andrew_id>");
    println!("  program mark-excused <andrew_id>");
    println!("  program mark-attended <andrew_id>");
    println!("  program bulk-mark-attended <file_path>");
    println!("  program list-unexcused");
    println!("  program email-unexcused");
    println!("  program set-week <week_number>");
    println!("  program show-week");
    println!("  program reset-week");
    println!("  program bump-week");
    println!("  program aggregate-stats");
    println!("  program flag-at-risk");
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let roster_path = settings.get_string("attendance_manager.roster_path")?;
    let weekly_data_path = settings.get_string("attendance_manager.weekly_data_path")?;

    let args: Vec<String> = std::env::args().collect();
    let mut manager = AttendanceManager::new(&roster_path, &weekly_data_path)?;

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "add-student" => {
            if args.len() < 5 {
                println!("Usage: {} add-student <andrew_id> <name> <email>", args[0]);
                return Ok(());
            }
            manager.add_student(args[2].clone(), args[3].clone(), args[4].clone())?;
            println!("Student added successfully.");
        }
        "remove-student" => {
            if args.len() < 3 {
                println!("Usage: {} remove-student <andrew_id>", args[0]);
                return Ok(());
            }
            manager.remove_student(&args[2])?;
            println!("Student removed successfully.");
        }
        "mark-excused" => {
            if args.len() < 3 {
                println!("Usage: {} mark-excused <andrew_id>", args[0]);
                return Ok(());
            }
            manager.mark_excused(&args[2])?;
            println!("Student marked as excused.");
            print_weekly_summary(&manager);
        }
        "mark-attended" => {
            if args.len() < 3 {
                println!("Usage: {} mark-attended <andrew_id>", args[0]);
                return Ok(());
            }
            manager.mark_attended(&args[2])?;
            println!("Student marked as attended.");
            print_weekly_summary(&manager);
        }
        "bulk-mark-attended" => {
            if args.len() < 3 {
                println!("Usage: {} bulk-mark-attended <file_path>", args[0]);
                return Ok(());
            }
            let path = &args[2];
            manager.bulk_mark_attended(path)?;
            println!("Bulk attendance marked successfully.");
            print_weekly_summary(&manager);
        }
        "list-unexcused" => {
            let unexcused = manager.get_unexcused_absentees();
            println!("Unexcused absentees:");
            for (id, student) in unexcused {
                println!("{}: {} ({})", id, student.name, student.email);
            }
        }
        "email-unexcused" => {
            manager.email_unexcused_absentees()?;
        }
        "show-week" => {
            print_weekly_summary(&manager);
        }
        "bump-week" => {
            manager.bump_week()?;
            println!("Week bumped successfully.");
            print_weekly_summary(&manager);
        }
        "reset-week" => {
            manager.reset_weekly_data()?;
            println!("Weekly data reset successfully.");
            print_weekly_summary(&manager);
        }
        "set-week" => {
            if args.len() < 3 {
                println!("Usage: {} set-week <new_week_number>", args[0]);
                return Ok(());
            }
            let new_week: u32 = args[2].parse()?;
            match manager.set_current_week(new_week) {
                Ok(_) => println!("Set current week to {}", new_week),
                Err(e) => eprintln!("Error: {}", e),
            };
            print_weekly_summary(&manager);
        }
        "aggregate-stats" => {
            let counts = manager.aggregate_unexcused();
            println!("Unexcused absences:");
            let mut sorted: Vec<_> = counts.iter().collect();
            // Sort by highest count first, then alphabetically by andrewid
            sorted.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(&b.0)));

            for (andrew_id, count) in sorted {
                println!("- {}: {} absences", andrew_id, count);
            }
        }
        "flag-at-risk" => {
            let (_, warnings) = manager.aggregate_unexcused_with_warning(2);
            if !warnings.is_empty() {
                println!("Students at risk:");
                for (andrew_id, count) in warnings {
                    println!("! {} has {} unexcused absences", andrew_id, count);
                }
            }
        }
        _ => {
            print_usage();
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
