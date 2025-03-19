//! This module contains the command-line interface [`Cli`] parser for managing student attendance
//! records.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::Student;

/// The command line configuration struct, where the command-line interface parser is automatically
/// derived by [`clap::Parser`].
#[derive(Parser, Debug)]
pub struct Cli {
    /// The different commands available for managing student attendance records.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add a new student to the roster.
    AddStudent(Student),

    /// Remove a student from the roster.
    RemoveStudent(Student),

    /// Mark a student as excused for the current week.
    MarkExcused(Student),

    /// Mark a student as attended for the current week.
    MarkAttended(Student),

    /// Mark multiple students as attended from a file.
    BulkMarkAttended { file_path: PathBuf },

    /// List students with unexcused absences.
    ListUnexcused,

    /// Email students with unexcused absences.
    EmailUnexcused,

    /// Display the current week.
    ShowWeek,

    /// Reset attendance data for the current week.
    ResetWeek,

    /// Increment to the next week.
    BumpWeek,

    /// Set the current week number.
    SetWeek { week: u8 },

    /// Show aggregate absence statistics.
    AggregateStats,

    /// Flag students at risk due to multiple absences.
    FlagAtRisk,
}
