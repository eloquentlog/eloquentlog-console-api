//! # Message model for logging
//!
//! See diesel_tests' custom_types.rs
use std::fmt;

// use diesel::debug_query;
use diesel::{self, Insertable, prelude::*};
use diesel::pg::PgConnection;

use model::level::LogLevel;
use model::format::LogFormat;
pub use model::level::Level;
pub use model::format::Format;

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

/// NewMessage
#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub code: Option<String>,
    pub lang: String,
    pub level: Level,
    pub format: Format,
    pub title: Option<String>,
    pub content: Option<String>,
}

impl fmt::Display for NewMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.title {
            Some(title) => write!(f, "<NewMessage {title}>", title = title),
            None => write!(f, "<NewMessage>"),
        }
    }
}

impl Default for NewMessage {
    fn default() -> Self {
        Self {
            code: None,
            lang: "en".to_string(),
            level: Level::Information,
            format: Format::TOML,
            title: None, // validation error
            content: None,
        }
    }
}

/// Message
#[derive(AsChangeset, AsExpression, Debug, Identifiable, Queryable)]
#[table_name = "messages"]
pub struct Message {
    pub id: i64,
    pub code: Option<String>,
    pub lang: String,
    pub level: Level,
    pub format: Format,
    pub title: String,
    pub content: Option<String>,
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
    pub fn insert(message: &NewMessage, conn: &PgConnection) -> Option<i64> {
        let q = diesel::insert_into(messages::table)
            .values(message)
            .returning(messages::id);

        // TODO
        // let sql = debug_query::<Pg, _>(&q).to_string();
        // println!("sql: {}", sql);

        match q.get_result::<i64>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(id) => Some(id),
        }
    }

    /// Update a message.
    pub fn update(message: &Message, conn: &PgConnection) -> Option<i64> {
        let q = diesel::update(messages::table)
            .set(message)
            .filter(messages::id.eq(message.id))
            .returning(messages::id);

        // TODO
        // let sql = debug_query::<Pg, _>(&q).to_string();
        // println!("sql: {}", sql);

        match q.get_result::<i64>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(id) => Some(id),
        }
    }
}

#[cfg(test)]
mod message_test {
    use std::panic::{self, AssertUnwindSafe};

    use diesel::PgConnection;

    use config::Config;
    use super::*;

    #[test]
    fn test_insert() {
        run(|conn| {
            let m = NewMessage {
                code: None,
                lang: "en".to_string(),
                level: Level::Information,
                format: Format::TOML,
                title: Some("title".to_string()),
                content: None,
            };
            let result = Message::insert(&m, conn);
            assert!(result.is_some());

            let rows_count: i64 = messages::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);
        })
    }

    #[test]
    fn test_update() {
        run(|conn| {
            let m = NewMessage {
                code: Some("200".to_string()),
                lang: "en".to_string(),
                level: Level::Information,
                format: Format::TOML,
                title: Some("title".to_string()),
                content: None,
            };

            let inserted_id = diesel::insert_into(messages::table)
                .values(&m)
                .returning(messages::id)
                .get_result::<i64>(conn)
                .unwrap_or_else(|_| panic!("Error inserting: {}", m));
            assert_eq!(1, inserted_id);

            let current_title = messages::table
                .filter(messages::id.eq(inserted_id))
                .select(messages::title)
                .first::<String>(conn)
                .expect("Failed to select a row");
            assert_eq!("title", current_title);

            let m = Message {
                id: inserted_id,
                code: Some("200".to_string()),
                lang: "en".to_string(),
                level: Level::Information,
                format: Format::TOML,
                title: "updated".to_string(),
                content: Some("content".to_string()),
            };
            let result = Message::update(&m, conn);
            assert!(result.is_some());

            let value = messages::table
                .select(messages::title)
                .filter(messages::id.eq(m.id))
                .get_result::<String>(conn)
                .expect("Failed to load");
            assert_eq!("updated", &value);
        })
    }

    /// A test runner
    fn run<T>(test: T)
    where T: FnOnce(&PgConnection) -> () + panic::UnwindSafe {
        let conn = establish_connection();

        let _: std::result::Result<(), diesel::result::Error> = conn
            .build_transaction()
            .serializable()
            .read_write()
            .run(|| {
                setup(&conn);

                let result =
                    panic::catch_unwind(AssertUnwindSafe(|| test(&conn)));

                teardown(&conn);

                assert!(result.is_ok());
                Ok(())
            });
    }

    fn setup(conn: &PgConnection) {
        truncate_messages(conn);
    }

    fn teardown(conn: &PgConnection) {
        truncate_messages(conn);
    }

    fn truncate_messages(conn: &PgConnection) {
        let _ = diesel::sql_query("TRUNCATE TABLE messages;")
            .execute(conn)
            .expect("Failed to truncate");

        let _ =
            diesel::sql_query("ALTER SEQUENCE messages_id_seq RESTART WITH 1;")
                .execute(conn)
                .expect("Failed to reset sequence");
    }

    fn establish_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let c = Config::from("testing").unwrap();
        PgConnection::establish(&c.database_url).unwrap_or_else(|_| {
            panic!("Error connecting to : {}", c.database_url)
        })
    }
}
