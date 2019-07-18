use std::fmt;

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};

pub use model::user_email_activation_state::*;
pub use model::user_email_role::*;
pub use schema::user_emails;

use logger::Logger;
use model::user::User;
use util::generate_random_hash;

const ACTIVATION_HASH_LENGTH: i32 = 128;
const ACTIVATION_HASH_SOURCE: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";

/// NewUserEmail
#[derive(Debug)]
pub struct NewUserEmail {
    pub user_id: i64,
    pub email: String,
    pub role: UserEmailRole,
    pub activation_state: UserEmailActivationState,
}

impl Default for NewUserEmail {
    fn default() -> Self {
        Self {
            user_id: -1,           // validation error
            email: "".to_string(), // validation error
            role: UserEmailRole::General,

            activation_state: UserEmailActivationState::Pending,
        }
    }
}

impl<'a> From<&'a User> for NewUserEmail {
    fn from(user: &'a User) -> Self {
        Self {
            user_id: user.id,
            email: user.email.to_owned(),
            role: UserEmailRole::Primary,

            ..Default::default()
        }
    }
}

/// UserEmail
#[derive(Associations, Debug, Identifiable, Insertable, Queryable)]
#[belongs_to(User)]
#[table_name = "user_emails"]
pub struct UserEmail {
    pub id: i64,
    pub user_id: i64,
    pub email: Option<String>,
    pub role: UserEmailRole,
    pub activation_state: UserEmailActivationState,
    pub activation_token: Option<String>,
    pub activation_token_expires_at: Option<NaiveDateTime>,
    pub activation_token_granted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for UserEmail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<UserEmail {role}>", role = &self.role)
    }
}

impl Clone for UserEmail {
    fn clone(&self) -> Self {
        let role = format!("{}", self.role);
        UserEmail {
            role: UserEmailRole::from(role),
            email: self.email.clone(),
            activation_state: self.activation_state.clone(),
            activation_token: self.activation_token.clone(),

            ..*self
        }
    }
}

