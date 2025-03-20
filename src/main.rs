use attendance::manager::AttendanceManager;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::io::{self, BufRead};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// TODO: Add functionality to show data for a specific student.
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Actions to perform specific to a given week.
    Week(WeekArgs),
    /// Show the roster of students.
    ShowRoster,
    /// Show the info and attendance of a specific student,
    StudentInfo { id: String },
}

#[derive(Args, Debug, Clone)]
struct WeekArgs {
    /// The current week.
    week: i32,
    /// The action to perform for the given week.
    #[arg(value_enum)]
    command: WeekCommand,
}

#[derive(ValueEnum, Debug, Clone)]
enum WeekCommand {
    /// Reads student emails from stdin and marks those students as present for the given week.
    MarkPresent,
    /// Reads student emails from stdin and marks those students as excused for the given week.
    MarkExcused,
    /// Marks any remaining students as absent.
    MarkAbsent,
    /// Displays the attendance for a given week.
    ShowWeek,
}

fn main() {
    // Parse the command line args.
    let args = Cli::parse();

    let Command::Week(week_args) = args.command else {
        todo!("implement printing the entire roster");
    };

    let curr_week = week_args.week;

    if let WeekCommand::ShowWeek = week_args.command {
        attendance::show_week_attendance(curr_week);
        return;
    }

    if let WeekCommand::MarkAbsent = week_args.command {
        let mut manager = AttendanceManager::connect();
        manager
            .mark_remaining_absent(curr_week)
            .expect("Unable to mark students as absent");
        return;
    }

    let mut emails = vec![];

    // Read in data from stdin.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("Unable to read from stdin");
        emails.push(line);
    }

    // Convert all email addresses to IDs (verifying them along the way).
    let ids: Vec<&str> = emails
        .iter()
        .map(|email| {
            assert!(
                email.ends_with("@andrew.cmu.edu\n"),
                "email entry is invalid"
            );

            let i = email
                .find("@andrew.cmu.edu\n")
                .expect("We just checked that this is here");

            &email[0..i]
        })
        .collect();

    let mut manager = AttendanceManager::connect();

    match week_args.command {
        WeekCommand::MarkPresent => manager
            .mark_present(curr_week, &ids)
            .expect("Unable to mark students as present"),
        WeekCommand::MarkExcused => manager
            .mark_excused(curr_week, &ids)
            .expect("Unable to mark students as excused"),
        _ => (),
    }
}
