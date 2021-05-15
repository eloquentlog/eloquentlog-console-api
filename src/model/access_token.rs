//! # Access Token
//!
//! AccessToken belongs to User through agent_id and agent_type.
use std::fmt;
use std::str;

use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Queryable, debug_query, prelude::*};
use diesel::dsl;
use diesel::pg::{Pg, PgConnection};
use uuid::Uuid;

pub use crate::model::access_token_state::*;
pub use crate::model::agent_type::*;
pub use crate::model::token::Claims;
pub use crate::schema::access_tokens;

use crate::logger::Logger;
use crate::model::user::User;
use crate::util::generate_random_hash;

const HASH_LENGTH: i32 = 128;
const HASH_SOURCE: &[u8] =
    b"+/ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// NewAccessToken
#[derive(Debug)]
pub struct NewAccessToken {
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub name: String,
}

impl Default for NewAccessToken {
    fn default() -> Self {
        Self {
            agent_id: 0, // validation error
            agent_type: AgentType::Client,
            name: "".to_string(), // validation error
        }
    }
}

impl<'a> From<&'a User> for NewAccessToken {
    fn from(user: &'a User) -> Self {
        Self {
            agent_id: user.id,
            agent_type: AgentType::Person,

            ..Default::default()
        }
    }
}

type AllColumns = (
    access_tokens::id,
    access_tokens::uuid,
    access_tokens::agent_id,
    access_tokens::agent_type,
    access_tokens::name,
    access_tokens::token,
    access_tokens::state,
    access_tokens::revoked_at,
    access_tokens::created_at,
    access_tokens::updated_at,
);

const ALL_COLUMNS: AllColumns = (
    access_tokens::id,
    access_tokens::uuid,
    access_tokens::agent_id,
    access_tokens::agent_type,
    access_tokens::name,
    access_tokens::token,
    access_tokens::state,
    access_tokens::revoked_at,
    access_tokens::created_at,
    access_tokens::updated_at,
);

/// AccessToken
#[derive(
    AsChangeset,
    AsExpression,
    Debug,
    Identifiable,
    Insertable,
    PartialEq,
    Queryable,
)]
#[table_name = "access_tokens"]
#[changeset_options(treat_none_as_null = "true")]
pub struct AccessToken {
    pub id: i64,
    pub uuid: Uuid,
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub name: String,
    pub token: Option<Vec<u8>>,
    pub state: AccessTokenState,
    // pub expired_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

type All = dsl::Select<access_tokens::table, AllColumns>;
type WithType = dsl::Eq<access_tokens::agent_type, AgentType>;
type WithUser = dsl::Eq<access_tokens::agent_id, i64>;
type WithUuid = dsl::Eq<access_tokens::uuid, Uuid>;
type Visible = dsl::IsNull<access_tokens::revoked_at>;
type ByUser = dsl::Filter<All, WithUser>;
type VisibleTo = dsl::Filter<All, dsl::And<WithUser, Visible>>;

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<AccessToken {uuid}>", uuid = &self.uuid.to_string())
    }
}

impl AccessToken {
    pub fn all() -> All {
        access_tokens::table.select(ALL_COLUMNS)
    }

    pub fn by_user(user: &User) -> ByUser {
        Self::all().filter(Self::with_user(user))
    }

    pub fn insert(
        access_token: &NewAccessToken,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        let uuid = Uuid::new_v4();
        let q = diesel::insert_into(access_tokens::table).values((
            access_tokens::uuid.eq(uuid),
            access_tokens::agent_id.eq(access_token.agent_id),
            access_tokens::agent_type.eq(&access_token.agent_type),
            access_tokens::name.eq(&access_token.name),
            // default
            access_tokens::state.eq(AccessTokenState::Disabled),
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

    pub fn owned_all_by_agent_type(
        user: &User,
        agent_type: AgentType,
        offset: i64,
        limit: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Vec<Self>> {
        if user.id < 1 || limit < 1 {
            return None;
        }

        let with_type = Self::with_type(agent_type);
        let q = Self::visible_to(user)
            .filter(with_type)
            .offset(offset)
            .limit(limit);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(v) => Some(v),
        }
    }

    pub fn owned_by_uuid(
        user: &User,
        uuid: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        let with_uuid = Self::with_uuid(&uuid);
        let q = Self::visible_to(user).filter(with_uuid).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<Self>(conn) {
            Ok(v) => Some(v),
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
        }
    }

    pub fn generate_token() -> String {
        generate_random_hash(HASH_SOURCE, HASH_LENGTH)
    }

    pub fn update_token(
        &mut self,
        token: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<Self, &'static str> {
        let q = diesel::update(
            access_tokens::table
                .filter(access_tokens::id.eq(self.id))
                .filter(access_tokens::state.eq(AccessTokenState::Enabled))
                .filter(access_tokens::revoked_at.is_null()),
        )
        .set(Some(access_tokens::token.eq(token.as_bytes())));
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to renovate token")
            },
            Ok(access_token) => Ok(access_token),
        }
    }

    pub fn mark_as(
        &self,
        state: AccessTokenState,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<AccessTokenState, &'static str> {
        let q = diesel::update(self).set(access_tokens::state.eq(state));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to change state")
            },
            Ok(access_token) => Ok(access_token.state),
        }
    }

    pub fn revoke(
        &self,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<Self, &'static str> {
        let now = Utc::now().naive_utc();

        // TODO: refactor
        let a = &Self {
            id: self.id,
            uuid: self.uuid,
            name: self.name.to_owned(),
            agent_id: self.agent_id,
            agent_type: AgentType::from(self.agent_type.to_string()),
            state: AccessTokenState::Disabled,
            token: None,
            revoked_at: Some(now),
            created_at: self.created_at,
            updated_at: self.updated_at,
        };
        let q = diesel::update(self).set(a);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to change state")
            },
            Ok(access_token) => Ok(access_token),
        }
    }

    pub fn visible() -> Visible {
        access_tokens::revoked_at.is_null()
    }

    pub fn visible_to(user: &User) -> VisibleTo {
        Self::all().filter(Self::with_user(user).and(Self::visible()))
    }

    pub fn with_type(agent_type: AgentType) -> WithType {
        access_tokens::agent_type.eq(agent_type)
    }

    pub fn with_user(user: &User) -> WithUser {
        access_tokens::agent_id.eq(user.id)
    }

    pub fn with_uuid(s: &str) -> WithUuid {
        let uuid = Uuid::parse_str(s).unwrap_or_else(|_| Uuid::nil());
        access_tokens::uuid.eq(uuid)
    }
}

