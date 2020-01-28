pub use crate::model::role::*;

/// Membership
#[derive(Clone, Debug, Identifiable, Insertable, Queryable)]
pub struct Membership {
    pub id: i64,
    pub namespace_id: i64,
    pub role: Role,
    pub user_id: i64,
}
