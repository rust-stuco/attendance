// @generated automatically by Diesel CLI.

diesel::table! {
    attendance (student, week) {
        student -> Text,
        week -> Int8,
        status -> Text,
    }
}

diesel::table! {
    students (id) {
        id -> Text,
        email -> Text,
        first_name -> Text,
        middle_initial -> Text,
        last_name -> Text,
        college -> Text,
        department -> Text,
        major -> Text,
        class -> Int8,
        graduation_semester -> Text,
    }
}

diesel::table! {
    weeks (id) {
        id -> Int8,
        date -> Date,
    }
}

diesel::joinable!(attendance -> students (student));
diesel::joinable!(attendance -> weeks (week));

diesel::allow_tables_to_appear_in_same_query!(
    attendance,
    students,
    weeks,
);
