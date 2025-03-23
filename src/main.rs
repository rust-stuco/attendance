use attendance::manager::AttendanceManager;
use clap::{Args, Parser, Subcommand, ValueEnum};
use diesel::QueryResult;
use std::io::{self, BufRead};

/// A parser for the command line interface for this attendance application.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The different kinds of commands that can be run for this application.
    #[command(subcommand)]
    command: Command,
}

/// The different subcommands that can be run for this attendance application.
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Runs setup for a semester's attendance. ONLY RUN ONCE!
    Setup,
    /// Updates the roster of students via the [`ROSTER_PATH`].
    UpdateRoster,
    /// Show the roster of students.
    ShowRoster {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show every student who has been absent at least once, after a certain week.
    Absences {
        #[arg(short, long)]
        after_week: i32,
    },
    /// Show the info and attendance of a specific student,
    StudentInfo { id: String },
    /// Actions to perform specific to a given week.
    Week(WeekArgs),
    /// Email students with excessive absences after a given week.
    EmailAbsentees(EmailAbsenteesArgs),
}

/// The command-line arguments for doing actions given a specific week.
#[derive(Args, Debug, Clone)]
struct WeekArgs {
    /// The current week.
    week: i32,
    /// The action to perform for the given week.
    #[arg(value_enum)]
    command: WeekCommand,
}

/// The command-line arguments for emailing absentee students.
#[derive(Args, Debug, Clone)]
struct EmailAbsenteesArgs {
    /// The mode for emailing absentees
    #[arg(value_enum)]
    mode: EmailMode,

    /// The specific week for which to email absentees (for SingleWeek mode)
    /// Or the starting week to check from (for Cumulative mode)
    week: i32,

    /// The minimum number of absences to trigger an email (only for Cumulative mode)
    #[arg(short, long, required_if_eq("mode", "Cumulative"))]
    min_absences: Option<i32>,
}

/// The different modes for emailing absentees
#[derive(ValueEnum, Debug, Clone)]
enum EmailMode {
    /// Email students absent for a specific week
    SingleWeek,
    /// Email students with cumulative absences exceeding threshold after a specific week
    Cumulative,
}

/// The different kinds of actions that can be done for a specific week.
#[derive(ValueEnum, Debug, Clone)]
enum WeekCommand {
    /// Reads student emails from stdin and marks those students as present for the given week.
    MarkPresent,
    /// Reads student emails from stdin and marks those students as excused for the given week.
    MarkExcused,
    /// Marks any remaining students as absent.
    MarkAbsent,
    /// Displays the attendance for the given week.
    ShowWeek,
    /// Resets / deletes all attendance records for the given week.
    Reset,
}

fn main() -> QueryResult<()> {
    let args = Cli::parse();

    match args.command {
        Command::Setup => attendance::setup(),
        Command::UpdateRoster => attendance::update_roster(),
        Command::ShowRoster { verbose } => attendance::display::show_roster(verbose),
        Command::Absences { after_week } => attendance::display::show_absences(after_week),
        Command::StudentInfo { id } => attendance::display::show_student_info(&id),
        Command::Week(week_args) => run_week_command(week_args),
        Command::EmailAbsentees(email_args) => match email_args.mode {
            EmailMode::SingleWeek => attendance::mailer::email_weekly_absentees(email_args.week),
            EmailMode::Cumulative => attendance::mailer::email_cumulative_absentees(
                email_args.week,
                email_args.min_absences.unwrap_or(2), // Should always be present due to required_if_eq
            ),
        },
    }
}

/// A helper function for running the week-specific subcommands.
fn run_week_command(week_args: WeekArgs) -> QueryResult<()> {
    let curr_week = week_args.week;

    match week_args.command {
        WeekCommand::ShowWeek => {
            attendance::display::show_week_attendance(curr_week)?;
            return Ok(());
        }
        WeekCommand::MarkAbsent => {
            AttendanceManager::connect().mark_remaining_absent(curr_week)?;
            return Ok(());
        }
        WeekCommand::Reset => {
            AttendanceManager::connect().delete_week_attendance(curr_week)?;
            return Ok(());
        }
        WeekCommand::MarkPresent | WeekCommand::MarkExcused => (),
    };

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
                email.trim().ends_with("@andrew.cmu.edu"),
                "email entry is invalid: '{email}'"
            );

            let i = email
                .find("@andrew.cmu.edu")
                .expect("We just checked that this is here");

            &email[0..i]
        })
        .collect();

    let mut manager = AttendanceManager::connect();

    match week_args.command {
        WeekCommand::MarkPresent => manager.mark_present(curr_week, &ids),
        WeekCommand::MarkExcused => manager.mark_excused(curr_week, &ids),
        _ => unreachable!("we checked for the other variants above"),
    }
    .map(|_| ())
}
