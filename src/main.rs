use attendance::manager::AttendanceManager;
use attendance::{self, models::Student};
use diesel::result::QueryResult;

fn main() -> QueryResult<()> {
    let mut manager = AttendanceManager::new();

    let new_student = Student {
        id: "cjtsui".to_string(),
        email: "cjtsui@andrew.cmu.edu".to_string(),
        first_name: "Connor".to_string(),
        last_name: "Tsui".to_string(),
        major: "CS".to_string(),
        class: 4,
        graduation_semester: "S25".to_string(),
    };

    manager.insert_students(&[new_student])?;

    Ok(())
}
