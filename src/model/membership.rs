use std::fmt;

use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};

pub use crate::model::membership_role::*;
pub use crate::schema::memberships;

use crate::logger::Logger;
use crate::model::user::User;
use crate::model::namespace::Namespace;

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

impl Membership {
    pub fn find_by_id(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
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
}
