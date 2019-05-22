use std::fmt;
use std::str;

use bcrypt::{hash, verify};
use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};
use jsonwebtoken::{encode, decode, Header, Validation, TokenData};
use uuid::Uuid;

pub use model::user_activation_state::*;
pub use schema::users;

use logger::Logger;
use request::User as RequestData;

const BCRYPT_COST: u32 = 12;

/// Claims
#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub uuid: String,
    pub email: String,
    pub issuer: String,
}

impl Claims {
    pub fn decode(
        jwt: &str,
        jwt_secret: &str,
    ) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error>
    {
        let v = Validation {
            // TODO
            validate_exp: false,

            ..Validation::default()
        };
        decode::<Claims>(&jwt, jwt_secret.as_ref(), &v)
    }
}

/// NewUser
#[derive(Debug, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: Vec<u8>,
    pub activation_state: UserActivationState,
    pub access_token: String,
    pub access_token_expires_at: NaiveDateTime,
}

impl fmt::Display for NewUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<NewUser {email}>", email = self.email)
    }
}

impl Default for NewUser {
    fn default() -> Self {
        Self {
            name: None,
            username: None,
            email: "".to_string(), // validation error
            password: vec![],      // validation error
            activation_state: UserActivationState::Pending,

            // TODO
            access_token: "".to_string(),
            access_token_expires_at: Utc::now().naive_utc(),
        }
    }
}

impl From<RequestData> for NewUser {
    fn from(data: RequestData) -> Self {
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
        "test".to_string()
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
    pub activation_state: UserActivationState,
    pub access_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub reset_password_token: Option<String>,
    pub reset_password_token_expires_at: Option<NaiveDateTime>,
    pub reset_password_token_sent_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<User {id}>", id = self.id)
    }
}

impl User {
    pub fn find_by_email_or_username(
        s: &str,
        conn: &PgConnection,
    ) -> Option<Self>
    {
        let q = users::table
            .filter(users::email.eq(s).or(users::username.eq(s)))
            .limit(1);

        // TODO
        // let sql = debug_query::<Pg, _>(&q).to_string();
        // println!("sql: {}", sql);

        match q.first::<User>(conn) {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn find_by_jwt(
        jwt: &str,
        jwt_secret: &str,
        conn: &PgConnection,
    ) -> Option<Self>
    {
        let c = Claims::decode(jwt, jwt_secret)
            .expect("Invalid token")
            .claims;
        Self::find_by_email_or_username(&c.email, conn)
    }

    /// Save a new user into users.
    pub fn insert(
        user: &NewUser,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<i64>
    {
        // TODO
        // * set valid access_token
        // * update access_token_expires_at
        let q = diesel::insert_into(users::table)
            .values((
                Some(users::name.eq(&user.name)),
                Some(users::username.eq(&user.username)),
                users::email.eq(&user.email),
                users::password.eq(&user.password),
                users::activation_state.eq(UserActivationState::Pending),
                users::access_token.eq(NewUser::generate_access_token()),
                users::access_token_expires_at.eq(Utc::now().naive_utc()),
            ))
            .returning(users::id);

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<i64>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(id) => Some(id),
        }
    }

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

    pub fn to_jwt(&self, issuer: &str, secret: &str) -> String {
        let c = Claims {
            uuid: self.uuid.to_urn().to_string(),
            email: self.email.clone(),
            issuer: issuer.to_string(),
        };

        encode(&Header::default(), &c, secret.as_ref()).unwrap()
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &str::from_utf8(&self.password).unwrap()).unwrap()
    }
}

#[cfg(test)]
mod user_test {
    use model::test::run;
    use super::*;

    #[test]
    fn test_insert() {
        run(|conn, logger| {
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
