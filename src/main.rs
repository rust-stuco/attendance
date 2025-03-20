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

    let other_student = Student {
        id: "ferris".to_string(),
        email: "ferris@andrew.cmu.edu".to_string(),
        first_name: "Ferris".to_string(),
        last_name: "The Crab".to_string(),
        major: "ECE".to_string(),
        class: 2,
        graduation_semester: "S27".to_string(),
    };

    manager.insert_students(&[new_student, other_student])?;

    let students = manager.roster()?;
    println!("{:?}", students);

    let removed_students = manager.remove_all_students()?;
    assert_eq!(students, removed_students);

    Ok(())
}
