//! # Message model for logging
//!
//! See diesel_tests' custom_types.rs
use std::fmt;
use std::io::Write;

use diesel::{self, Insertable, prelude::*};
use diesel::pg::{Pg, PgConnection};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::deserialize::{self, FromSql};

mod schema {
    table! {
        use diesel::sql_types::*;
        use model::message::LogFormat;
        use model::message::LogLevel;

        messages (id) {
            id -> Nullable<Integer>,
            code -> Nullable<Varchar>,
            lang -> Varchar,
            level -> LogLevel,
            format -> LogFormat,
            title -> Text,
            content -> Text,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }
}

use self::schema::messages;

// Log Format
#[derive(SqlType)]
#[postgres(type_name = "log_format")]
pub struct LogFormat;

#[derive(AsExpression, Debug, FromSqlRow, PartialEq)]
#[sql_type = "LogFormat"]
pub enum Format {
    TOML, // default
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

// Log Level
#[derive(SqlType)]
#[postgres(type_name = "log_level")]
pub struct LogLevel;

#[derive(AsExpression, Debug, FromSqlRow, PartialEq)]
#[sql_type = "LogLevel"]
pub enum Level {
    Debug,
    Information, // default
    Warning,
    Error,
    Critical,
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

#[derive(Debug, Identifiable)]
#[table_name = "messages"]
pub struct Message {
    pub id: i64,
    pub code: String,
    pub lang: String,
    pub level: Level,
    pub format: Format,
    pub title: String,
    pub content: String,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub code: String,
    pub lang: String,
    pub level: Level,
    pub format: Format,
    pub title: String,
    pub content: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Message {title}>", title = self.title)
    }
}

impl Message {
    /// Save new message.
    ///
    /// `created_at` and `updated_at` will be filled on PostgreSQL side
    /// using timezone('utc'::text, now()).
    pub fn insert(message: NewMessage, conn: &PgConnection) -> bool {
        let result = diesel::insert_into(messages::table)
            .values(&message)
            .execute(conn);
        match result {
            Err(e) => {
                println!("err: {}", e);
                false
            },
            Ok(_) => true,
        }
    }
}
