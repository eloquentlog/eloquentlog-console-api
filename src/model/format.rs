//! # A type Format for Message in message.rs.
//!
//! LogFormat represents SQL type value and Format is an enum holds all the
//! values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "log_format")]
pub struct LogFormat;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq)]
#[sql_type = "LogFormat"]
pub enum Format {
    TOML, // default
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Format::TOML => write!(f, "toml"),
        }
    }
}

impl ToSql<LogFormat, Pg> for Format {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Format::TOML => out.write_all(b"toml")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LogFormat, Pg> for Format {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"toml" => Ok(Format::TOML),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for Format {
    fn from(s: String) -> Self {
        match s.to_ascii_uppercase().as_ref() {
            "toml" => Format::TOML,
            _ => Format::TOML,
        }
    }
}

impl Format {
    pub fn iter() -> Iter<'static, Format> {
        static FORMATS: [Format; 1] = [Format::TOML];
        FORMATS.iter()
    }

    pub fn as_vec() -> Vec<Format> {
        Format::iter().cloned().collect()
    }
}

#[cfg(test)]
mod format_test {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Format::TOML, Format::from("toml".to_string()));
        assert_eq!(Format::TOML, Format::from("Toml".to_string()));
        assert_eq!(Format::TOML, Format::from("TOML".to_string()));

        // default
        assert_eq!(Format::TOML, Format::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("toml", format!("{}", Format::TOML));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(vec![Format::TOML], Format::as_vec());
    }
}