#[cfg(test)]
pub mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;
    use crate::model::user::data::USERS;

    type AccessTokenFixture = FnvHashMap<&'static str, AccessToken>;

    lazy_static! {
        pub static ref ACCESS_TOKENS: AccessTokenFixture = fnvhashmap! {
            "oswald's personal token" => AccessToken {
                id: 1,
                uuid: Uuid::new_v4(),
                agent_id: USERS.get("oswald").unwrap().id,
                agent_type: AgentType::Person,
                name: "personal access token".to_string(),
                token: Some(b"token".to_vec()),
                state: AccessTokenState::Enabled,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "weenie's personal token" => AccessToken {
                id: 2,
                uuid: Uuid::new_v4(),
                agent_id: USERS.get("weenie").unwrap().id,
                agent_type: AgentType::Person,
                name: "personal access token".to_string(),
                token: Some(b"token".to_vec()),
                state: AccessTokenState::Enabled,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "hennry's personal token" => AccessToken {
                id: 3,
                uuid: Uuid::new_v4(),
                agent_id: USERS.get("hennry").unwrap().id,
                agent_type: AgentType::Person,
                name: "personal access token".to_string(),
                token: Some(b"token".to_vec()),
                state: AccessTokenState::Enabled,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::user::{User, users};

    use crate::model::test::run;
    use crate::model::access_token::data::ACCESS_TOKENS;
    use crate::model::user::data::USERS;

    #[test]
    fn test_new_access_tokens_default() {
        let at = NewAccessToken {
            ..Default::default()
        };

        assert_eq!(at.agent_id, 0);
        assert_eq!(at.agent_type, AgentType::Client);
        assert_eq!(at.name, "".to_string());
    }

    #[test]
    fn test_new_access_token_from_user() {
        run(|conn, _, _| {
            let u = USERS.get("weenie").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let at = NewAccessToken::from(&user);

            assert_eq!(at.agent_id, user.id);
            assert_eq!(at.agent_type, AgentType::Person);
            assert_eq!(at.name, "".to_string());
        });
    }

    #[test]
    fn test_access_token_format() {
        let at = ACCESS_TOKENS.get("hennry's personal token").unwrap();
        assert_eq!(format!("{}", at), format!("<AccessToken {}>", at.uuid));
    }

    #[test]
    fn test_owned_by_uuid_does_not_return_if_not_match() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let access_token = diesel::insert_into(access_tokens::table)
                .values((
                    access_tokens::agent_id.eq(user.id),
                    access_tokens::agent_type.eq(AgentType::Person),
                    access_tokens::name.eq("name"),
                    access_tokens::state.eq(AccessTokenState::Enabled),
                ))
                .get_result::<AccessToken>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = AccessToken::owned_by_uuid(
                &user,
                "invalid-uuid-value",
                conn,
                logger,
            );
            assert!(result.is_none());

            let u = USERS.get("hennry").unwrap();
            let another_user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = AccessToken::owned_by_uuid(
                &another_user,
                &access_token.uuid.to_string(),
                conn,
                logger,
            );
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_owned_by_uuid() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let access_token = diesel::insert_into(access_tokens::table)
                .values((
                    access_tokens::uuid.eq(Uuid::new_v4()),
                    access_tokens::agent_id.eq(user.id),
                    access_tokens::agent_type.eq(AgentType::Client),
                    access_tokens::name.eq("name"),
                    access_tokens::state.eq(AccessTokenState::Enabled),
                ))
                .get_result::<AccessToken>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = AccessToken::owned_by_uuid(
                &user,
                &access_token.uuid.to_string(),
                conn,
                logger,
            );
            assert_eq!(Some(access_token), result);
        });
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let at = NewAccessToken {
                agent_id: user.id,
                agent_type: AgentType::Person,
                name: "".to_string(),
            };

            let result = AccessToken::insert(&at, conn, logger);
            assert!(result.is_some());

            let access_token = result.unwrap();

            let result = access_tokens::table
                .filter(access_tokens::id.eq(access_token.id))
                .first::<AccessToken>(conn)
                .expect("Failed to get a record");

            assert!(result.token.is_none());
            assert_eq!(result.state, AccessTokenState::Disabled);
        })
    }
}
