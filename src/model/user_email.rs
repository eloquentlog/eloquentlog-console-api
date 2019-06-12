use std::fmt;

use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};

pub use model::user_email_activation_state::*;
pub use model::user_email_role::*;
pub use schema::user_emails;

use logger::Logger;
use model::voucher::{ActivationClaims, Claims, VoucherData};
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
#[derive(Associations, Debug, Identifiable, Queryable)]
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

impl UserEmail {
    /// Save a new user_email into user_emails.
    ///
    /// # Note
    ///
    /// `activation_state` is assigned always as pending. And following
    /// columns keep still remaining as NULL until granting voucher later.
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

    pub fn generate_activation_voucher(
        &self,
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> VoucherData
    {
        ActivationClaims::encode(value, issuer, key_id, secret, Utc::now())
    }

    pub fn grant_activation_voucher(
        &self,
        issuer: &str,
        key_id: &str,
        secret: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<VoucherData>
    {
        // TODO: check duplication
        let activation_token = generate_random_hash(
            ACTIVATION_HASH_SOURCE,
            ACTIVATION_HASH_LENGTH,
        );

        let voucher_data = self.generate_activation_voucher(
            activation_token.to_owned(),
            &issuer,
            &key_id,
            &secret,
        );

        let q = diesel::update(self).set((
            user_emails::activation_state.eq(UserEmailActivationState::Pending),
            user_emails::activation_token.eq(activation_token),
            // from VoucherData
            user_emails::activation_token_expires_at
                .eq(Utc.timestamp(voucher_data.expires_at, 0).naive_utc()),
            user_emails::activation_token_granted_at
                .eq(Utc.timestamp(voucher_data.granted_at, 0).naive_utc()),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(_) => Some(voucher_data),
        }
    }
}

#[cfg(test)]
mod user_email_test {
    use super::*;

    extern crate base64;
    use self::base64::decode;

    use model::user::NewUser;
    use model::test::run;

    #[test]
    fn test_new_user_emails_default() {
        let e = NewUserEmail {
            ..Default::default()
        };

        assert_eq!(e.user_id, -1);
        assert_eq!(e.email, "".to_string());
        assert_eq!(e.role, UserEmailRole::General);
        assert_eq!(e.activation_state, UserEmailActivationState::Pending);
    }

    #[test]
    fn test_new_user_email_from_user() {
        run(|conn, _, logger| {
            let email = "foo@example.org";
            let mut u = NewUser {
                name: None,
                username: None,
                email: email.to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let user = User::insert(&u, conn, logger).unwrap();

            let e = NewUserEmail::from(&user);

            assert_eq!(e.user_id, user.id);
            assert_eq!(e.email, email);
            assert_eq!(e.role, UserEmailRole::Primary);
            assert_eq!(e.activation_state, UserEmailActivationState::Pending);
        });
    }

    #[test]
    fn test_user_email_format() {
        let now = Utc::now().naive_utc();

        let e = UserEmail {
            id: 1,
            user_id: 1,
            email: Some("foo@example.org".to_string()),
            role: UserEmailRole::General,
            activation_state: UserEmailActivationState::Pending,
            activation_token: None,
            activation_token_expires_at: None,
            activation_token_granted_at: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(format!("{}", e), "<UserEmail general>");
    }

    #[test]
    #[should_panic]
    fn test_insert_should_panic_on_failure() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: Some("Hennry the Penguin".to_string()),
                username: Some("henry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let user = User::insert(&u, conn, logger).unwrap();

            let e = NewUserEmail::from(&user);
            let result = UserEmail::insert(&e, conn, logger);
            assert!(result.is_some());

            // abort: duplicate key value violates unique constraint
            let e = NewUserEmail::from(&user);
            let result = UserEmail::insert(&e, conn, logger);
            assert!(result.is_none());
        })
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: Some("Hennry the Penguin".to_string()),
                username: Some("henry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let user = User::insert(&u, conn, logger).unwrap();

            let e = NewUserEmail::from(&user);
            let result = UserEmail::insert(&e, conn, logger);
            assert!(result.is_some());

            let user_email = result.unwrap();
            assert!(user_email.id > 0);
            assert_eq!(user_email.email.unwrap(), e.email);

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);
        })
    }

    #[test]
    fn test_generation_activation_voucher() {
        let activation_token = generate_random_hash(
            ACTIVATION_HASH_SOURCE,
            ACTIVATION_HASH_LENGTH,
        );

        let now = Utc::now().naive_utc();
        let e = UserEmail {
            id: 1,
            user_id: 1,
            email: Some("foo@example.org".to_string()),
            role: UserEmailRole::General,
            activation_state: UserEmailActivationState::Pending,
            activation_token: None,
            activation_token_expires_at: None,
            activation_token_granted_at: None,
            created_at: now,
            updated_at: now,
        };

        let voucher_data = e.generate_activation_voucher(
            activation_token.to_string(),
            "test",
            "key_id",
            "secret",
        );
        let s: Vec<&str> = voucher_data.value.split('.').collect();
        assert_eq!(s.len(), 3);

        let token_body = &decode(s[1]).unwrap()[..];
        assert!(String::from_utf8_lossy(token_body)
            .contains(activation_token.as_str()));
    }

    #[test]
    fn test_grant_activation_voucher() {
        run(|conn, config, logger| {
            let mut u = NewUser {
                name: Some("Hennry the Penguin".to_string()),
                username: Some("henry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let user = User::insert(&u, conn, logger).unwrap();

            let e = NewUserEmail::from(&user);
            let result = UserEmail::insert(&e, conn, logger);
            assert!(result.is_some());

            let user_email = result.unwrap();

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);

            let voucher = user_email
                .grant_activation_voucher(
                    &config.activation_voucher_issuer,
                    &config.activation_voucher_key_id,
                    &config.activation_voucher_secret,
                    conn,
                    logger,
                )
                .expect("Failed to grant activation voucher");

            let rows_count: i64 = user_emails::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);

            let s: Vec<&str> = voucher.value.split('.').collect();
            assert_eq!(s.len(), 3);

            let token_body = &decode(s[1]).unwrap()[..];

            let user_email = user_emails::table
                .filter(user_emails::user_id.eq(user.id))
                .limit(1)
                .first::<UserEmail>(conn)
                .unwrap();

            assert!(String::from_utf8_lossy(token_body)
                .contains(user_email.activation_token.unwrap().as_str()));
        });
    }
}
