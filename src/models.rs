use crate::schema::students;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = students)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Student {
    pub andrew_id: String,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = students)]
pub struct NewStudent<'a> {
    pub andrew_id: &'a str,
    pub name: &'a str,
}
