//! # A type UserEmailRole for UserEmail in user_email.rs
//!
//! EUserEmailRole represents SQL type value
//! `e_user_email_role` and UserEmailRole is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType, QueryId)]
#[postgres(type_name = "e_user_email_role")]
pub struct EUserEmailRole;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EUserEmailRole"]
pub enum UserEmailRole {
    General, // default
    Primary,
}

impl fmt::Display for UserEmailRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::General => write!(f, "general"),
            Self::Primary => write!(f, "primary"),
        }
    }
}

impl ToSql<EUserEmailRole, Pg> for UserEmailRole {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::General => out.write_all(b"general")?,
            Self::Primary => out.write_all(b"primary")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EUserEmailRole, Pg> for UserEmailRole {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"general" => Ok(Self::General),
            b"primary" => Ok(Self::Primary),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for UserEmailRole {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "general" => Self::General,
            "primary" => Self::Primary,
            _ => Self::General,
        }
    }
}

impl UserEmailRole {
    pub fn iter() -> Iter<'static, UserEmailRole> {
        static USER_EMAIL_ROLES: [UserEmailRole; 2] =
            [UserEmailRole::General, UserEmailRole::Primary];
        USER_EMAIL_ROLES.iter()
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
            UserEmailRole::General,
            UserEmailRole::from("general".to_string())
        );
        assert_eq!(
            UserEmailRole::Primary,
            UserEmailRole::from("primary".to_string())
        );

        // default
        assert_eq!(
            UserEmailRole::General,
            UserEmailRole::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("general", format!("{}", UserEmailRole::General));
        assert_eq!("primary", format!("{}", UserEmailRole::Primary));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![UserEmailRole::General, UserEmailRole::Primary],
            UserEmailRole::as_vec()
        )
    }
}
