//! # A type LogFormat for Message in message.rs.
//!
//! ELogFormat represents SQL type value `e_log_format` and LogFormat is
//! an Enum holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::Serialize;

#[derive(SqlType)]
#[postgres(type_name = "e_log_format")]
pub struct ELogFormat;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "ELogFormat"]
pub enum LogFormat {
    TOML, // default
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogFormat::TOML => write!(f, "toml"),
        }
    }
}

impl ToSql<ELogFormat, Pg> for LogFormat {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            LogFormat::TOML => out.write_all(b"toml")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ELogFormat, Pg> for LogFormat {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"toml" => Ok(LogFormat::TOML),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for LogFormat {
    fn from(s: String) -> Self {
        match s.to_ascii_uppercase().as_ref() {
            "toml" => LogFormat::TOML,
            _ => LogFormat::TOML,
        }
    }
}

impl LogFormat {
    pub fn iter() -> Iter<'static, LogFormat> {
        static LOG_FORMATS: [LogFormat; 1] = [LogFormat::TOML];
        LOG_FORMATS.iter()
    }

    pub fn as_vec() -> Vec<LogFormat> {
        LogFormat::iter().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(LogFormat::TOML, LogFormat::from("toml".to_string()));
        assert_eq!(LogFormat::TOML, LogFormat::from("Toml".to_string()));
        assert_eq!(LogFormat::TOML, LogFormat::from("TOML".to_string()));

        // default
        assert_eq!(LogFormat::TOML, LogFormat::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("toml", format!("{}", LogFormat::TOML));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(vec![LogFormat::TOML], LogFormat::as_vec());
    }
}
