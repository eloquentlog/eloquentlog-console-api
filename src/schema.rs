table! {
    use diesel::sql_types::*;
    use model::message::{ELogFormat, ELogLevel};

    messages (id) {
        id -> Int8,
        code -> Nullable<Varchar>,
        lang -> Varchar,
        level -> ELogLevel,
        format -> ELogFormat,
        title -> Varchar,
        content -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel::pg::types::sql_types::Uuid;
    use model::user::{EUserState, EUserResetPasswordState};

    users (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        email -> Varchar,
        password -> Bytea,
        state -> EUserState,
        access_token -> Nullable<Varchar>,
        access_token_issued_at -> Nullable<Timestamp>,
        reset_password_state -> EUserResetPasswordState,
        reset_password_token -> Nullable<Varchar>,
        reset_password_token_expires_at -> Nullable<Timestamp>,
        reset_password_token_sent_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use model::user_email::{EUserEmailActivationState, EUserEmailRole};

    user_emails (id) {
        id -> Int8,
        user_id -> Int8,
        email -> Nullable<Varchar>,
        role -> EUserEmailRole,
        activation_state -> EUserEmailActivationState,
        activation_token -> Nullable<Varchar>,
        activation_token_expires_at -> Nullable<Timestamp>,
        activation_token_sent_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
