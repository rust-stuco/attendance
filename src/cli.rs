use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
pub struct AddStudentArgs {
    /// Student's Andrew ID
    pub andrew_id: String,

    /// Student's full name
    pub name: String,

    /// Student's email address
    pub email: String,
}

#[derive(Args)]
pub struct StudentIdArg {
    /// Student's Andrew ID
    pub andrew_id: String,
}

#[derive(Args)]
pub struct FilePathArg {
    /// Path to file containing Andrew IDs
    pub file_path: String,
}

#[derive(Args)]
pub struct WeekArg {
    /// Week number to set as current
    pub week_number: u32,
}
