use std::any::Any;
use std::fmt;
use std::str;

use bcrypt::{hash, verify};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};
use diesel::result::Error;
use uuid::Uuid;

pub use model::user_state::*;
pub use model::user_reset_password_state::*;
pub use model::token::{AuthenticationClaims, Claims, VerificationClaims};
pub use schema::users;
pub use schema::user_emails;

use model::{Activatable, Authenticatable, Verifiable};
use model::user_email::{UserEmail, UserEmailRole, UserEmailIdentificationState};
use logger::Logger;
use request::user::registration::UserRegistration as RequestData;
use util::generate_random_hash;

const BCRYPT_COST: u32 = 12;
const RESET_PASSWORD_HASH_LENGTH: i32 = 128;
const RESET_PASSWORD_HASH_SOURCE: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";

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

/// NewUser
#[derive(Debug)]
pub struct NewUser {
    pub name: Option<String>,
    pub username: String,
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
            username: "".to_string(),
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
    // NOTE:
    // run asynchronously? It (encrypt_password) may slow.
    pub fn set_password(&mut self, password: &str) {
        self.password = encrypt_password(password).unwrap();
    }
}

/// User
#[derive(Clone, Debug, Identifiable, Insertable, Queryable)]
pub struct User {
    pub id: i64,
    pub uuid: Uuid,
    pub name: Option<String>,
    pub username: String,
    pub email: String,
    pub password: Vec<u8>,
    pub state: UserState,
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

        let q = users::table
            .filter(users::email.eq(s))
            .filter(users::state.eq(UserState::Active))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_email_only_in_available_to_reset(
        s: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if s.is_empty() {
            return None;
        }

        let in_3_minutes = Utc::now().naive_utc() - Duration::minutes(3);
        let q =
            users::table
                .inner_join(user_emails::table)
                .filter(user_emails::role.eq(UserEmailRole::Primary))
                .filter(
                    user_emails::identification_state
                        .eq(UserEmailIdentificationState::Done),
                )
                .filter(users::email.eq(s))
                .filter(users::state.eq(UserState::Active))
                .filter(users::reset_password_token_granted_at.is_null().or(
                    users::reset_password_token_granted_at.lt(in_3_minutes),
                ))
                .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<(User, UserEmail)>(conn) {
            Ok(v) => Some(v.0),
            _ => None,
        }
    }

