//! # A type LogLevel for Message in message.rs
//!
//! LogLevel represents SQL type value `e_log_level` and Level is an Enum
//! holds all the values.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use serde::Serialize;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};

#[derive(SqlType)]
#[postgres(type_name = "e_log_level")]
pub struct ELogLevel;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "ELogLevel"]
pub enum LogLevel {
    Debug,
    Information, // default
    Warning,
    Error,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Information => write!(f, "information"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Critical => write!(f, "critical"),
        }
    }
}

impl ToSql<ELogLevel, Pg> for LogLevel {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            LogLevel::Debug => out.write_all(b"debug")?,
            LogLevel::Information => out.write_all(b"information")?,
            LogLevel::Warning => out.write_all(b"warning")?,
            LogLevel::Error => out.write_all(b"error")?,
            LogLevel::Critical => out.write_all(b"critical")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ELogLevel, Pg> for LogLevel {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"debug" => Ok(LogLevel::Debug),
            b"information" => Ok(LogLevel::Information),
            b"warning" => Ok(LogLevel::Warning),
            b"error" => Ok(LogLevel::Error),
            b"critical" => Ok(LogLevel::Critical),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for LogLevel {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_ref() {
            "debug" => LogLevel::Debug,
            "information" => LogLevel::Information,
            "info" => LogLevel::Information,
            "warning" => LogLevel::Warning,
            "warn" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "erro" => LogLevel::Error,
            "err" => LogLevel::Error,
            "critical" => LogLevel::Critical,
            _ => LogLevel::Information,
        }
    }
}

impl LogLevel {
    pub fn iter() -> Iter<'static, LogLevel> {
        static LOG_LEVELS: [LogLevel; 5] = [
            LogLevel::Debug,
            LogLevel::Information,
            LogLevel::Warning,
            LogLevel::Error,
            LogLevel::Critical,
        ];
        LOG_LEVELS.iter()
    }

    pub fn as_vec() -> Vec<LogLevel> {
        LogLevel::iter().cloned().collect()
    }
}

#[cfg(test)]
mod log_level_test {
    use super::*;

    #[allow(clippy::cyclomatic_complexity)]
    #[test]
    fn test_from() {
        assert_eq!(LogLevel::Debug, LogLevel::from("debug".to_string()));
        assert_eq!(LogLevel::Debug, LogLevel::from("Debug".to_string()));
        assert_eq!(LogLevel::Debug, LogLevel::from("DEBUG".to_string()));
        assert_eq!(
            LogLevel::Information,
            LogLevel::from("information".to_string())
        );
        assert_eq!(
            LogLevel::Information,
            LogLevel::from("Information".to_string())
        );
        assert_eq!(
            LogLevel::Information,
            LogLevel::from("INFORMATION".to_string())
        );
        assert_eq!(LogLevel::Information, LogLevel::from("info".to_string()));
        assert_eq!(LogLevel::Information, LogLevel::from("Info".to_string()));
        assert_eq!(LogLevel::Information, LogLevel::from("INFO".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("warning".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("Warning".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("WARNING".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("warn".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("Warn".to_string()));
        assert_eq!(LogLevel::Warning, LogLevel::from("WARN".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("error".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("Error".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("ERROR".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("erro".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("Erro".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("ERRO".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("err".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("Err".to_string()));
        assert_eq!(LogLevel::Error, LogLevel::from("ERR".to_string()));
        assert_eq!(LogLevel::Critical, LogLevel::from("critical".to_string()));
        assert_eq!(LogLevel::Critical, LogLevel::from("Critical".to_string()));

        // default
        assert_eq!(
            LogLevel::Information,
            LogLevel::from("unknown".to_string())
        );
    }

    #[test]
    fn test_fmt() {
        assert_eq!("debug", format!("{}", LogLevel::Debug));
        assert_eq!("information", format!("{}", LogLevel::Information));
        assert_eq!("warning", format!("{}", LogLevel::Warning));
        assert_eq!("error", format!("{}", LogLevel::Error));
        assert_eq!("critical", format!("{}", LogLevel::Critical));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![
                LogLevel::Debug,
                LogLevel::Information,
                LogLevel::Warning,
                LogLevel::Error,
                LogLevel::Critical,
            ],
            LogLevel::as_vec()
        )
    }
}
