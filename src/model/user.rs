use std::fmt;
use std::str;

use bcrypt::{hash, verify};
use chrono::NaiveDateTime;
use diesel::{Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};
use uuid::Uuid;

pub use model::user_state::*;
pub use model::user_reset_password_state::*;
pub use schema::users;

use logger::Logger;
use model::voucher::{AuthorizationClaims, Claims, VoucherData};
use request::user::UserSignUp as RequestData;

const BCRYPT_COST: u32 = 12;

/// NewUser
#[derive(Debug)]
pub struct NewUser {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: Vec<u8>,
    pub state: UserState,
    pub reset_password_state: UserResetPasswordState,
}

impl fmt::Display for NewUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<NewUser {state}>", state = &self.state)
    }
}

impl Default for NewUser {
    fn default() -> Self {
        Self {
            name: None,
            username: None,
            email: "".to_string(), // validation error
            password: vec![],      // validation error

            state: UserState::Pending,
            reset_password_state: UserResetPasswordState::Never,
        }
    }
}

impl<'a> From<&'a RequestData> for NewUser {
    fn from(data: &'a RequestData) -> Self {
        let data = data.clone();
        Self {
            name: data.name,
            username: data.username,
            email: data.email,

            ..Default::default()
        }
    }
}

impl NewUser {
    /// Returns encrypted password hash as bytes using bcrypt.
    fn encrypt_password(password: &str) -> Option<Vec<u8>> {
        match hash(password, BCRYPT_COST) {
            Ok(v) => Some(v.into_bytes()),
            Err(e) => {
                println!("err: {:?}", e);
                None
            },
        }
    }

    pub fn generate_access_token() -> String {
        // TODO
        // API access token for user
        "".to_string()
    }

    // NOTE:
    // run asynchronously? It (encrypt_password) may slow.
    pub fn set_password(&mut self, password: &str) {
        self.password = Self::encrypt_password(password).unwrap();
    }
}

/// User
#[derive(Debug, Identifiable, Queryable)]
pub struct User {
    pub id: i64,
    pub uuid: Uuid,
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: Vec<u8>,
    pub state: UserState,
    pub access_token: Option<String>,
    pub access_token_granted_at: Option<NaiveDateTime>,
    pub reset_password_state: UserResetPasswordState,
    pub reset_password_token: Option<String>,
    pub reset_password_token_expires_at: Option<NaiveDateTime>,
    pub reset_password_token_granted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<User {uuid}>", uuid = &self.uuid.to_string())
    }
}

