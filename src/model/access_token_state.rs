//! # A type AccessTokenState for AccessToken in access_token.rs
//!
//! EAccessTokenState represents SQL type value
//! `e_access_token_state` and AccessTokenState is an
//! Enum contains all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(QueryId, SqlType)]
#[postgres(type_name = "e_access_token_state")]
pub struct EAccessTokenState;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EAccessTokenState"]
pub enum AccessTokenState {
    Disabled, // default
    Enabled,
}

impl fmt::Display for AccessTokenState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Disabled => write!(f, "disabled"),
            Self::Enabled => write!(f, "enabled"),
        }
    }
}

impl ToSql<EAccessTokenState, Pg> for AccessTokenState {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::Disabled => out.write_all(b"disabled")?,
            Self::Enabled => out.write_all(b"enabled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EAccessTokenState, Pg> for AccessTokenState {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"disabled" => Ok(Self::Disabled),
            b"enabled" => Ok(Self::Enabled),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for AccessTokenState {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "disabled" => Self::Disabled,
            "enabled" => Self::Enabled,
            _ => Self::Disabled,
        }
    }
}

impl AccessTokenState {
    pub fn iter() -> Iter<'static, Self> {
        static ACCESS_TOKEN_STATES: [AccessTokenState; 2] =
            [AccessTokenState::Disabled, AccessTokenState::Enabled];
        ACCESS_TOKEN_STATES.iter()
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
            AccessTokenState::Disabled,
            AccessTokenState::from("disabled".to_string())
        );
        assert_eq!(
            AccessTokenState::Enabled,
            AccessTokenState::from("enabled".to_string())
        );

        // default
        assert_eq!(
            AccessTokenState::Disabled,
            AccessTokenState::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("disabled", format!("{}", AccessTokenState::Disabled));
        assert_eq!("enabled", format!("{}", AccessTokenState::Enabled));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![AccessTokenState::Disabled, AccessTokenState::Enabled],
            AccessTokenState::as_vec()
        )
    }
}
