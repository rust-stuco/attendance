use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

use self::models::{NewStudent, Student};

pub fn create_student(conn: &mut SqliteConnection, andrew_id: &str, name: &str) -> Student {
    let new_student = NewStudent { andrew_id, name };

    diesel::insert_into(schema::students::table)
        .values(&new_student)
        .returning(Student::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}
