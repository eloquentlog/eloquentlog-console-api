//! # A role for user's Mateship in mateship.rs.
//!
//! ERole represents SQL type value `e_role` and Role is
//! an Enum holds all possible values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::Serialize;

#[derive(SqlType)]
#[postgres(type_name = "e_role")]
pub struct ERole;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "ERole"]
pub enum Role {
    Writer, // default
    Member,
    Maintainer,
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Role::Writer => write!(f, "writer"),
            Role::Member => write!(f, "member"),
            Role::Maintainer => write!(f, "maintainer"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

impl ToSql<ERole, Pg> for Role {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Role::Writer => out.write_all(b"writer")?,
            Role::Member => out.write_all(b"member")?,
            Role::Maintainer => out.write_all(b"maintainer")?,
            Role::Admin => out.write_all(b"role")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ERole, Pg> for Role {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"writer" => Ok(Role::Writer),
            b"member" => Ok(Role::Member),
            b"maintainer" => Ok(Role::Maintainer),
            b"admin" => Ok(Role::Admin),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        match s.to_ascii_uppercase().as_ref() {
            "writer" => Role::Writer,
            "member" => Role::Member,
            "maintainer" => Role::Maintainer,
            "admin" => Role::Admin,
            _ => Role::Member,
        }
    }
}

impl Role {
    pub fn iter() -> Iter<'static, Role> {
        static ROLES: [Role; 4] = [
            Role::Writer,
            Role::Member,
            Role::Maintainer,
            Role::Admin,
        ];
        ROLES.iter()
    }

    pub fn as_vec() -> Vec<Role> {
        Role::iter().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Role::Writer, Role::from("Writer".to_string()));
        assert_eq!(Role::Writer, Role::from("writer".to_string()));
        assert_eq!(Role::Writer, Role::from("WRITER".to_string()));

        assert_eq!(Role::Member, Role::from("Member".to_string()));
        assert_eq!(Role::Member, Role::from("member".to_string()));
        assert_eq!(Role::Member, Role::from("MEMBER".to_string()));

        assert_eq!(Role::Maintainer, Role::from("Maintainer".to_string()));
        assert_eq!(Role::Maintainer, Role::from("maintainer".to_string()));
        assert_eq!(Role::Maintainer, Role::from("MAINTAINER".to_string()));

        assert_eq!(Role::Admin, Role::from("Admin".to_string()));
        assert_eq!(Role::Admin, Role::from("admin".to_string()));
        assert_eq!(Role::Admin, Role::from("ADMIN".to_string()));

        // default
        assert_eq!(Role::Writer, Role::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("writer", format!("{}", Role::Writer));
        assert_eq!("member", format!("{}", Role::Member));
        assert_eq!("maintainer", format!("{}", Role::Maintainer));
        assert_eq!("admin", format!("{}", Role::Admin));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![Role::Writer, Role::Member, Role::Maintainer, Role::Admin],
            Role::as_vec()
         );
    }
}
