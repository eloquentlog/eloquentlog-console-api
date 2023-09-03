//! # Namespace
use std::fmt;
use std::str;

use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, debug_query, prelude::*};
use diesel::dsl;
use diesel::pg::{Pg, PgConnection};
use serde::Serialize;
use uuid::Uuid;

use crate::logger::Logger;
use crate::request::namespace::Namespace as RequestData;
use crate::model::membership::{Membership, memberships};
use crate::model::user::User;

pub use crate::schema::namespaces;

/// NewNamespace
#[derive(Debug)]
pub struct NewNamespace {
    pub name: String,
    pub description: Option<String>,
    pub streams_count: i64,
}

impl fmt::Display for NewNamespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<NewNamespace {name}>", name = &self.name)
    }
}

impl Default for NewNamespace {
    // includes validation errors
    fn default() -> Self {
        Self {
            name: "".to_string(),
            description: None,
            streams_count: 0,
        }
    }
}

impl From<RequestData> for NewNamespace {
    fn from(data: RequestData) -> Self {
        Self {
            name: data.name.unwrap_or_else(|| "".to_string()),
            description: data.description,
            streams_count: 0,
        }
    }
}

type AllColumns = (
    namespaces::id,
    namespaces::uuid,
    namespaces::name,
    namespaces::description,
    namespaces::streams_count,
    namespaces::archived_at,
    namespaces::created_at,
    namespaces::updated_at,
);

const ALL_COLUMNS: AllColumns = (
    namespaces::id,
    namespaces::uuid,
    namespaces::name,
    namespaces::description,
    namespaces::streams_count,
    namespaces::archived_at,
    namespaces::created_at,
    namespaces::updated_at,
);

/// Namespace
#[derive(
    AsChangeset,
    AsExpression,
    Debug,
    Identifiable,
    Insertable,
    PartialEq,
    Queryable,
    Serialize,
)]
#[table_name = "namespaces"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Namespace {
    #[serde(skip)]
    pub id: i64,
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub streams_count: i32,
    pub archived_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

mod uuid_as_string {
    use uuid::Uuid;
    use serde::{Serialize, Serializer};

    pub fn serialize<S>(val: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        val.to_string().serialize(serializer)
    }
}

impl Clone for Namespace {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            name: self.name.clone(),
            description: self.description.clone(),
            streams_count: self.streams_count,
            archived_at: None,

            ..*self
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Namespace {uuid}>", uuid = &self.uuid.to_string())
    }
}

type All = dsl::Select<namespaces::table, AllColumns>;
type Visible = dsl::IsNull<namespaces::archived_at>;
type VisibleTo = dsl::Filter<
    dsl::InnerJoin<All, memberships::table>,
    dsl::And<crate::model::membership::WithUser, Visible>,
>;
type WithUuid = dsl::Eq<namespaces::uuid, Uuid>;

impl Namespace {
    pub fn all() -> All {
        namespaces::table.select(ALL_COLUMNS)
    }

    pub fn find_all(
        user: &User,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Vec<Self>> {
        if user.id < 1 {
            return None;
        }

        let q = Self::visible_to(user);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(v) => Some(v),
        }
    }

    pub fn find_by_uuid(
        uuid: &str,
        user: &User,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        if user.id < 1 {
            return None;
        }

        let q = Self::visible_to(user)
            .filter(Self::with_uuid(uuid))
            .limit(1);

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
        namespace: &NewNamespace,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        let uuid = Uuid::new_v4();
        let q = diesel::insert_into(namespaces::table).values((
            namespaces::uuid.eq(uuid),
            namespaces::name.eq(&namespace.name),
            namespaces::description.eq(&namespace.description),
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
        namespaces::uuid.eq(uuid)
    }

    pub fn visible() -> Visible {
        namespaces::archived_at.is_null()
    }

    pub fn visible_to(user: &User) -> VisibleTo {
        Self::all()
            .inner_join(memberships::table)
            .filter(Membership::with_user(user).and(Self::visible()))
    }
}

#[cfg(test)]
pub mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;

    type NamespaceFixture = FnvHashMap<&'static str, Namespace>;

    lazy_static! {
        pub static ref NAMESPACES: NamespaceFixture = fnvhashmap! {
            "piano" => Namespace {
                id: 1,
                uuid: Uuid::new_v4(),
                name: "oswald".to_string(),
                description: Some("description".to_string()),
                streams_count: 0,
                archived_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "ball" => Namespace {
                id: 2,
                uuid: Uuid::new_v4(),
                name: "weenie".to_string(),
                description: Some("description".to_string()),
                streams_count: 0,
                archived_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "fish" => Namespace {
                id: 3,
                uuid: Uuid::new_v4(),
                name: "henry".to_string(),
                description: Some("description".to_string()),
                streams_count: 0,
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

    use crate::model::membership::{Membership, memberships};
    use crate::model::user::{User, users};

    use crate::model::membership::data::MEMBERSHIPS;
    use crate::model::namespace::data::NAMESPACES;
    use crate::model::user::data::USERS;
    use crate::model::test::run;

    #[test]
    fn test_new_namespaces_default() {
        let ns = NewNamespace {
            ..Default::default()
        };

        assert_eq!(ns.name, "".to_string());
        assert_eq!(ns.description, None);
        assert_eq!(ns.streams_count, 0);
    }

    #[test]
    fn test_namespace_format() {
        let ns = NAMESPACES.get("fish").unwrap();
        assert_eq!(format!("{}", ns), format!("<Namespace {}>", ns.uuid));
    }

    #[test]
    fn test_find_all() {
        run(|conn, _, logger| {
            let n1 = NAMESPACES.get("piano").unwrap();
            let namespace1 = diesel::insert_into(namespaces::table)
                .values(n1)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let n2 = NAMESPACES.get("ball").unwrap();
            let _ = diesel::insert_into(namespaces::table)
                .values(n2)
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = Namespace::find_all(&user, conn, logger);
            assert_eq!(result, Some(vec![]));

            let m = MEMBERSHIPS.get("oswald as a primary owner").unwrap();
            let membership = diesel::insert_into(memberships::table)
                .values(m)
                .get_result::<Membership>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            assert_eq!(membership.namespace_id, namespace1.id);
            assert_eq!(membership.user_id, user.id);

            let result = Namespace::find_all(&user, conn, logger);
            assert_eq!(result, Some(vec![namespace1]));
        });
    }

    #[test]
    fn test_find_by_uuid() {
        run(|conn, _, logger| {
            let namespace = diesel::insert_into(namespaces::table)
                .values((namespaces::name.eq("name"),))
                .get_result::<Namespace>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = Namespace::find_by_uuid(
                &namespace.uuid.to_string(),
                &user,
                conn,
                logger,
            );
            assert_eq!(result, None);

            let m = MEMBERSHIPS.get("oswald as a primary owner").unwrap();
            let _ = diesel::insert_into(memberships::table)
                .values(m)
                .get_result::<Membership>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = Namespace::find_by_uuid(
                &namespace.uuid.to_string(),
                &user,
                conn,
                logger,
            );
            assert_eq!(result, Some(namespace));
        });
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let ns = NewNamespace {
                name: "".to_string(),
                description: None,
                streams_count: 0,
            };

            let result = Namespace::insert(&ns, conn, logger);
            assert!(result.is_some());

            let namespace = result.unwrap();

            let result = namespaces::table
                .filter(namespaces::id.eq(namespace.id))
                .first::<Namespace>(conn)
                .expect("Failed to get a record");

            assert_eq!(result.streams_count, 0);
        })
    }
}
