//! # A type MembershipRole for Membership in membership.rs
//!
//! EMembershipRole represents SQL type value
//! `e_membership_role` and MembershipRole is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(QueryId, SqlType)]
#[postgres(type_name = "e_membership_role")]
pub struct EMembershipRole;

#[derive(
    AsExpression, Clone, Debug, Deserialize, FromSqlRow, PartialEq, Serialize,
)]
#[sql_type = "EMembershipRole"]
pub enum MembershipRole {
    PrimaryOwner,
    Owner,
    Member, // default
}

const MEMBERSHIP_ROLES: [MembershipRole; 3] = [
    MembershipRole::PrimaryOwner,
    MembershipRole::Owner,
    MembershipRole::Member,
];

impl fmt::Display for MembershipRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::PrimaryOwner => write!(f, "primary_owner"),
            Self::Owner => write!(f, "owner"),
            Self::Member => write!(f, "member"),
        }
    }
}

impl ToSql<EMembershipRole, Pg> for MembershipRole {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Self::PrimaryOwner => out.write_all(b"primary_owner")?,
            Self::Owner => out.write_all(b"owner")?,
            Self::Member => out.write_all(b"member")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EMembershipRole, Pg> for MembershipRole {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"primary_owner" => Ok(Self::PrimaryOwner),
            b"owner" => Ok(Self::Owner),
            b"member" => Ok(Self::Member),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for MembershipRole {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "primary_owner" => Self::PrimaryOwner,
            "owner" => Self::Owner,
            "member" => Self::Member,
            _ => Self::Member,
        }
    }
}

impl MembershipRole {
    pub fn iter() -> Iter<'static, MembershipRole> {
        MEMBERSHIP_ROLES.iter()
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
            MembershipRole::PrimaryOwner,
            MembershipRole::from("primary_owner".to_string())
        );
        assert_eq!(
            MembershipRole::Owner,
            MembershipRole::from("owner".to_string())
        );

        // default
        assert_eq!(
            MembershipRole::Member,
            MembershipRole::from("member".to_string())
        );

        assert_eq!(
            MembershipRole::Member,
            MembershipRole::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!(
            "primary_owner",
            format!("{}", MembershipRole::PrimaryOwner)
        );
        assert_eq!("owner", format!("{}", MembershipRole::Owner));
        assert_eq!("member", format!("{}", MembershipRole::Member));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                MembershipRole::PrimaryOwner,
                MembershipRole::Owner,
                MembershipRole::Member,
            ],
            MembershipRole::as_vec()
        )
    }
}
