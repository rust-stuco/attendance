mod mailer;
mod roster;

use roster::AttendanceManager;
use config::Config;
use std::error::Error;

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
        },
        "remove-student" => {
            if args.len() < 3 {
                println!("Usage: {} remove-student <andrew_id>", args[0]);
                return Ok(());
            }
            manager.remove_student(&args[2])?;
            println!("Student removed successfully.");
        },
        "mark-excused" => {
            if args.len() < 3 {
                println!("Usage: {} mark-excused <andrew_id>", args[0]);
                return Ok(());
            }
            manager.mark_excused(&args[2])?;
            println!("Student marked as excused.");
        },
        "mark-attended" => {
            if args.len() < 3 {
                println!("Usage: {} mark-attended <andrew_id>", args[0]);
                return Ok(());
            }
            manager.mark_attended(&args[2])?;
            println!("Student marked as attended.");
        },
        "list-unexcused" => {
            let unexcused = manager.get_unexcused_absentees();
            println!("Unexcused absentees:");
            for (id, student) in unexcused {
                println!("{}: {} ({})", id, student.name, student.email);
            }
        },
        "email-unexcused" => {
            manager.email_unexcused_absentees()?;
        },
        "reset-week" => {
            manager.reset_weekly_data()?;
            println!("Weekly data reset successfully.");
        },
        "bump-week" => {
            manager.bump_week()?;
            println!("Week bumped successfully.");
        },
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  program add-student <andrew_id> <name> <email>");
    println!("  program remove-student <andrew_id>");
    println!("  program mark-excused <andrew_id>");
    println!("  program mark-attended <andrew_id>");
    println!("  program list-unexcused");
    println!("  program email-unexcused");
    println!("  program reset-week");
    println!("  program bump-week");
}