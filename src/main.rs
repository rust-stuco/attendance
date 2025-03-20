use attendance::{self, models::Student};
use diesel::prelude::*;
use diesel::result::QueryResult;

fn main() -> QueryResult<()> {
    use attendance::schema::students::dsl::*;

    let mut connection = attendance::establish_connection();

    let _ = attendance::create_student(&mut connection, "cjtsui", "Connor");
    let _ = attendance::create_student(&mut connection, "abcde", "Ferris");

    let results = students
        .select(Student::as_select())
        .load(&mut connection)
        .expect("Error loading students");

    println!("Displaying {} students", results.len());
    for student in results {
        println!("{} ({})", student.andrew_id, student.name);
    }

    Ok(())
}
