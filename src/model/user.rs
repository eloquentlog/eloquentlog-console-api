use std::fmt;
use std::str;

use bcrypt::{hash, verify};
use chrono::{NaiveDateTime, Utc};
use diesel::{Identifiable, Insertable, prelude::*};
use diesel::pg::PgConnection;
use diesel::pg::types::sql_types::Uuid;

// use diesel::pg::Pg;
// use diesel::debug_query;

pub use model::user_activation_state::*;
pub use schema::users;

use request::User as RequestData;

const BCRYPT_COST: u32 = 12;

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
        _s: &str,
        _conn: &PgConnection,
    ) -> Option<Self>
    {
        // TODO
        None
    }

    /// Save a new user into users.
    pub fn insert(user: &NewUser, conn: &PgConnection) -> Option<i64> {
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

        // TODO
        // let sql = debug_query::<Pg, _>(&q).to_string();
        // println!("sql: {}", sql);

        match q.get_result::<i64>(conn) {
            Err(e) => {
                println!("err: {}", e);
                None
            },
            Ok(id) => Some(id),
        }
    }

    pub fn check_email_uniqueness(email: &str, conn: &PgConnection) -> bool {
        let q = users::table
            .select(users::id)
            .filter(users::email.eq(email))
            .limit(1);

        // TODO
        // let sql = debug_query::<Pg, _>(&q).to_string();
        // println!("sql: {}", sql);

        match q.load::<i64>(conn) {
            Ok(ref v) if v.is_empty() => true,
            _ => false,
        }
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
        run(|conn| {
            let mut u = NewUser {
                name: None,
                username: None,
                email: "foo@example.org".to_string(),

                ..Default::default()
            };
            u.set_password("password");
            let result = User::insert(&u, conn);
            assert!(result.is_some());

            let rows_count: i64 = users::table
                .count()
                .first(conn)
                .expect("Failed to count rows");
            assert_eq!(1, rows_count);
        })
    }
}