impl UserEmail {
    pub fn find_by_id(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if id < 1 {
            return None;
        }

        let q = user_emails::table.filter(user_emails::id.eq(id)).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<UserEmail>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    /// Save a new user_email into user_emails.
    ///
    /// # Note
    ///
    /// `activation_state` is assigned always as pending. And following
    /// columns keep still remaining as NULL until granting token later.
    ///
    /// * activation_token
    /// * activation_token_expires_at
    /// * activation_token_granted_at
    pub fn insert(
        user_email: &NewUserEmail,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = diesel::insert_into(user_emails::table).values((
            user_emails::user_id.eq(&user_email.user_id),
            Some(user_emails::email.eq(&user_email.email)),
            user_emails::role.eq(UserEmailRole::Primary),
            user_emails::activation_state.eq(UserEmailActivationState::Pending),
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

    pub fn grant_activation_token(
        &self,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<String, &'static str>
    {
        // TODO: check duplication
        let activation_token = generate_random_hash(
            ACTIVATION_HASH_SOURCE,
            ACTIVATION_HASH_LENGTH,
        );

        let granted_at = Utc::now();
        let expires_at = granted_at + Duration::hours(24);

        let q = diesel::update(self).set((
            user_emails::activation_state.eq(UserEmailActivationState::Pending),
            user_emails::activation_token.eq(activation_token.clone()),
            user_emails::activation_token_expires_at.eq(expires_at.naive_utc()),
            user_emails::activation_token_granted_at.eq(granted_at.naive_utc()),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to grant token")
            },
            Ok(_) => Ok(activation_token),
        }
    }
}

#[cfg(test)]
mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;

    use fnvhashmap;

    type UserEmailFixture = FnvHashMap<&'static str, UserEmail>;

    lazy_static! {
        pub static ref USER_EMAILS: UserEmailFixture = fnvhashmap! {
            "oswald's primary address" => UserEmail {
                id: 1,
                user_id: 1,
                email: Some("oswald@example.org".to_string()),
                role: UserEmailRole::Primary,
                activation_state: UserEmailActivationState::Done,
                activation_token: None,
                activation_token_expires_at: None,
                activation_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "weenie's primary address" => UserEmail {
                id: 2,
                user_id: 2,
                email: Some("weenie@example.org".to_string()),
                role: UserEmailRole::Primary,
                activation_state: UserEmailActivationState::Done,
                activation_token: None,
                activation_token_expires_at: None,
                activation_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "hennry's primary address" => UserEmail {
                id: 3,
                user_id: 3,
                email: Some("hennry@example.org".to_string()),
                role: UserEmailRole::Primary,
                activation_state: UserEmailActivationState::Done,
                activation_token: None,
                activation_token_expires_at: None,
                activation_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use model::user;

    use model::test::run;
    use model::user::data::USERS;
    use model::user_email::data::USER_EMAILS;

    #[test]
    fn test_new_user_emails_default() {
        let ue = NewUserEmail {
            ..Default::default()
        };

        assert_eq!(ue.user_id, -1);
        assert_eq!(ue.email, "".to_string());
        assert_eq!(ue.role, UserEmailRole::General);
        assert_eq!(ue.activation_state, UserEmailActivationState::Pending);
    }

    #[test]
    fn test_new_user_email_from_user() {
        run(|conn, _, _| {
            let u = USERS.get("weenie").unwrap();
            let user = diesel::insert_into(user::users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let ue = NewUserEmail::from(&user);

            assert_eq!(ue.user_id, user.id);
            assert_eq!(ue.email, user.email);
            assert_eq!(ue.role, UserEmailRole::Primary);
            assert_eq!(ue.activation_state, UserEmailActivationState::Pending);
        });
    }

    #[test]
    fn test_user_email_format() {
        let ue = USER_EMAILS.get("hennry's primary address").unwrap();
        assert_eq!(format!("{}", ue), "<UserEmail primary>");

        let mut ue = ue.clone();
        ue.role = UserEmailRole::General;
        assert_eq!(format!("{}", ue), "<UserEmail general>");
    }

    #[test]
    fn test_find_by_id() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            let user_id = diesel::insert_into(user::users::table)
                .values(u)
                .returning(user::users::id)
                .get_result::<i64>(conn)
                .unwrap_or_else(|e| panic!("Error inserting: {}", e));

            let mut ue =
                USER_EMAILS.get("hennry's primary address").unwrap().clone();
            ue.user_id = user_id;

            let id = diesel::insert_into(user_emails::table)
                .values(&ue)
                .returning(user_emails::id)
                .get_result::<i64>(conn)
                .unwrap_or_else(|e| panic!("Error inserting: {}", e));

            let result = UserEmail::find_by_id(id, conn, logger);
            let user_email = result.unwrap();
            assert_eq!(user_email.id, id);
        })
    }

    #[test]
    #[should_panic]
    fn test_insert_should_panic_on_failure() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            let user = diesel::insert_into(user::users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let ue = NewUserEmail::from(&user);
            let result = UserEmail::insert(&ue, conn, logger);
            assert!(result.is_some());

            // abort: duplicate key value violates unique constraint
            let ue = NewUserEmail::from(&user);
            let result = UserEmail::insert(&ue, conn, logger);
            assert!(result.is_none());
        })
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            let user = diesel::insert_into(user::users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let ue = NewUserEmail::from(&user);
            let result = UserEmail::insert(&ue, conn, logger);
            assert!(result.is_some());

            let user_email = result.unwrap();
            assert!(user_email.id > 0);
            assert_eq!(user_email.email.unwrap(), ue.email);

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("failed to count rows");
            assert_eq!(1, rows_count);
        })
    }

    #[test]
    fn test_grant_activation_token() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            let user = diesel::insert_into(user::users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let ue = NewUserEmail::from(&user);
            let result = UserEmail::insert(&ue, conn, logger);
            assert!(result.is_some());

            let user_email = result.unwrap();

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("failed to count rows");
            assert_eq!(1, rows_count);

            let token = user_email
                .grant_activation_token(conn, logger)
                .expect("failed to grant activation token");

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("failed to count rows");
            assert_eq!(1, rows_count);

            let user_email = user_emails::table
                .filter(user_emails::user_id.eq(user.id))
                .limit(1)
                .first::<UserEmail>(conn)
                .unwrap();

            assert_eq!(token, user_email.activation_token.unwrap());
        });
    }
}
