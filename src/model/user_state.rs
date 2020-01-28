//! # A type UserState for User in user.rs
//!
//! EUserState represents SQL type value `e_user_state`
//! and UserState is an Enum holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(QueryId, SqlType)]
#[postgres(type_name = "e_user_state")]
pub struct EUserState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserState"]
pub enum UserState {
    Pending, // default
    Active,
}

impl fmt::Display for UserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::Active => write!(f, "active"),
        }
    }
}

impl ToSql<EUserState, Pg> for UserState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::Pending => out.write_all(b"pending")?,
            Self::Active => out.write_all(b"active")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserState, Pg> for UserState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(Self::Pending),
            b"active" => Ok(Self::Active),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => Self::Pending,
            "active" => Self::Active,
            _ => Self::Pending,
        }
    }
}

impl UserState {
    pub fn iter() -> Iter<'static, Self> {
        static USER_STATES: [UserState; 2] =
            [UserState::Pending, UserState::Active];
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
        assert_eq!(UserState::Pending, UserState::from("pending".to_string()));
        assert_eq!(UserState::Active, UserState::from("active".to_string()));

        // default
        assert_eq!(UserState::Pending, UserState::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("pending", format!("{}", UserState::Pending));
        assert_eq!("active", format!("{}", UserState::Active));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![UserState::Pending, UserState::Active],
            UserState::as_vec()
        )
    }
}
