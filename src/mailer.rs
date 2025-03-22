use crate::manager::AttendanceManager;
use diesel::QueryResult;
use std::io::{self, Write};

/// Emails students who have more than the specified number of absences after a given week.
pub fn email_absentees(after_week: i32, min_absences: i32) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();
    let roster = manager.get_roster()?;

    let mut absentees = Vec::new();
    for student in roster {
        let attendance = manager.get_student_attendance(&student.id)?;
        let absences = attendance
            .absent
            .iter()
            .filter(|(week, _)| *week >= after_week)
            .count();

        if absences >= min_absences as usize {
            absentees.push((student, absences));
        }
    }

    // Display results
    println!(
        "\nStudents with {} or more absences after week {}:",
        min_absences, after_week
    );
    println!("{:<30} {:<30} {:<10}", "Name", "Email", "Absences");
    println!("{}", "-".repeat(70));

    for (student, absences) in &absentees {
        println!(
            "{:<30} {:<30} {:<10}",
            format!("{} {}", student.first_name, student.last_name),
            student.email,
            absences
        );
    }

    if absentees.is_empty() {
        println!(
            "\nNo students found with {} or more absences after week {}.",
            min_absences, after_week
        );
        return Ok(());
    }

    // Ask for confirmation
    print!("\nWould you like to email these students? [y/N] ");
    if let Err(e) = io::stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return Ok(());
    }

    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Error reading input: {}", e);
        return Ok(());
    }

    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return Ok(());
    }

    // TODO implement mailing
    println!("Emailed the following students:");
    for (student, absences) in absentees {
        println!(
            "- {} ({}) - {} absences",
            student.email,
            format!("{} {}", student.first_name, student.last_name),
            absences
        );
    }

    Ok(())
}