impl User {
    pub fn check_email_uniqueness(
        email: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> bool
    {
        let q = users::table
            .select(users::id)
            .filter(users::email.eq(email))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<i64>(conn) {
            Ok(ref v) if v.is_empty() => true,
            _ => false,
        }
    }

    pub fn check_username_uniqueness(
        username: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> bool
    {
        let q = users::table
            .select(users::id)
            .filter(users::username.eq(username))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<i64>(conn) {
            Ok(ref v) if v.is_empty() => true,
            _ => false,
        }
    }

    pub fn find_by_email(
        s: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if s.is_empty() {
            return None;
        }

        let q = users::table.filter(users::email.eq(s)).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_uuid(
        s: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if s.is_empty() {
            return None;
        }

        let u = Uuid::parse_str(s).unwrap();
        let q = users::table.filter(users::uuid.eq(u)).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_email_or_uuid(
        s: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        // TODO
        // refactor find_by_xxx
        if s.contains('@') {
            User::find_by_email(s, conn, logger)
        } else if s.contains('-') {
            User::find_by_uuid(s, conn, logger)
        } else {
            None
        }
    }

    pub fn find_by_voucher<T: Claims>(
        value: &str,
        issuer: &str,
        secret: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let c = T::decode(value, issuer, secret).expect("Invalid value");
        Self::find_by_uuid(c.get_subject().as_ref(), conn, logger)
    }

    /// Save a new user into users.
    pub fn insert(
        user: &NewUser,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = diesel::insert_into(users::table).values((
            Some(users::name.eq(&user.name)),
            Some(users::username.eq(&user.username)),
            users::email.eq(&user.email),
            users::password.eq(&user.password),
            // default
            users::state.eq(UserState::Pending),
            users::reset_password_state.eq(UserResetPasswordState::Never),
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

    pub fn generate_authorization_voucher(
        &self,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> VoucherData
    {
        let subject = self.uuid.to_urn().to_string();
        AuthorizationClaims::encode(subject, issuer, key_id, secret)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &str::from_utf8(&self.password).unwrap()).unwrap()
    }
}

#[cfg(test)]
mod user_test {
    use super::*;

    use chrono::Utc;
    use uuid::Uuid;

    use model::test::run;

    #[test]
    fn test_new_user_format() {
        let u = NewUser {
            name: Some("Hennry the Penguin".to_string()),
            username: Some("hennry".to_string()),
            email: "hennry@example.org".to_string(),
            password: "password".to_string().as_bytes().to_vec(),
            state: UserState::Pending,
            reset_password_state: UserResetPasswordState::Never,
        };

        assert_eq!(format!("{}", u), "<NewUser pending>");
    }

    #[test]
    fn test_new_user_default() {
        let u = NewUser {
            ..Default::default()
        };

        assert_eq!(u.name, None);
        assert_eq!(u.username, None);
        assert_eq!(u.email, "".to_string());
        assert_eq!(u.password, Vec::new() as Vec<u8>);
        assert_eq!(u.state, UserState::Pending);
        assert_eq!(u.reset_password_state, UserResetPasswordState::Never);
    }

    #[test]
    fn test_new_user_from() {
        let data = RequestData {
            name: Some("Hennry the Penguin".to_string()),
            username: Some("hennry".to_string()),
            email: "hennry@example.org".to_string(),
            password: "password".to_string(),
        };

        let u = NewUser::from(&data);
        assert_eq!(u.name, data.name);
        assert_eq!(u.username, data.username);
        assert_eq!(u.email, data.email);
        assert_eq!(u.password, "".to_string().as_bytes().to_vec());
        assert_eq!(u.state, UserState::Pending);
        assert_eq!(u.reset_password_state, UserResetPasswordState::Never);
    }

    #[test]
    fn test_user_format() {
        let now = Utc::now().naive_utc();

        let u = User {
            id: 1,
            uuid: Uuid::nil(),
            name: Some("Hennry the Penguin".to_string()),
            username: Some("hennry".to_string()),
            email: "hennry@example.org".to_string(),
            password: Vec::new() as Vec<u8>,
            state: UserState::Pending,
            access_token: None,
            access_token_granted_at: None,
            reset_password_state: UserResetPasswordState::Never,
            reset_password_token: None,
            reset_password_token_expires_at: None,
            reset_password_token_granted_at: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(
            format!("{}", u),
            "<User 00000000-0000-0000-0000-000000000000>",
        );
    }

    #[test]
    fn test_check_email_uniqueness() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: None,
                username: None,
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn, logger);
            assert!(result.is_some());

            assert!(!User::check_email_uniqueness(&u.email, conn, logger));
            assert!(User::check_email_uniqueness(
                "unknown@example.org",
                conn,
                logger,
            ));
        });
    }

    #[test]
    fn test_check_username_uniqueness() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: None,
                username: Some("hennry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn, logger);
            assert!(result.is_some());

            assert!(!User::check_username_uniqueness(
                &u.username.unwrap(),
                conn,
                logger,
            ));
            assert!(User::check_username_uniqueness("unknown", conn, logger));
        });
    }

    #[test]
    fn test_find_by_email() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: Some("Hennry the Penguin".to_string()),
                username: Some("hennry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn, logger);
            assert!(result.is_some());

            let result = User::find_by_email(&u.email, conn, logger);
            assert!(result.is_some());

            let user = result.unwrap();
            assert_eq!(user.email, u.email);

            let result =
                User::find_by_email("unknown@example.org", conn, logger);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_uuid() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: Some("Hennry the Penguin".to_string()),
                username: Some("hennry".to_string()),
                email: "hennry@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn, logger);
            assert!(result.is_some());

            let uuid = result.unwrap().uuid;

            let result = User::find_by_uuid(&uuid.to_string(), conn, logger);
            assert!(result.is_some());

            let user = result.unwrap();
            assert_eq!(user.uuid, uuid);

            let result =
                User::find_by_uuid(&Uuid::nil().to_string(), conn, logger);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: None,
                username: None,
                email: "foo@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn, logger);
            assert!(result.is_some());

            let rows_count: i64 = users::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);
        })
    }
}
