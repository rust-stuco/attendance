use absentee_tracker::{establish_connection, models::Student};
use diesel::prelude::*;

fn main() {
    use absentee_tracker::schema::students::dsl::*;

    let connection = &mut establish_connection();
    let results = students
        .select(Student::as_select())
        .load(connection)
        .expect("Error loading students");

    println!("Displaying {} students", results.len());
    for student in results {
        println!("{}", student.andrew_id);
        println!("-----------\n");
        println!("{}", student.name);
    }
}
