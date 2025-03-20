use csv::Reader;
use std::path::Path;
use tabled::{Table, settings::Style};

pub mod manager;
pub mod models;
pub mod schema;

use manager::AttendanceManager;
use models::Student;

/// Downloads a student roster given a path to a CSV file.
///
/// # Panics
///
/// This function will panic if it is unable to find or read the file specified by the path, as well
/// as if it is unable to deserialize any of the records in the file.
pub fn download_roster<P: AsRef<Path>>(path: P) -> Vec<Student> {
    let mut csv = Reader::from_path(path).expect("unable to read from csv");

    csv.deserialize()
        .collect::<Result<Vec<Student>, _>>()
        .expect("unable to deserialize record")
}

/// Pretty prints the attendance data for a given week.
pub fn show_week_attendance(week: i32) {
    let mut manager = AttendanceManager::connect();

    let attendance = manager
        .get_week_attendance(week)
        .expect("Unable to show the week's data");

    let num_students = manager
        .num_students()
        .expect("Unable to get the number of students");
    assert_eq!(attendance.len(), num_students);

    let mut table = Table::new(attendance);
    table.with(Style::modern());

    println!("{table}");
}
