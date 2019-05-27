//! # A type UserEmailActivationState for UserEmail in user_email.rs
//!
//! EUserEmailActivationState represents SQL type value
//! `e_user_email_activation_state` and UserEmailActivationState is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "e_user_email_activation_state")]
pub struct EUserEmailActivationState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserEmailActivationState"]
pub enum UserEmailActivationState {
    Pending, // default
    Done,
}

impl fmt::Display for UserEmailActivationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl ToSql<EUserEmailActivationState, Pg> for UserEmailActivationState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::Pending => out.write_all(b"pending")?,
            Self::Done => out.write_all(b"done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserEmailActivationState, Pg> for UserEmailActivationState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(Self::Pending),
            b"done" => Ok(Self::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserEmailActivationState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => Self::Pending,
            "done" => Self::Done,
            _ => Self::Pending,
        }
    }
}

impl UserEmailActivationState {
    pub fn iter() -> Iter<'static, Self> {
        static USER_STATES: [UserEmailActivationState; 2] = [
            UserEmailActivationState::Pending,
            UserEmailActivationState::Done,
        ];
        USER_STATES.iter()
    }

    pub fn as_vec() -> Vec<Self> {
        Self::iter().cloned().collect()
    }
}

#[cfg(test)]
mod user_state_test {
    use super::*;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(
            UserEmailActivationState::Pending,
            UserEmailActivationState::from("pending".to_string())
        );
        assert_eq!(
            UserEmailActivationState::Done,
            UserEmailActivationState::from("done".to_string())
        );

        // default
        assert_eq!(
            UserEmailActivationState::Pending,
            UserEmailActivationState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("pending", format!("{}", UserEmailActivationState::Pending));
        assert_eq!("done", format!("{}", UserEmailActivationState::Done));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                UserEmailActivationState::Pending,
                UserEmailActivationState::Done,
            ],
            UserEmailActivationState::as_vec()
        )
    }
}
