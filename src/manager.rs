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

    /// Retrieves the IDs of all students on the roster.
    pub fn get_roster_ids(&mut self) -> QueryResult<Vec<String>> {
        Ok(self
            .get_roster()?
            .into_iter()
            .map(|student| student.id.to_string())
            .collect())
    }

    /// Removes and returns all students from the roster.
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

    /// Removes and returns a student from the roster given their ID.
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

    /// For a given week, mark all of the given students with the given [`Status`]. If that record
    /// already exists, this will update that [`Status`].
    ///
    /// If `student_ids` contains an ID that is not on the roster, this function will ignore it.
    fn mark(&mut self, week: i32, student_ids: &[&str], status: Status) -> QueryResult<()> {
        let roster = self.get_roster_ids()?;

        // Note that we can't use `.contains` here beacuse roster is `Vec<String>`, not `Vec<&str>`.
        let records: Vec<Attendance> = student_ids
            .iter()
            .filter(|&id| {
                if roster.iter().any(|s| s == id) {
                    true
                } else {
                    eprintln!("Tried to mark an unknown student {} as {:?}", id, status);
                    false
                }
            })
            .map(|id| Attendance {
                student: id.to_string(),
                week,
                status,
            })
            .collect();

        // Mark the students with the given status.
        // If the record already exists, this simply updates the status.
        let records_inserted = diesel::replace_into(schema::attendance::table)
            .values(records)
            .execute(&mut self.db)?;

        assert_eq!(records_inserted, student_ids.len());

        Ok(())
    }

    /// For a given week, mark all of the given students as [`Status::Present`].
    ///
    /// If `student_ids` contains an ID that is not on the roster, this function will ignore it.
    pub fn mark_present(&mut self, week: i32, student_ids: &[&str]) -> QueryResult<()> {
        self.mark(week, student_ids, Status::Present)
    }

    /// For a given week, mark all of the given students as [`Status::Excused`].
    ///
    /// If `student_ids` contains an ID that is not on the roster, this function will ignore it.
    pub fn mark_excused(&mut self, week: i32, student_ids: &[&str]) -> QueryResult<()> {
        self.mark(week, student_ids, Status::Excused)
    }

    /// For a given week, mark every student who has not been marked as either [`Status::Present`]
    /// or [`Status::Excused`] as [`Status::Absent`].
    ///
    /// Returns the number of students that were marked absent.
    pub fn mark_remaining_absent(&mut self, week: i32) -> QueryResult<usize> {
        let roster = self.get_roster_ids()?;

        let records: Vec<Attendance> = roster
            .into_iter()
            .map(|student| Attendance {
                student,
                week,
                status: Status::Absent,
            })
            .collect();

        // Inserts absent records for every student, but if the record already exists, do nothing.
        diesel::insert_or_ignore_into(schema::attendance::table)
            .values(records)
            .execute(&mut self.db)
    }
}

impl Default for AttendanceManager {
    fn default() -> Self {
        Self::connect()
    }
}
