use crate::manager::AttendanceManager;
use diesel::QueryResult;
use tabled::{Table, Tabled, settings::Style};

/// Pretty prints the attendance data for a given week.
pub fn show_week_attendance(week: i32) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    let attendance = manager.get_week_attendance(week)?;

    let num_students = manager.num_students()?;
    assert_eq!(attendance.len(), num_students);

    let mut table = Table::new(attendance);
    table.with(Style::modern());

    println!("Week {week} attendance:\n{table}");

    Ok(())
}

/// Pretty prints the attendance data for a given week.
pub fn show_roster(verbose: bool) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    let roster = manager.get_roster()?;

    let mut table = if verbose {
        Table::new(roster)
    } else {
        #[derive(Tabled)]
        struct SimpleStudent {
            id: String,
            first_name: String,
            last_name: String,
        }

        let simplified_roster: Vec<SimpleStudent> = roster
            .into_iter()
            .map(|student| SimpleStudent {
                id: student.id,
                first_name: student.first_name,
                last_name: student.last_name,
            })
            .collect();

        Table::new(simplified_roster)
    };

    table.with(Style::modern());
    println!("Roster:\n{table}");

    Ok(())
}

/// Prints all info about a student, including the number of lectures attended, excused, and absent.
pub fn show_student_info(student_id: &str) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Get the student info.
    let student = match manager.get_student(student_id) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Student with ID '{student_id}' not found.");
            return Ok(());
        }
    };

    // Print student information.
    println!("Student Information:\n{:#?}", student);

    // Get attendance records.
    let attendance = manager.get_student_attendance(student_id)?;

    println!("Attendance:\n{:#?}", attendance);

    Ok(())
}
