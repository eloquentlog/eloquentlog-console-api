//! # Message model for logging
//!
//! See diesel_tests' custom_types.rs
use std::fmt;

use chrono::{NaiveDateTime, Utc};
use diesel::{self, Insertable, prelude::*};
use diesel::debug_query;
use diesel::pg::{Pg, PgConnection};
use serde::Serialize;

use logger::Logger;
pub use model::log_level::*;
pub use model::log_format::*;
pub use schema::messages;

use request::message::Message as RequestData;

/// NewMessage
#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub code: Option<String>,
    pub lang: String,
    pub level: LogLevel,
    pub format: LogFormat,
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
            level: LogLevel::Information,
            format: LogFormat::TOML,
            title: None, // validation error
            content: None,
        }
    }
}

impl From<RequestData> for NewMessage {
    fn from(data: RequestData) -> Self {
        Self {
            code: data.code,
            lang: data.lang.unwrap_or_else(|| "en".to_string()),
            level: LogLevel::from(
                data.level.unwrap_or_else(|| "information".to_string()),
            ),
            format: LogFormat::from(
                data.format.unwrap_or_else(|| "toml".to_string()),
            ),
            title: data.title,
            content: data.content,
        }
    }
}

/// Message
#[derive(
    AsChangeset,
    AsExpression,
    Debug,
    Identifiable,
    Insertable,
    Queryable,
    Serialize,
)]
#[table_name = "messages"]
pub struct Message {
    pub id: i64,
    pub code: Option<String>,
    pub lang: String,
    pub level: LogLevel,
    pub format: LogFormat,
    pub title: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Message {title}>", title = self.title)
    }
}

impl Message {
    pub fn first(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = messages::table.find(id);
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<Message>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(m) => Some(m),
        }
    }

    /// Save new message.
    ///
    /// `created_at` and `updated_at` will be filled on PostgreSQL side
    /// using timezone('utc'::text, now()).
    pub fn insert(
        message: &NewMessage,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<i64>
    {
        let q = diesel::insert_into(messages::table)
            .values(message)
            .returning(messages::id);
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<i64>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(id) => Some(id),
        }
    }

    pub fn recent(
        count: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Vec<Message>
    {
        let q = messages::table
            .limit(count)
            .order(messages::created_at.desc());
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<Message>(conn) {
            Err(e) => {
                println!("err: {}", e);
                vec![]
            },
            Ok(r) => r,
        }
    }

    /// Update a message.
    pub fn update(
        message: &mut Message,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<i64>
    {
        message.updated_at = Utc::now().naive_utc();
        let q = diesel::update(messages::table)
            .set(&*message)
            .filter(messages::id.eq(message.id))
            .returning(messages::id);
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

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
mod data {
    use super::*;

    use std::collections::HashMap;

    use chrono::{Utc, TimeZone};

    use hashmap;

    lazy_static! {
        pub static ref MESSAGES: HashMap<&'static str, Message> = hashmap! {
            "blank message" => Message {
                id: 1,
                code: None,
                lang: "en".to_string(),
                level: LogLevel::Information,
                format: LogFormat::TOML,
                title: "title".to_string(),
                content: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            }
        };
    }
}

#[cfg(test)]
mod test {
    use model::test::run;
    use super::*;

    use model::message::data::MESSAGES;

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let m = NewMessage {
                code: None,
                lang: "en".to_string(),
                level: LogLevel::Information,
                format: LogFormat::TOML,
                title: Some("title".to_string()),
                content: None,
            };
            let result = Message::insert(&m, conn, logger);
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
        run(|conn, _, logger| {
            let m = MESSAGES.get("blank message");
            let message = diesel::insert_into(messages::table)
                .values(m)
                .get_result::<Message>(conn)
                .unwrap_or_else(|e| panic!("Error inserting: {}", e));

            assert_eq!(message.title, "title");

            let mut m = Message {
                title: "updated".to_string(),
                content: Some("content".to_string()),

                ..message
            };
            let result = Message::update(&mut m, conn, logger);
            assert!(result.is_some());

            let title = messages::table
                .select(messages::title)
                .filter(messages::id.eq(m.id))
                .get_result::<String>(conn)
                .expect("Failed to load");
            assert_eq!(title, "updated");
        })
    }
}
