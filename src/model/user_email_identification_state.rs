//! # A type UserEmailIdentificationState for UserEmail in user_email.rs
//!
//! EUserEmailIdentificationState represents SQL type value
//! `e_user_email_identification_state` and UserEmailIdentificationState is an
//! Enum holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(QueryId, SqlType)]
#[postgres(type_name = "e_user_email_identification_state")]
pub struct EUserEmailIdentificationState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserEmailIdentificationState"]
pub enum UserEmailIdentificationState {
    Pending, // default
    Done,
}

impl fmt::Display for UserEmailIdentificationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl ToSql<EUserEmailIdentificationState, Pg> for UserEmailIdentificationState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::Pending => out.write_all(b"pending")?,
            Self::Done => out.write_all(b"done")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserEmailIdentificationState, Pg>
    for UserEmailIdentificationState
{
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(Self::Pending),
            b"done" => Ok(Self::Done),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserEmailIdentificationState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => Self::Pending,
            "done" => Self::Done,
            _ => Self::Pending,
        }
    }
}

impl UserEmailIdentificationState {
    pub fn iter() -> Iter<'static, Self> {
        static USER_STATES: [UserEmailIdentificationState; 2] = [
            UserEmailIdentificationState::Pending,
            UserEmailIdentificationState::Done,
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
            UserEmailIdentificationState::Pending,
            UserEmailIdentificationState::from("pending".to_string())
        );
        assert_eq!(
            UserEmailIdentificationState::Done,
            UserEmailIdentificationState::from("done".to_string())
        );

        // default
        assert_eq!(
            UserEmailIdentificationState::Pending,
            UserEmailIdentificationState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            "pending",
            format!("{}", UserEmailIdentificationState::Pending)
        );
        assert_eq!("done", format!("{}", UserEmailIdentificationState::Done));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                UserEmailIdentificationState::Pending,
                UserEmailIdentificationState::Done,
            ],
            UserEmailIdentificationState::as_vec()
        )
    }
}
