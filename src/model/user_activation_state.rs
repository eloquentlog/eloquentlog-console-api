//! # A type UserActivationState for User in user.rs
//!
//! EUserActivationState represents SQL type value `e_user_activation_state`
//! and UserActivationState is an Enum holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "e_user_activation_state")]
pub struct EUserActivationState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserActivationState"]
pub enum UserActivationState {
    Pending, // default
    Active,
}

impl fmt::Display for UserActivationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserActivationState::Pending => write!(f, "pending"),
            UserActivationState::Active => write!(f, "active"),
        }
    }
}

impl ToSql<EUserActivationState, Pg> for UserActivationState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            UserActivationState::Pending => out.write_all(b"pending")?,
            UserActivationState::Active => out.write_all(b"active")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserActivationState, Pg> for UserActivationState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(UserActivationState::Pending),
            b"active" => Ok(UserActivationState::Active),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserActivationState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => UserActivationState::Pending,
            "active" => UserActivationState::Active,
            _ => UserActivationState::Pending,
        }
    }
}

impl UserActivationState {
    pub fn iter() -> Iter<'static, UserActivationState> {
        static USER_ACTIVATION_STATES: [UserActivationState; 2] =
            [UserActivationState::Pending, UserActivationState::Active];
        USER_ACTIVATION_STATES.iter()
    }

    pub fn as_vec() -> Vec<UserActivationState> {
        UserActivationState::iter().cloned().collect()
    }
}

#[cfg(test)]
mod user_activation_state_test {
    use super::*;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(
            UserActivationState::Pending,
            UserActivationState::from("pending".to_string())
        );
        assert_eq!(
            UserActivationState::Active,
            UserActivationState::from("active".to_string())
        );

        // default
        assert_eq!(
            UserActivationState::Pending,
            UserActivationState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("pending", format!("{}", UserActivationState::Pending));
        assert_eq!("active", format!("{}", UserActivationState::Active));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![UserActivationState::Pending, UserActivationState::Active],
            UserActivationState::as_vec()
        )
    }
}
