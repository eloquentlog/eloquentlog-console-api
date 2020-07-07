table! {
    use diesel::sql_types::*;

    namespaces (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Varchar,
        description -> Nullable<VarChar>,
        streams_count -> Int8,
        archived_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    streams (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Varchar,
        namespace_id -> Int8,
        description -> Nullable<VarChar>,
        archived_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    use crate::model::message::{EAgentType, ELogFormat, ELogLevel};

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
        stream_id -> Int8,
        agent_id -> Int8,
        agent_type -> EAgentType,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel::pg::types::sql_types::Uuid;

    use crate::model::user::{EUserState, EUserResetPasswordState};

    users (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Varchar,
        password -> Bytea,
        state -> EUserState,
        reset_password_state -> EUserResetPasswordState,
        reset_password_token -> Nullable<Varchar>,
        reset_password_token_expires_at -> Nullable<Timestamp>,
        reset_password_token_granted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;

    use crate::model::user_email::{
        EUserEmailIdentificationState,
        EUserEmailRole,
    };

    user_emails (id) {
        id -> Int8,
        user_id -> Int8,
        email -> Nullable<Varchar>,
        role -> EUserEmailRole,
        identification_state -> EUserEmailIdentificationState,
        identification_token -> Nullable<Varchar>,
        identification_token_expires_at -> Nullable<Timestamp>,
        identification_token_granted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel::pg::types::sql_types::Uuid;

    use crate::model::access_token::{EAccessTokenState, EAgentType};

    access_tokens (id) {
        id -> Int8,
        uuid -> Uuid,
        agent_id -> Int8,
        agent_type -> EAgentType,
        name -> VarChar,
        token -> Nullable<Bytea>,
        state -> EAccessTokenState,
        revoked_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(user_emails -> users (user_id));
joinable!(streams -> namespaces (namespace_id));
allow_tables_to_appear_in_same_query!(users, user_emails);
allow_tables_to_appear_in_same_query!(users, access_tokens);
allow_tables_to_appear_in_same_query!(namespaces, streams);
