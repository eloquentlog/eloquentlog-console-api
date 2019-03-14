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
            id -> BigInt,
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

/// NewMessage
#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub code: String,
    pub lang: String,
    pub level: Level,
    pub format: Format,
    pub title: String,
    pub content: String,
}

impl fmt::Display for NewMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<NewMessage {title}>", title = self.title)
    }
}

/// Message
#[derive(AsChangeset, AsExpression, Debug, Identifiable, Queryable)]
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

    /// Update a message.
    pub fn update(message: &Message, conn: &PgConnection) -> bool {
        let id = diesel::update(messages::table)
            .set(message)
            .filter(messages::id.eq(message.id))
            .returning(messages::id)
            .get_result::<i64>(conn);
        match id {
            Err(e) => {
                println!("err: {}", e);
                false
            },
            Ok(_) => true,
        }
    }
}

#[cfg(test)]
mod message_test {
    use diesel::PgConnection;

    use config::Config;
    use super::*;

    #[test]
    fn test_update() {
        let conn = establish_connection();

        let m = NewMessage {
            code: "".to_string(),
            lang: "en".to_string(),
            level: Level::Information,
            format: Format::TOML,
            title: "title".to_string(),
            content: "".to_string(),
        };

        // TODO
        let _ = diesel::sql_query("TRUNCATE TABLE messages;")
            .execute(&conn)
            .expect("Failed to clean");

        let _ =
            diesel::sql_query("ALTER SEQUENCE messages_id_seq RESTART WITH 1;")
                .execute(&conn)
                .expect("Failed to reset sequence");

        let inserted_id = diesel::insert_into(messages::table)
            .values(&m)
            .returning(messages::id)
            .get_result::<i64>(&conn)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));
        assert_eq!(1, inserted_id);

        let current_title = messages::table
            .filter(messages::id.eq(inserted_id))
            .select(messages::title)
            .first::<String>(&conn)
            .expect("Failed to select a row");
        assert_eq!("title", current_title);

        let m = Message {
            id: inserted_id,
            code: "".to_string(),
            lang: "en".to_string(),
            level: Level::Information,
            format: Format::TOML,
            title: "updated".to_string(),
            content: "".to_string(),
        };
        assert!(Message::update(&m, &conn));

        let result = messages::table
            .select(messages::title)
            .filter(messages::id.eq(m.id))
            .get_result::<String>(&conn)
            .expect("Failed to load");
        assert_eq!("updated", &result);
    }

    fn establish_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let c = Config::from("testing").unwrap();
        PgConnection::establish(&c.database_url).unwrap_or_else(|_| {
            panic!("Error connecting to : {}", c.database_url)
        })
    }
}
