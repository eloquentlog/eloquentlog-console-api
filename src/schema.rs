table! {
    use diesel::sql_types::*;
    use model::message::{LogFormat, LogLevel};

    messages (id) {
        id -> Int8,
        code -> Nullable<Varchar>,
        lang -> Varchar,
        level -> LogLevel,
        format -> LogFormat,
        title -> Varchar,
        content -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
