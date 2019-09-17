//! # A type UserEmailVerificationState for UserEmail in user_email.rs
//!
//! EUserEmailVerificationState represents SQL type value
//! `e_user_email_verification_state` and UserEmailVerificationState is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(QueryId, SqlType)]
#[postgres(type_name = "e_user_email_verification_state")]
pub struct EUserEmailVerificationState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserEmailVerificationState"]
pub enum UserEmailVerificationState {
    Pending, // default
    Done,
}

impl fmt::Display for UserEmailVerificationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl ToSql<EUserEmailVerificationState, Pg> for UserEmailVerificationState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::Pending => out.write_all(b"pending")?,
            Self::Done => out.write_all(b"done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserEmailVerificationState, Pg> for UserEmailVerificationState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(Self::Pending),
            b"done" => Ok(Self::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserEmailVerificationState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => Self::Pending,
            "done" => Self::Done,
            _ => Self::Pending,
        }
    }
}

impl UserEmailVerificationState {
    pub fn iter() -> Iter<'static, Self> {
        static USER_STATES: [UserEmailVerificationState; 2] = [
            UserEmailVerificationState::Pending,
            UserEmailVerificationState::Done,
        ];
        USER_STATES.iter()
    }

    pub fn as_vec() -> Vec<Self> {
        Self::iter().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(
            UserEmailVerificationState::Pending,
            UserEmailVerificationState::from("pending".to_string())
        );
        assert_eq!(
            UserEmailVerificationState::Done,
            UserEmailVerificationState::from("done".to_string())
        );

        // default
        assert_eq!(
            UserEmailVerificationState::Pending,
            UserEmailVerificationState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            "pending",
            format!("{}", UserEmailVerificationState::Pending)
        );
        assert_eq!("done", format!("{}", UserEmailVerificationState::Done));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                UserEmailVerificationState::Pending,
                UserEmailVerificationState::Done,
            ],
            UserEmailVerificationState::as_vec()
        )
    }
}
