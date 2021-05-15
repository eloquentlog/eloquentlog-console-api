use std::fmt;

use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::dsl;
use diesel::pg::{Pg, PgConnection};

pub use crate::model::membership_role::*;
pub use crate::schema::memberships;

use crate::logger::Logger;
use crate::model::user::User;
use crate::model::namespace::Namespace;

/// NewMembership
#[derive(Debug)]
pub struct NewMembership {
    pub namespace_id: i64,
    pub user_id: i64,
    pub role: MembershipRole,
}

impl Default for NewMembership {
    // includes validation errors
    fn default() -> Self {
        Self {
            namespace_id: -1,
            user_id: -1,
            role: MembershipRole::PrimaryOwner,
        }
    }
}

/// Membership
#[derive(Associations, Debug, Identifiable, Insertable, Queryable)]
#[belongs_to(Namespace)]
#[belongs_to(User)]
#[table_name = "memberships"]
pub struct Membership {
    pub id: i64,
    pub namespace_id: i64,
    pub user_id: i64,
    pub role: MembershipRole,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for Membership {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Membership {role}>", role = &self.role)
    }
}

impl Clone for Membership {
    fn clone(&self) -> Self {
        let role = format!("{}", self.role);
        Membership {
            role: MembershipRole::from(role),

            ..*self
        }
    }
}

pub type WithUser = dsl::Eq<memberships::user_id, i64>;

impl Membership {
    pub fn find_by_id(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        if id < 1 {
            return None;
        }

        let q = memberships::table.filter(memberships::id.eq(id)).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<Membership>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn insert(
        membership: &NewMembership,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self> {
        let q = diesel::insert_into(memberships::table).values((
            memberships::namespace_id.eq(membership.namespace_id),
            memberships::user_id.eq(membership.user_id),
            memberships::role.eq(&membership.role),
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

    pub fn with_user(user: &User) -> WithUser {
        memberships::user_id.eq(user.id)
    }
}

#[cfg(test)]
pub mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use crate::fnvhashmap;

    type MembershipFixture = FnvHashMap<&'static str, Membership>;

    lazy_static! {
        pub static ref MEMBERSHIPS: MembershipFixture = fnvhashmap! {
            "oswald as a primary owner" => Membership {
                id: 1,
                namespace_id: 1,
                user_id: 1,
                role: MembershipRole::PrimaryOwner,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "weenie as a primary owner" => Membership {
                id: 2,
                namespace_id: 2,
                user_id: 2,
                role: MembershipRole::PrimaryOwner,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "henry as a primary owner" => Membership {
                id: 3,
                namespace_id: 3,
                user_id: 3,
                role: MembershipRole::PrimaryOwner,
                revoked_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            }
        };
    }
}
