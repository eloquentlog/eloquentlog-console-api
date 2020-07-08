//! # Stream
use std::fmt;
use std::str;

use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable, debug_query, prelude::*};
use diesel::dsl;
use diesel::pg::{Pg, PgConnection};
use uuid::Uuid;

use crate::logger::Logger;

pub use crate::schema::streams;

/// NewStream
#[derive(Debug)]
pub struct NewStream {
    pub namespace_id: i64,
    pub name: String,
    pub description: Option<String>,
}

impl Default for NewStream {
    // includes validation errors
    fn default() -> Self {
        Self {
            namespace_id: -1,
            name: "".to_string(),
            description: None,
        }
    }
}

type AllColumns = (
    streams::id,
    streams::uuid,
    streams::namespace_id,
    streams::name,
    streams::description,
    streams::archived_at,
    streams::created_at,
    streams::updated_at,
);

const ALL_COLUMNS: AllColumns = (
    streams::id,
    streams::uuid,
    streams::namespace_id,
    streams::name,
    streams::description,
    streams::archived_at,
    streams::created_at,
    streams::updated_at,
);

/// Stream
#[derive(
    AsChangeset,
    AsExpression,
    Debug,
    Identifiable,
    Insertable,
    PartialEq,
    Queryable,
)]
#[table_name = "streams"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Stream {
    pub id: i64,
    pub uuid: Uuid,
    pub namespace_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub archived_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Stream {uuid}>", uuid = &self.uuid.to_string())
    }
}

type All = dsl::Select<streams::table, AllColumns>;
type WithUuid = dsl::Eq<streams::uuid, Uuid>;
type Visible = dsl::IsNull<streams::archived_at>;
type ByUuid = dsl::Filter<All, WithUuid>;

impl Stream {
    pub fn all() -> All {
        streams::table.select(ALL_COLUMNS)
    }

    pub fn by_uuid(uuid: &str) -> ByUuid {
        Self::all().filter(Self::with_uuid(uuid))
    }

    pub fn find_by_uuid(
        uuid: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = Self::by_uuid(&uuid).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<Self>(conn) {
            Ok(v) => Some(v),
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
        }
    }

    pub fn insert(
        stream: &NewStream,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let uuid = Uuid::new_v4();
        let q = diesel::insert_into(streams::table).values((
            streams::uuid.eq(uuid),
            streams::namespace_id.eq(stream.namespace_id),
            streams::name.eq(&stream.name),
            streams::description.eq(&stream.description),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(u) => Some(u),
        }
    }

    pub fn with_uuid(s: &str) -> WithUuid {
        let uuid = Uuid::parse_str(s).unwrap_or_else(|_| Uuid::nil());
        streams::uuid.eq(uuid)
    }

    pub fn visible() -> Visible {
        streams::archived_at.is_null()
    }
}

#[cfg(test)]
pub mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;
    use crate::model::namespace::data::NAMESPACES;

    type StreamFixture = FnvHashMap<&'static str, Stream>;

    lazy_static! {
        pub static ref STREAMS: StreamFixture = fnvhashmap! {
            "oswald's stream" => Stream {
                id: 1,
                uuid: Uuid::new_v4(),
                namespace_id: NAMESPACES.get("piano").unwrap().id,
                name: "oswald's stream".to_string(),
                description: Some("description".to_string()),
                archived_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "weenie's stream" => Stream {
                id: 2,
                uuid: Uuid::new_v4(),
                namespace_id: NAMESPACES.get("ball").unwrap().id,
                name: "weenie's stream".to_string(),
                description: Some("description".to_string()),
                archived_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "henry's stream" => Stream {
                id: 3,
                uuid: Uuid::new_v4(),
                namespace_id: NAMESPACES.get("fish").unwrap().id,
                name: "personal access token".to_string(),
                description: Some("description".to_string()),
                archived_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::namespace::{Namespace, namespaces};
    use crate::model::namespace::data::NAMESPACES;
    use crate::model::stream::data::STREAMS;
    use crate::model::test::run;

    #[test]
    fn test_new_streams_default() {
        let at = NewStream {
            ..Default::default()
        };

        assert_eq!(at.namespace_id, -1);
        assert_eq!(at.name, "".to_string());
        assert_eq!(at.description, None);
    }

    #[test]
    fn test_stream_format() {
        let s = STREAMS.get("henry's stream").unwrap();
        assert_eq!(format!("{}", s), format!("<Stream {}>", s.uuid));
    }

    #[test]
    fn test_find_by_uuid() {
        run(|conn, _, logger| {
            let ns = NAMESPACES.get("piano").unwrap();
            let namespace = diesel::insert_into(namespaces::table)
                .values(ns)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let stream = diesel::insert_into(streams::table)
                .values((
                    streams::uuid.eq(Uuid::new_v4()),
                    streams::name.eq("name"),
                    streams::namespace_id.eq(namespace.id),
                ))
                .get_result::<Stream>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result =
                Stream::find_by_uuid(&stream.uuid.to_string(), conn, logger);
            assert_eq!(result, Some(stream));
        });
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let ns = NAMESPACES.get("piano").unwrap();
            let namespace = diesel::insert_into(namespaces::table)
                .values(ns)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let s = NewStream {
                namespace_id: namespace.id,
                name: "".to_string(),
                description: None,
            };

            let result = Stream::insert(&s, conn, logger);
            assert!(result.is_some());

            let stream = result.unwrap();

            let result = streams::table
                .filter(streams::id.eq(stream.id))
                .first::<Stream>(conn)
                .expect("Failed to get a record");

            assert!(result.description.is_none());
            assert_eq!(result.namespace_id, namespace.id);

            let rows_count: i64 = streams::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);
        })
    }
}
