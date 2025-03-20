//! Updates the roster of students.
//!
//! This binary will look at the roster provided in [`ROSTER_PATH`] and look at the diff between the
//! current roster stored in the database. It will then add / delete students according to the CSV
//! roster from [`ROSTER_PATH`].

use attendance::{manager::AttendanceManager, models::Student};
use diesel::result::QueryResult;

/// The path to the roster of students.
const ROSTER_PATH: &str = "../roster-s25.csv";

pub fn main() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Insert the students from the given roster.
    let new_roster = attendance::download_roster(ROSTER_PATH);

    let curr_roster = manager.get_roster()?;

    let dropped: Vec<&Student> = curr_roster
        .iter()
        .filter(|student| !new_roster.contains(student))
        .collect();
    println!("Students dropped: {:#?}", dropped);

    for student in dropped {
        manager.delete_student(&student.id)?;
    }

    let added: Vec<Student> = new_roster
        .iter()
        .filter(|student| !curr_roster.contains(student))
        .cloned()
        .collect();
    println!("Students added: {:#?}", added);

    manager.insert_students(&added)?;

    Ok(())
}
