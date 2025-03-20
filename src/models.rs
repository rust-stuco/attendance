use crate::schema::{attendance, students, weeks};
use chrono::NaiveDate;
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{AsExpression, FromSqlRow, sql_types::Text};
use std::fmt::Display;

/// The attendance record for a student for a specific week.
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = attendance)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Attendance {
    pub student: String,
    pub week: i32,
    pub status: Status,
}

/// An entry in the roster of students, representing a student in the class.
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = students)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Student {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub major: String,
    pub class: i32,
    pub graduation_semester: String,
}

/// The actual date of a given week during the semester.
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = weeks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Week {
    pub id: i32,
    pub date: NaiveDate,
}

#[derive(Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Status {
    Present,
    Excused,
    Absent,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Present => write!(f, "Present"),
            Status::Excused => write!(f, "Excused"),
            Status::Absent => write!(f, "Absent"),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "Present" => Ok(Status::Present),
            "Excused" => Ok(Status::Excused),
            "Absent" => Ok(Status::Absent),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

impl FromSql<Text, Sqlite> for Status {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Sqlite> for Status {
    fn to_sql<'a>(&'a self, out: &mut Output<'a, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.to_string());
        Ok(diesel::serialize::IsNull::No)
    }
}
