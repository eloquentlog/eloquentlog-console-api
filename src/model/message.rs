//! # Message model for logging
//!
//! See diesel_tests' custom_types.rs
use std::fmt;

use chrono::{NaiveDateTime, Utc};
use diesel::{self, Insertable, prelude::*};
use diesel::debug_query;
use diesel::pg::{Pg, PgConnection};
use serde::Serialize;

use crate::logger::Logger;
use crate::request::message::Message as RequestData;

pub use crate::model::agent_type::*;
pub use crate::model::log_level::*;
pub use crate::model::log_format::*;
pub use crate::schema::messages;

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
    pub stream_id: i64,
    pub agent_id: i64,
    pub agent_type: AgentType,
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
            stream_id: 0,
            agent_id: 0, // validation error
            agent_type: AgentType::Client,
        }
    }
}

impl From<RequestData> for NewMessage {
    fn from(data: RequestData) -> Self {
        // TODO: find stream by stream.uuid
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
            stream_id: 0,
            agent_id: 0,
            agent_type: AgentType::Client,
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
    pub stream_id: i64,
    pub agent_id: i64,
    pub agent_type: AgentType,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        let agent_type = format!("{}", self.agent_type);
        let level = format!("{}", self.level);
        let format = format!("{}", self.format);
        Message {
            code: self.code.clone(),
            lang: self.lang.clone(),
            level: LogLevel::from(level),
            format: LogFormat::from(format),
            title: self.title.clone(),
            content: self.content.clone(),
            agent_id: self.agent_id,
            agent_type: AgentType::from(agent_type),

            ..*self
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Message {title}>", title = self.title)
    }
}

impl Message {
    pub fn first_by_stream_id(
        id: i64,
        stream_id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = messages::table
            .filter(messages::stream_id.eq(stream_id))
            .find(id);
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

    pub fn fetch_messages_by_stream_id(
        _key: String,
        stream_id: i64,
        offset: i64,
        limit: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Vec<Self>>
    {
        if stream_id < 1 || limit < 1 {
            return None;
        }

        // FIXME: Add key (namespace) support

        let q = messages::table
            .filter(messages::stream_id.eq(stream_id))
            .order(messages::created_at.desc())
            .offset(offset)
            .limit(limit);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<Message>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(r) => Some(r),
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

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;

    type MessageFixture = FnvHashMap<&'static str, Message>;

    lazy_static! {
        pub static ref MESSAGES: MessageFixture = fnvhashmap! {
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
                stream_id: 0, // dummy
                agent_id: 0,
                agent_type: AgentType::Person,
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::user::{User, users};

    use crate::model::message::data::MESSAGES;
    use crate::model::test::run;
    use crate::model::user::data::USERS;

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let u = USERS.get("weenie").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let m = NewMessage {
                code: None,
                lang: "en".to_string(),
                level: LogLevel::Information,
                format: LogFormat::TOML,
                title: Some("title".to_string()),
                content: None,
                stream_id: 1,
                agent_id: 1,
                agent_type: AgentType::Person,
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
            let u = USERS.get("weenie").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let mut m = MESSAGES.get("blank message").unwrap().clone();
            m.stream_id = 1;
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
