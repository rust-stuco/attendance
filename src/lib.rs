use chrono::NaiveDate;
use csv::Reader;
use diesel::QueryResult;
use std::path::Path;
use std::sync::OnceLock;

pub mod display;
pub mod mailer;
pub mod manager;
pub mod models;
pub mod schema;

use manager::AttendanceManager;
use models::Student;

use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    setup: SetupDetails,
}

#[derive(Debug, Deserialize)]
struct SetupDetails {
    roster_path: String,
    start_date: String,
    valid_weeks: Vec<bool>,
}

// Global config, lazy initialized with OnceLock
static CONFIG: OnceLock<SetupDetails> = OnceLock::new();

fn get_config() -> &'static SetupDetails {
    CONFIG.get_or_init(|| load_config().expect("Failed to load config"))
}

fn load_config() -> Result<SetupDetails, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;
    let app_config: AppConfig = settings.try_deserialize()?;
    Ok(app_config.setup)
}

/// A helper struct to carry information about a student's attendance.
#[derive(Debug, Clone)]
pub struct StudentAttendance {
    /// The dates where a student is present.
    pub present: Vec<(i32, NaiveDate)>,
    /// The dates where a student is excused.
    pub excused: Vec<(i32, NaiveDate)>,
    /// The dates where a student is absent.
    pub absent: Vec<(i32, NaiveDate)>,
}

/// Downloads a student roster given a path to a CSV file.
///
/// # Panics
///
/// This function will panic if it is unable to find or read the file specified by the path, as well
/// as if it is unable to deserialize any of the records in the file.
fn download_roster<P: AsRef<Path>>(path: P) -> Vec<Student> {
    let mut csv = Reader::from_path(path).expect("unable to read from csv");

    csv.deserialize()
        .collect::<Result<Vec<Student>, _>>()
        .expect("unable to deserialize record")
}

/// Runs setup for a semester's attendance.
///
/// This binary should ONLY be run once, at the beginning of the semester.
///
/// This binary will download the roster from the provided path in config, and it will also set up
/// the `weeks` table with the correct starting date and weeks from config.
///
/// IMPORTANT: Depending on the semester, [`VALID_WEEKS`] might have to change.
pub fn setup() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Delete the entire roster before importing a new one.
    let _ = manager.delete_roster();

    // Get config
    let config = get_config();

    // Insert the students from the given roster.
    let new_roster = download_roster(&config.roster_path);
    manager.insert_students(&new_roster)?;

    let roster = manager.get_roster()?;
    println!("{:#?}", roster);
    println!("{} students total", roster.len());

    // Parse the start date from config
    let start_date = NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d")
        .expect("Invalid start date format in config");

    // Initialize the weeks for this semester.
    manager.initialize_weeks(start_date, &config.valid_weeks)?;

    Ok(())
}

/// Updates the roster of students.
///
/// This binary will look at the roster provided in config and look at the diff between the
/// current roster stored in the database. It will then add / delete students according to the CSV
/// roster from config.
pub fn update_roster() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Get configuration
    let config = get_config();

    // Insert the students from the given roster.
    let new_roster = download_roster(&config.roster_path);

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
