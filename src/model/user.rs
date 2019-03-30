use std::fmt;

use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, prelude::*};
use uuid::Uuid;

pub use model::activation_state::{ActivationState, AccountActivationState};
pub use schema::users;

/// NewUser
#[derive(Debug, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub password: Vec<u8>,
}

impl fmt::Display for NewUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<NewUser {email}>", email = self.email)
    }
}

#[derive(Debug, Deserialize, Identifiable, Serialize)]
pub struct User {
    pub id: i64,
    pub uuid: Uuid,
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: Vec<u8>,
    pub activation_state: ActivationState,
    pub access_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub reset_password_token: Option<String>,
    pub reset_password_token_expires_at: Option<NaiveDateTime>,
    pub reset_password_token_sent_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<User {uuid}>", uuid = self.uuid)
    }
}

impl User {
    pub fn verify_password(&self, _password: &str) -> bool {
        // TODO
        true
    }
}

impl User {
    pub fn find_by_email_or_username(
        _s: &str,
        _conn: &PgConnection,
    ) -> Option<Self>
    {
        // TODO
        None
    }
}
