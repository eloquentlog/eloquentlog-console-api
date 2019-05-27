//! # A type UserResetPasswordState for User in user.rs
//!
//! EUserResetPasswordState represents SQL type value
//! `e_user_reset_password_state` and UserResetPasswordState is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "e_user_reset_password_state")]
pub struct EUserResetPasswordState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserResetPasswordState"]
pub enum UserResetPasswordState {
    NeverYet, // default
    Pending,
    InProgress,
    Done,
}

impl fmt::Display for UserResetPasswordState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NeverYet => write!(f, "never-yet"),
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in-progress"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl ToSql<EUserResetPasswordState, Pg> for UserResetPasswordState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::NeverYet => out.write_all(b"never-yet")?,
            Self::Pending => out.write_all(b"pending")?,
            Self::InProgress => out.write_all(b"in-progress")?,
            Self::Done => out.write_all(b"done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserResetPasswordState, Pg> for UserResetPasswordState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"never-yet" => Ok(Self::NeverYet),
            b"pending" => Ok(Self::Pending),
            b"in-progress" => Ok(Self::InProgress),
            b"done" => Ok(Self::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserResetPasswordState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "never-yet" => Self::NeverYet,
            "pending" => Self::Pending,
            "in-progress" => Self::InProgress,
            "done" => Self::Done,
            _ => Self::NeverYet,
        }
    }
}

impl UserResetPasswordState {
    pub fn iter() -> Iter<'static, Self> {
        static USER_RESET_PASSWORD_STATES: [UserResetPasswordState; 4] = [
            UserResetPasswordState::NeverYet,
            UserResetPasswordState::Pending,
            UserResetPasswordState::InProgress,
            UserResetPasswordState::Done,
        ];
        USER_RESET_PASSWORD_STATES.iter()
    }

    pub fn as_vec() -> Vec<Self> {
        Self::iter().cloned().collect()
    }
}

#[cfg(test)]
mod user_reset_password_state_test {
    use super::*;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(
            UserResetPasswordState::NeverYet,
            UserResetPasswordState::from("never-yet".to_string())
        );
        assert_eq!(
            UserResetPasswordState::Pending,
            UserResetPasswordState::from("pending".to_string())
        );
        assert_eq!(
            UserResetPasswordState::InProgress,
            UserResetPasswordState::from("in-progress".to_string())
        );
        assert_eq!(
            UserResetPasswordState::Done,
            UserResetPasswordState::from("done".to_string())
        );

        // default
        assert_eq!(
            UserResetPasswordState::NeverYet,
            UserResetPasswordState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            "never-yet",
            format!("{}", UserResetPasswordState::NeverYet),
        );
        assert_eq!("pending", format!("{}", UserResetPasswordState::Pending));
        assert_eq!(
            "in-progress",
            format!("{}", UserResetPasswordState::InProgress),
        );
        assert_eq!("done", format!("{}", UserResetPasswordState::Done));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                UserResetPasswordState::NeverYet,
                UserResetPasswordState::Pending,
                UserResetPasswordState::InProgress,
                UserResetPasswordState::Done,
            ],
            UserResetPasswordState::as_vec()
        )
    }
}
