use csv::Reader;
use std::path::Path;

pub mod manager;
pub mod models;
pub mod schema;

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
