use crate::models::{Attendance, Status, Student, Week};
use crate::schema;
use chrono::{Days, NaiveDate};
use diesel::prelude::*;
use diesel::result::QueryResult;
use dotenvy::dotenv;
use std::env;

/// The manager for recording, modifying, and retrieving attendance data.
pub struct AttendanceManager {
    db: SqliteConnection,
}

impl AttendanceManager {
    /// Creates a new `AttendanceManager` by connecting to the a `sqlite3` instance located at the
    /// `DATABASE_URL` environment variable.
    pub fn connect() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let connection = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self { db: connection }
    }

    /// Retrieves all students on the roster.
    pub fn get_roster(&mut self) -> QueryResult<Vec<Student>> {
        use schema::students::dsl::*;

        students.select(Student::as_select()).load(&mut self.db)
    }

    /// Removes all students from the roster.
    pub fn delete_roster(&mut self) -> QueryResult<Vec<Student>> {
        diesel::delete(schema::students::table)
            .returning(Student::as_returning())
            .get_results(&mut self.db)
    }

    /// Inserts students into the database.
    pub fn insert_students(&mut self, new_students: &[Student]) -> QueryResult<()> {
        let students_inserted = diesel::insert_into(schema::students::table)
            .values(new_students)
            .execute(&mut self.db)?;

        assert_eq!(students_inserted, new_students.len());

        Ok(())
    }

    /// Removes a student from the roster given their ID.
    pub fn delete_student(&mut self, student_id: &str) -> QueryResult<Student> {
        use schema::students::dsl::*;

        let mut deleted_students = diesel::delete(schema::students::table)
            .filter(id.eq(student_id))
            .returning(Student::as_returning())
            .get_results(&mut self.db)?;

        assert_eq!(
            deleted_students.len(),
            1,
            "there shoudl only be 1 student per ID"
        );

        Ok(deleted_students
            .pop()
            .expect("we just checked this was not empty"))
    }

    /// Given the starting date and the list of valid weeks (since not all weeks may need to take
    /// attendance), initializes the list of dates for valid weeks.
    ///
    /// The number of records that will be inserted into the `weeks` table will be equal to the
    /// number of valid weeks passed in.
    pub fn initialize_weeks(
        &mut self,
        start_date: NaiveDate,
        valid_weeks: &[bool],
    ) -> QueryResult<()> {
        /// The number of days in a week.
        const WEEK_DAYS: Days = Days::new(7);

        let total_weeks = valid_weeks
            .iter()
            .filter(|&&is_valid_week| is_valid_week)
            .count();

        let mut dates = vec![];
        let mut curr_date = start_date;

        // Add dates for every week, skipping invalid weeks.
        valid_weeks.iter().for_each(|&is_valid_week| {
            curr_date = curr_date
                .checked_add_days(WEEK_DAYS)
                .expect("Somehow reached the end of time");

            if is_valid_week {
                let week = Week {
                    id: dates.len() as i32,
                    date: curr_date,
                };

                dates.push(week);
            }
        });

        assert_eq!(dates.len(), total_weeks);

        // Store the dates in the `weeks` table.
        let weeks_inserted = diesel::insert_into(schema::weeks::table)
            .values(dates)
            .execute(&mut self.db)?;

        assert_eq!(weeks_inserted, total_weeks);

        Ok(())
    }

    fn mark(&mut self, week: i32, student_ids: &[&str], status: Status) -> QueryResult<()> {
        let records: Vec<Attendance> = student_ids
            .iter()
            .map(|id| Attendance {
                student: id.to_string(),
                week,
                status,
            })
            .collect();

        // TODO: If the primary key already exists, then this needs to be an update, not an insert.
        let records_inserted = diesel::insert_into(schema::attendance::table)
            .values(records)
            .execute(&mut self.db)?;

        assert_eq!(records_inserted, student_ids.len());

        Ok(())
    }

    /// Mark all of the given students as [`Status::Present`].
    pub fn mark_present(&mut self, week: i32, student_ids: &[&str]) -> QueryResult<()> {
        self.mark(week, student_ids, Status::Present)
    }

    /// Mark all of the given students as [`Status::Excused`].
    pub fn mark_excused(&mut self, week: i32, student_ids: &[&str]) -> QueryResult<()> {
        self.mark(week, student_ids, Status::Excused)
    }

    /// Given a specific week, mark every student who has not been marked as either
    /// [`Status::Present`] or [`Status::Excused`] as [`Status::Absent`].
    pub fn mark_absent(&mut self, week: i32) -> QueryResult<()> {
        let roster = self.get_roster()?;

        // Query for every record in the `attendance` table for the specific week.
        // Find the people who have not been marked as present or excused.
        // Mark them as absent.

        todo!()

        // self.mark(week, &[], Status::Absent)
    }
}

impl Default for AttendanceManager {
    fn default() -> Self {
        Self::connect()
    }
}
