use attendance::manager::AttendanceManager;
use diesel::result::QueryResult;

static ROSTER_PATH: &str = "../roster-s25.csv";

pub fn main() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Delete the entire roster before importing a new one.
    let _ = manager.delete_roster();

    // Insert the students from the given roster.
    let students = attendance::download_roster(ROSTER_PATH);
    manager.insert_students(&students)?;

    let roster = manager.get_roster()?;
    println!("{:#?}", roster);

    Ok(())
}
