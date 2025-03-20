//! Runs setup for a semester's attendance. This binary should only be run once, at the beginning of
//! the semester. To update the roster, see `src/bin/update_roster.rs`.
//!
//! This binary will download the roster from the provided [`ROSTER_PATH`], and it will also set up
//! the `weeks` table with the correct starting date and weeks.
//!
//! IMPORTANT: Depending on the semester, [`VALID_WEEKS`] might have to change.

use attendance::manager::AttendanceManager;
use chrono::NaiveDate;
use diesel::result::QueryResult;

/// The path to the roster of students.
const ROSTER_PATH: &str = "../roster-s25.csv";

/// The date of the first day of attendance.
const START_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 1, 15).expect("date is not real");

/// The weeks that are valid.
const VALID_WEEKS: [bool; 15] = [
    true, true, true, true, true, true, true, false, true, true, true, true, true, true, true,
];

pub fn main() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Delete the entire roster before importing a new one.
    let _ = manager.delete_roster();

    // Insert the students from the given roster.
    let new_roster = attendance::download_roster(ROSTER_PATH);
    manager.insert_students(&new_roster)?;

    let roster = manager.get_roster()?;
    println!("{:#?}", roster);
    println!("{} students total", roster.len());

    // Initialize the weeks for this semester.
    manager.initialize_weeks(START_DATE, &VALID_WEEKS)?;

    Ok(())
}
