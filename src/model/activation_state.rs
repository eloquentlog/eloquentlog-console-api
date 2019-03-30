//! # A type ActivationState for User in user.rs
//!
//! AccountActivationState represents SQL type value and ActivationState is an
//! enum holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "account_activation_state")]
pub struct AccountActivationState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "AccountActivationState"]
pub enum ActivationState {
    Pending, // default
    Active,
}

impl fmt::Display for ActivationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ActivationState::Pending => write!(f, "pending"),
            ActivationState::Active => write!(f, "active"),
        }
    }
}

impl ToSql<AccountActivationState, Pg> for ActivationState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            ActivationState::Pending => out.write_all(b"pending")?,
            ActivationState::Active => out.write_all(b"active")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<AccountActivationState, Pg> for ActivationState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"pending" => Ok(ActivationState::Pending),
            b"active" => Ok(ActivationState::Active),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for ActivationState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "pending" => ActivationState::Pending,
            "active" => ActivationState::Active,
            _ => ActivationState::Pending,
        }
    }
}

impl ActivationState {
    pub fn iter() -> Iter<'static, ActivationState> {
        static ACTIVATION_STATES: [ActivationState; 2] =
            [ActivationState::Pending, ActivationState::Active];
        ACTIVATION_STATES.iter()
    }

    pub fn as_vec() -> Vec<ActivationState> {
        ActivationState::iter().cloned().collect()
    }
}

#[cfg(test)]
mod activation_state_test {
    use super::*;

    #[allow(clippy::cyclomatic_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(
            ActivationState::Pending,
            ActivationState::from("pending".to_string())
        );
        assert_eq!(
            ActivationState::Active,
            ActivationState::from("active".to_string())
        );

        // default
        assert_eq!(
            ActivationState::Pending,
            ActivationState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("pending", format!("{}", ActivationState::Pending));
        assert_eq!("active", format!("{}", ActivationState::Active));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![ActivationState::Pending, ActivationState::Active],
            ActivationState::as_vec()
        )
    }
}
