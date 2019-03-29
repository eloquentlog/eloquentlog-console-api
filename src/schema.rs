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

table! {
    use diesel::sql_types::*;
    use model::user::EUserActivationState;

    users (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        email -> Varchar,
        password -> Bytea,
        activation_state -> EUserActivationState,
        access_token -> Varchar,
        access_token_expires_at -> Timestamp,
        reset_password_token -> Nullable<Varchar>,
        reset_password_token_expires_at -> Nullable<Timestamp>,
        reset_password_token_sent_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