    pub fn find_by_id(
        id: i64,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if id < 1 {
            return None;
        }

        let q = users::table.filter(users::id.eq(id)).limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_primary_email_in_pending(
        s: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        if s.is_empty() {
            return None;
        }

        let q = users::table
            .inner_join(user_emails::table)
            .filter(user_emails::role.eq(UserEmailRole::Primary))
            .filter(
                user_emails::identification_state
                    .eq(UserEmailIdentificationState::Pending),
            )
            .filter(users::state.eq(UserState::Pending))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<(User, UserEmail)>(conn) {
            Ok(ref mut v) if v.len() == 1 => {
                v.pop().map(|t| Some(t.0)).unwrap_or_else(|| None)
            },
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
        let q = users::table
            .filter(users::uuid.eq(u))
            .filter(users::state.eq(UserState::Active))
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_token<T: Any + Claims>(
        token: &str,
        issuer: &str,
        secret: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let t = T::decode(token, issuer, secret).expect("invalid value");
        let c = &t as &dyn Any;
        if let Some(claims) = c.downcast_ref::<AuthenticationClaims>() {
            let uuid = claims.get_subject();
            return Self::find_by_uuid(&uuid, conn, logger);
        } else if let Some(claims) = c.downcast_ref::<VerificationClaims>() {
            let concrete_token = claims.get_subject();
            return Self::load_by_concrete_token(&concrete_token, conn, logger)
                .ok();
        }
        None
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
            users::username.eq(&user.username),
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

    pub fn generate_password_reset_token() -> String {
        generate_random_hash(
            RESET_PASSWORD_HASH_SOURCE,
            RESET_PASSWORD_HASH_LENGTH,
        )
    }

    pub fn change_password(&mut self, password: &str) {
        self.password = encrypt_password(password).unwrap();
    }

    pub fn grant_token<T: Claims>(
        &self,
        token: &str,
        issuer: &str,
        secret: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<String, &'static str>
    {
        // TODO: should we check duplication?
        let c = T::decode(token, issuer, secret).expect("Invalid value");

        // reset password
        let q = diesel::update(self).set((
            users::reset_password_state.eq(UserResetPasswordState::Pending),
            users::reset_password_token.eq(c.get_subject()),
            users::reset_password_token_expires_at.eq(c.get_expiration_time()),
            users::reset_password_token_granted_at.eq(c.get_issued_at()),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to grant token")
            },
            Ok(user) => Ok(user.reset_password_token.unwrap()),
        }
    }
}

impl Activatable for User {
    fn activate(
        &self,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<(), &'static str>
    {
        conn.build_transaction()
            .serializable()
            .deferrable()
            .read_write()
            .run::<_, diesel::result::Error, _>(|| {
                let q = users::table
                    .inner_join(user_emails::table)
                    .filter(user_emails::user_id.eq(self.id))
                    .filter(user_emails::role.eq(UserEmailRole::Primary))
                    .filter(
                        user_emails::identification_state
                            .eq(UserEmailIdentificationState::Pending),
                    )
                    .limit(1);
                info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

                let dependency = q
                    .load::<(Self, UserEmail)>(conn)
                    .map(|mut v| v.pop().unwrap().1)
                    .or_else(|e| {
                        error!(logger, "error: {}", e);
                        Err(e)
                    });

                if let Ok(user_email) = dependency {
                    if user_email.activate(conn, logger).is_err() {
                        return Err(Error::RollbackTransaction);
                    }

                    let q = diesel::update(self)
                        .set(users::state.eq(UserState::Active));
                    info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

                    return match q.get_result::<Self>(conn) {
                        Err(e) => {
                            error!(logger, "err: {}", e);
                            Err(Error::RollbackTransaction)
                        },
                        Ok(_) => Ok(()),
                    };
                }
                Err(Error::RollbackTransaction)
            })
            .map_err(|_| "activation failed")
    }
}

impl Authenticatable for User {
    fn update_password(
        &mut self,
        password: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<(), &'static str>
    {
        self.change_password(password);

        let q = diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(users::password.eq(&self.password));
        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                Err("failed to change password")
            },
            Ok(_) => Ok(()),
        }
    }

    /// Checks whether the password given as an argument is valid or not.
    /// This takes a bit long til returning the result.
    fn verify_password(&self, password: &str) -> bool {
        verify(password, &str::from_utf8(&self.password).unwrap()).unwrap()
    }
}

impl Verifiable<User> for User {
    type TokenClaims = VerificationClaims;

    fn extract_concrete_token(
        token: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<String, &'static str>
    {
        let claims = Self::TokenClaims::decode(token, issuer, secret)
            .map_err(|_| "invalid token")?;
        Ok(claims.get_subject())
    }

    fn load_by_concrete_token(
        concrete_token: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<User, &'static str>
    {
        let q = users::table
            .inner_join(user_emails::table)
            .filter(user_emails::identification_token.eq(concrete_token))
            .filter(user_emails::role.eq(UserEmailRole::Primary))
            .filter(
                user_emails::identification_state
                    .eq(UserEmailIdentificationState::Pending),
            )
            .limit(1);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.load::<(Self, UserEmail)>(conn) {
            Ok(ref mut v) if v.len() == 1 => {
                v.pop().map(|t| t.0).ok_or("unexpected :'(")
            },
            _ => Err("not found"),
        }
    }
}

#[cfg(test)]
pub mod data {
    use super::*;

    use chrono::{Utc, TimeZone};
    use fnv::FnvHashMap;
    use uuid::Uuid;

    use fnvhashmap;

    type UserFixture = FnvHashMap<&'static str, User>;

    lazy_static! {
        pub static ref USERS: UserFixture = fnvhashmap! {
            "oswald" => User {
                id: 1,
                uuid: Uuid::new_v4(),
                name: Some("Oswald".to_string()),
                username: "oswald".to_string(),
                email: "oswald@example.org".to_string(),
                password: b"Pa$$w0rd".to_vec(),
                state: UserState::Active,
                reset_password_state: UserResetPasswordState::Never,
                reset_password_token: None,
                reset_password_token_expires_at: None,
                reset_password_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "weenie" => User {
                id: 2,
                uuid: Uuid::new_v4(),
                name: Some("Weenie".to_string()),
                username: "weenie".to_string(),
                email: "weenie@example.org".to_string(),
                password: b"Pa$$w0rd".to_vec(),
                state: UserState::Active,
                reset_password_state: UserResetPasswordState::Never,
                reset_password_token: None,
                reset_password_token_expires_at: None,
                reset_password_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            },
            "hennry" => User {
                id: 3,
                uuid: Uuid::new_v4(),
                name: Some("Hennry the Penguin".to_string()),
                username: "hennry".to_string(),
                email: "hennry@example.org".to_string(),
                password: b"Pa$$w0rd".to_vec(),
                state: UserState::Pending,
                reset_password_state: UserResetPasswordState::Never,
                reset_password_token: None,
                reset_password_token_expires_at: None,
                reset_password_token_granted_at: None,
                created_at: Utc.ymd(2019, 7, 8).and_hms(10, 3, 9).naive_utc(),
                updated_at: Utc.ymd(2019, 7, 8).and_hms(10, 3, 9).naive_utc(),
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use model::test::run;
    use model::user::data::USERS;

    #[test]
    fn test_new_user_format() {
        let u = NewUser {
            name: Some("Hennry the Penguin".to_string()),
            username: "hennry".to_string(),
            email: "hennry@example.org".to_string(),
            password: b"password".to_vec(),
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
        assert_eq!(u.username, "".to_string());
        assert_eq!(u.email, "".to_string());
        assert_eq!(u.password, Vec::new() as Vec<u8>);
        assert_eq!(u.state, UserState::Pending);
        assert_eq!(u.reset_password_state, UserResetPasswordState::Never);
    }

    #[test]
    fn test_new_user_from() {
        let data = RequestData {
            name: Some("Hennry the Penguin".to_string()),
            username: "hennry".to_string(),
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
        let user = USERS.get("hennry").unwrap();
        assert_eq!(format!("{}", user), format!("<User {}>", user.uuid));
    }

    #[test]
    fn test_check_email_uniqueness() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            let email = diesel::insert_into(users::table)
                .values(u)
                .returning(users::email)
                .get_result::<String>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            assert!(!User::check_email_uniqueness(&email, conn, logger));
            assert!(User::check_email_uniqueness(
                "oswald.new@example.org",
                conn,
                logger,
            ));
        });
    }

    #[test]
    fn test_check_username_uniqueness() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            let username = diesel::insert_into(users::table)
                .values(u)
                .returning(users::username)
                .get_result::<String>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            assert!(!User::check_username_uniqueness(&username, conn, logger));
            assert!(User::check_username_uniqueness("another", conn, logger));
        });
    }

    #[test]
    fn test_find_by_primary_email_in_pending_unknown() {
        run(|conn, _, logger| {
            let result = User::find_by_primary_email_in_pending(
                "unknown@example.org",
                conn,
                logger,
            );
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_primary_email_in_pending_user_email_role_is_general() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            assert_eq!(u.state, UserState::Pending);

            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let user_email = diesel::insert_into(user_emails::table)
                .values((
                    user_emails::user_id.eq(&user.id),
                    Some(user_emails::email.eq(&user.email)),
                    user_emails::role.eq(UserEmailRole::General),
                    user_emails::identification_state
                        .eq(UserEmailIdentificationState::Pending),
                ))
                .get_result::<UserEmail>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_primary_email_in_pending(
                &user_email.email.unwrap(),
                conn,
                logger,
            );
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_primary_email_in_pending_returns_id_state_is_done() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            assert_eq!(u.state, UserState::Pending);

            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let user_email = diesel::insert_into(user_emails::table)
                .values((
                    user_emails::user_id.eq(&user.id),
                    Some(user_emails::email.eq(&user.email)),
                    user_emails::role.eq(UserEmailRole::Primary),
                    user_emails::identification_state
                        .eq(UserEmailIdentificationState::Done),
                ))
                .get_result::<UserEmail>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_primary_email_in_pending(
                &user_email.email.unwrap(),
                conn,
                logger,
            );
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_primary_email_in_pending_user_state_is_active() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            assert_eq!(u.state, UserState::Active);

            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let user_email = diesel::insert_into(user_emails::table)
                .values((
                    user_emails::user_id.eq(&user.id),
                    Some(user_emails::email.eq(&user.email)),
                    user_emails::role.eq(UserEmailRole::Primary),
                    user_emails::identification_state
                        .eq(UserEmailIdentificationState::Pending),
                ))
                .get_result::<UserEmail>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_primary_email_in_pending(
                &user_email.email.unwrap(),
                conn,
                logger,
            );
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_primary_email_in_pending() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            assert_eq!(u.state, UserState::Pending);

            let user = diesel::insert_into(users::table)
                .values(u)
                .get_result::<User>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let user_email = diesel::insert_into(user_emails::table)
                .values((
                    user_emails::user_id.eq(&user.id),
                    Some(user_emails::email.eq(&user.email)),
                    user_emails::role.eq(UserEmailRole::Primary),
                    user_emails::identification_state
                        .eq(UserEmailIdentificationState::Pending),
                ))
                .get_result::<UserEmail>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_primary_email_in_pending(
                &user_email.email.unwrap(),
                conn,
                logger,
            );
            assert!(result.is_some());

            assert_eq!(result.unwrap().id, user.id);
        });
    }

    #[test]
    fn test_find_by_email_unknown() {
        run(|conn, _, logger| {
            let result =
                User::find_by_email("unknown@example.org", conn, logger);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_email_user_state_is_pending() {
        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            assert_eq!(u.state, UserState::Pending);

            let email = diesel::insert_into(users::table)
                .values(u)
                .returning(users::email)
                .get_result::<String>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_email(&email, conn, logger);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_find_by_email() {
        run(|conn, _, logger| {
            let u = USERS.get("oswald").unwrap();
            assert_eq!(u.state, UserState::Active);

            let email = diesel::insert_into(users::table)
                .values(u)
                .returning(users::email)
                .get_result::<String>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_email(&email, conn, logger);
            assert!(result.is_some());

            let user = result.unwrap();
            assert_eq!(user.email, email);
        });
    }

    #[test]
    fn test_find_by_uuid() {
        run(|conn, _, logger| {
            let result =
                User::find_by_uuid(&Uuid::nil().to_string(), conn, logger);
            assert!(result.is_none());
        });

        run(|conn, _, logger| {
            let u = USERS.get("hennry").unwrap();
            let uuid = diesel::insert_into(users::table)
                .values(u)
                .returning(users::uuid)
                .get_result::<Uuid>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_uuid(&uuid.to_string(), conn, logger);
            assert!(result.is_none());
        });

        run(|conn, _, logger| {
            let u = USERS.get("weenie").unwrap();
            let uuid = diesel::insert_into(users::table)
                .values(u)
                .returning(users::uuid)
                .get_result::<Uuid>(conn)
                .unwrap_or_else(|e| panic!("Error at inserting: {}", e));

            let result = User::find_by_uuid(&uuid.to_string(), conn, logger);
            assert!(result.is_some());

            let user = result.unwrap();
            assert_eq!(user.uuid, uuid);
        });
    }

    #[test]
    fn test_insert() {
        run(|conn, _, logger| {
            let mut u = NewUser {
                name: Some("Johnny Snowman".to_string()),
                username: "johnny".to_string(),
                email: "johnny@example.org".to_string(),

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
