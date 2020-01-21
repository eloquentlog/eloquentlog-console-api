//! # Access Token
//!
//! AccessToken belongs to User through agent_id and agent_type.
use std::fmt;

use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};

pub use crate::model::token::Claims;
pub use crate::model::access_token_state::*;
pub use crate::model::agent_type::*;
pub use crate::schema::access_tokens;

use crate::logger::Logger;
use crate::model::user::User;
use crate::util::generate_random_hash;

const HASH_LENGTH: i32 = 128;
const HASH_SOURCE: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";

/// NewAccessToken
#[derive(Debug)]
pub struct NewAccessToken {
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub name: String,
    pub token: Option<Vec<u8>>,
    pub state: AccessTokenState,
}

impl Default for NewAccessToken {
    fn default() -> Self {
        Self {
            agent_id: -1, // validation error
            agent_type: AgentType::Client,
            name: "".to_string(), // validation error
            token: None,          // validation error
            state: AccessTokenState::Disabled,
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

/// AccessToken
#[derive(Associations, Debug, Identifiable, Insertable, Queryable)]
#[table_name = "access_tokens"]
pub struct AccessToken {
    pub id: i64,
    pub agent_id: i64,
    pub agent_type: AgentType,
    pub name: String,
    pub token: Option<Vec<u8>>,
    pub state: AccessTokenState,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<AccessToken {state}>", state = &self.state)
    }
}

impl AccessToken {
    pub fn find_by_id(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if id < 1 {
            return None;
        }

        let q = access_tokens::table
            .filter(access_tokens::id.eq(id))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<AccessToken>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn generate_token() -> String {
        generate_random_hash(HASH_SOURCE, HASH_LENGTH)
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

        assert_eq!(at.agent_id, -1);
        assert_eq!(at.agent_type, AgentType::Client);
        assert_eq!(at.token, None);
        assert_eq!(at.state, AccessTokenState::Disabled);
    }

    #[test]
    fn test_new_access_token_from_access() {
        run(|conn, _, _| {
            let u = USERS.get("weenie").unwrap();
            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let at = NewAccessToken::from(&user);

            assert_eq!(at.agent_id, user.id);
            assert_eq!(at.agent_type, AgentType::Person);
            assert_eq!(at.token, None);
            assert_eq!(at.state, AccessTokenState::Disabled);
        });
    }

    #[test]
    fn test_access_token_format() {
        let at = ACCESS_TOKENS.get("hennry's personal token").unwrap();
        assert_eq!(format!("{}", at), format!("<AccessToken {}>", at.state));
    }
}
