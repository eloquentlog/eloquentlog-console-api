//! # Message model for logging
//!
//! This object describes a log message on a stream.
//!
//! ## Note
//!
//! See diesel_tests' custom_types.rs.
use std::fmt;

use chrono::{NaiveDateTime, Utc};
use diesel::{self, Insertable, prelude::*};
use diesel::debug_query;
use diesel::dsl;
use diesel::pg::{Pg, PgConnection};
use serde::Serialize;

use crate::logger::Logger;
use crate::request::message::Message as RequestData;

pub use crate::model::agent_type::*;
pub use crate::model::log_level::*;
pub use crate::model::log_format::*;
pub use crate::model::stream::{Stream, streams};
use crate::model::user::User;
pub use crate::schema::messages;

/// NewMessage
#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub stream_id: i64,
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
    // includes validation errors
    fn default() -> Self {
        Self {
            agent_id: -1,
            agent_type: AgentType::Person,
            stream_id: -1,
            code: None,
            lang: "en".to_string(),
            level: LogLevel::Information,
            format: LogFormat::TOML,
            title: None,
            content: None,
        }
    }
}

impl From<RequestData> for NewMessage {
    fn from(data: RequestData) -> Self {
        // TODO: get stream_id from data
        Self {
            agent_id: data.agent_id,
            agent_type: AgentType::from(
                data.agent_type.unwrap_or_else(|| "".to_string()),
            ),
            stream_id: data.stream_id,
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

type AllColumns = (
    messages::id,
    messages::agent_id,
    messages::agent_type,
    messages::code,
    messages::lang,
    messages::level,
    messages::format,
    messages::title,
    messages::content,
    messages::created_at,
    messages::updated_at,
);

const ALL_COLUMNS: AllColumns = (
    messages::id,
    messages::agent_id,
    messages::agent_type,
    messages::code,
    messages::lang,
    messages::level,
    messages::format,
    messages::title,
    messages::content,
    messages::created_at,
    messages::updated_at,
);

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
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub stream_id: i64,
    pub code: Option<String>,
    pub lang: String,
    pub level: LogLevel,
    pub format: LogFormat,
    pub title: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        let agent_type = format!("{}", self.agent_type);
        let level = format!("{}", self.level);
        let format = format!("{}", self.format);

        Self {
            agent_id: self.agent_id,
            agent_type: AgentType::from(agent_type),
            stream_id: self.stream_id,
            code: self.code.clone(),
            lang: self.lang.clone(),
            level: LogLevel::from(level),
            format: LogFormat::from(format),
            title: self.title.clone(),
            content: self.content.clone(),

            ..*self
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Message {title}>", title = self.title)
    }
}

type All = dsl::Select<messages::table, AllColumns>;
type WithType = dsl::Eq<messages::agent_type, AgentType>;
type WithUser = dsl::And<
    dsl::Eq<messages::agent_id, i64>,
    dsl::Eq<messages::agent_type, AgentType>,
>;
type Visible = dsl::IsNotNull<messages::content>;
type ByUser = dsl::Filter<All, WithUser>;
type VisibleTo = dsl::Filter<All, dsl::And<WithUser, Visible>>;

impl Message {
    pub fn all() -> All {
        messages::table.select(ALL_COLUMNS)
    }

    pub fn by_user(user: &User) -> ByUser {
        Self::all().filter(Self::with_user(user))
    }

    pub fn fetch_by_stream_slug(
        stream_slug: String,
        offset: i64,
        limit: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Vec<Self>>
    {
        if stream_slug.is_empty() {
            return None;
        }

        // TODO: Fix clause id = slug
        let stream_id = 1;
        let q = messages::table
            .inner_join(streams::table)
            .filter(streams::id.eq(stream_id))
            .order(messages::created_at.desc())
            .offset(offset)
            .limit(limit);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<(Self, Stream)>(conn) {
            Ok(r) => Some(r.into_iter().map(|(m, _)| m).collect::<Vec<Self>>()),
            Err(e) => {
                println!("err: {}", e);
                None
            },
        }
    }

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

    // FIXME: scope
    pub fn visible() -> Visible {
        messages::content.is_not_null()
    }

    // FIXME: scope
    pub fn visible_to(user: &User) -> VisibleTo {
        Self::all().filter(Self::with_user(user).and(Self::visible()))
    }

    pub fn with_type(agent_type: AgentType) -> WithType {
        messages::agent_type.eq(agent_type)
    }

    pub fn with_user(user: &User) -> WithUser {
        messages::agent_id
            .eq(user.id)
            .and(Self::with_type(AgentType::Person))
    }
}

#[cfg(test)]
mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;
    use crate::model::stream::data::STREAMS;

    type MessageFixture = FnvHashMap<&'static str, Message>;

    lazy_static! {
        pub static ref MESSAGES: MessageFixture = fnvhashmap! {
            "blank message" => Message {
                id: 1,
                agent_id: 0,
                agent_type: AgentType::Person,
                stream_id: STREAMS.clone().get("weenie's stream").unwrap().id,
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
    use super::*;

    use crate::model::message::data::MESSAGES;
    use crate::model::namespace::{Namespace, namespaces};
    use crate::model::namespace::data::NAMESPACES;
    use crate::model::stream::{Stream, streams};
    use crate::model::stream::data::STREAMS;
    use crate::model::test::run;

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let ns = NAMESPACES.get("ball").unwrap();
            let namespace = diesel::insert_into(namespaces::table)
                .values(ns)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let mut s = STREAMS.get("weenie's stream").unwrap().clone();
            s.namespace_id = namespace.id;
            let stream = diesel::insert_into(streams::table)
                .values(&s)
                .get_result::<Stream>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let m = NewMessage {
                agent_id: 1,
                agent_type: AgentType::Person,
                stream_id: stream.id,
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
            let ns = NAMESPACES.get("ball").unwrap();
            let namespace = diesel::insert_into(namespaces::table)
                .values(ns)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let mut s = STREAMS.get("weenie's stream").unwrap().clone();
            s.namespace_id = namespace.id;
            let stream = diesel::insert_into(streams::table)
                .values(s)
                .get_result::<Stream>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let mut m = MESSAGES.get("blank message").unwrap().clone();
            m.stream_id = stream.id;
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
