//! # A type Level for Message in message.rs
//!
//! LogLevel represents SQL type value and Level is an enum holds all the
//! values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "log_level")]
pub struct LogLevel;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "LogLevel"]
pub enum Level {
    Debug,
    Information, // default
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Level::Debug => write!(f, "debug"),
            Level::Information => write!(f, "information"),
            Level::Warning => write!(f, "warning"),
            Level::Error => write!(f, "error"),
            Level::Critical => write!(f, "critical"),
        }
    }
}

impl ToSql<LogLevel, Pg> for Level {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            Level::Debug => out.write_all(b"debug")?,
            Level::Information => out.write_all(b"information")?,
            Level::Warning => out.write_all(b"warning")?,
            Level::Error => out.write_all(b"error")?,
            Level::Critical => out.write_all(b"critical")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LogLevel, Pg> for Level {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"debug" => Ok(Level::Debug),
            b"information" => Ok(Level::Information),
            b"warning" => Ok(Level::Warning),
            b"error" => Ok(Level::Error),
            b"critical" => Ok(Level::Critical),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for Level {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "debug" => Level::Debug,
            "information" => Level::Information,
            "info" => Level::Information,
            "warning" => Level::Warning,
            "warn" => Level::Warning,
            "error" => Level::Error,
            "erro" => Level::Error,
            "err" => Level::Error,
            "critical" => Level::Critical,
            _ => Level::Information,
        }
    }
}

impl Level {
    pub fn iter() -> Iter<'static, Level> {
        static LEVELS: [Level; 5] = [
            Level::Debug,
            Level::Information,
            Level::Warning,
            Level::Error,
            Level::Critical,
        ];
        LEVELS.iter()
    }

    pub fn as_vec() -> Vec<Level> {
        Level::iter().cloned().collect()
    }
}

#[cfg(test)]
mod level_test {
    use super::*;

    #[allow(clippy::cyclomatic_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(Level::Debug, Level::from("debug".to_string()));
        assert_eq!(Level::Debug, Level::from("Debug".to_string()));
        assert_eq!(Level::Debug, Level::from("DEBUG".to_string()));
        assert_eq!(Level::Information, Level::from("information".to_string()));
        assert_eq!(Level::Information, Level::from("Information".to_string()));
        assert_eq!(Level::Information, Level::from("INFORMATION".to_string()));
        assert_eq!(Level::Information, Level::from("info".to_string()));
        assert_eq!(Level::Information, Level::from("Info".to_string()));
        assert_eq!(Level::Information, Level::from("INFO".to_string()));
        assert_eq!(Level::Warning, Level::from("warning".to_string()));
        assert_eq!(Level::Warning, Level::from("Warning".to_string()));
        assert_eq!(Level::Warning, Level::from("WARNING".to_string()));
        assert_eq!(Level::Warning, Level::from("warn".to_string()));
        assert_eq!(Level::Warning, Level::from("Warn".to_string()));
        assert_eq!(Level::Warning, Level::from("WARN".to_string()));
        assert_eq!(Level::Error, Level::from("error".to_string()));
        assert_eq!(Level::Error, Level::from("Error".to_string()));
        assert_eq!(Level::Error, Level::from("ERROR".to_string()));
        assert_eq!(Level::Error, Level::from("erro".to_string()));
        assert_eq!(Level::Error, Level::from("Erro".to_string()));
        assert_eq!(Level::Error, Level::from("ERRO".to_string()));
        assert_eq!(Level::Error, Level::from("err".to_string()));
        assert_eq!(Level::Error, Level::from("Err".to_string()));
        assert_eq!(Level::Error, Level::from("ERR".to_string()));
        assert_eq!(Level::Critical, Level::from("critical".to_string()));
        assert_eq!(Level::Critical, Level::from("Critical".to_string()));

        // default
        assert_eq!(Level::Information, Level::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("debug", format!("{}", Level::Debug));
        assert_eq!("information", format!("{}", Level::Information));
        assert_eq!("warning", format!("{}", Level::Warning));
        assert_eq!("error", format!("{}", Level::Error));
        assert_eq!("critical", format!("{}", Level::Critical));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                Level::Debug,
                Level::Information,
                Level::Warning,
                Level::Error,
                Level::Critical,
            ],
            Level::as_vec()
        )
    }
}
